package ante

import (
	"context"
	"time"

	appmodulev2 "cosmossdk.io/core/appmodule/v2"
	"cosmossdk.io/core/transaction"
	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	"github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/migrations/legacytx"
	authsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
)

// ValidateBasicDecorator will call tx.ValidateBasic and return any non-nil error.
// If ValidateBasic passes, decorator calls next AnteHandler in chain. Note,
// ValidateBasicDecorator decorator will not get executed on ReCheckTx since it
// is not dependent on application state.
type ValidateBasicDecorator struct {
	env appmodulev2.Environment
}

func NewValidateBasicDecorator(env appmodulev2.Environment) ValidateBasicDecorator {
	return ValidateBasicDecorator{
		env: env,
	}
}

// AnteHandle implements an AnteHandler decorator for the ValidateBasicDecorator.
func (vbd ValidateBasicDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, _ bool, next sdk.AnteHandler) (newCtx sdk.Context, err error) {
	if err := vbd.ValidateTx(ctx, tx); err != nil {
		return ctx, err
	}

	return next(ctx, tx, false)
}

// ValidateTx implements an TxValidator for ValidateBasicDecorator
func (vbd ValidateBasicDecorator) ValidateTx(ctx context.Context, tx sdk.Tx) error {
	// no need to validate basic on recheck tx, call next antehandler
	txService := vbd.env.TransactionService
	if txService.ExecMode(ctx) == transaction.ExecModeReCheck {
		return nil
	}

	if validateBasic, ok := tx.(sdk.HasValidateBasic); ok {
		if err := validateBasic.ValidateBasic(); err != nil {
			return err
		}
	}

	return nil
}

// ValidateMemoDecorator will validate memo given the parameters passed in
// If memo is too large decorator returns with error, otherwise call next AnteHandler
// CONTRACT: Tx must implement TxWithMemo interface
type ValidateMemoDecorator struct {
	ak AccountKeeper
}

func NewValidateMemoDecorator(ak AccountKeeper) ValidateMemoDecorator {
	return ValidateMemoDecorator{
		ak: ak,
	}
}

// AnteHandle implements an AnteHandler decorator for the ValidateMemoDecorator.
func (vmd ValidateMemoDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, _ bool, next sdk.AnteHandler) (newCtx sdk.Context, err error) {
	if err := vmd.ValidateTx(ctx, tx); err != nil {
		return ctx, err
	}

	return next(ctx, tx, false)
}

// ValidateTx implements an TxValidator for ValidateMemoDecorator
func (vmd ValidateMemoDecorator) ValidateTx(ctx context.Context, tx sdk.Tx) error {
	memoTx, ok := tx.(sdk.TxWithMemo)
	if !ok {
		return errorsmod.Wrap(sdkerrors.ErrTxDecode, "invalid transaction type")
	}

	memoLength := len(memoTx.GetMemo())
	if memoLength > 0 {
		params := vmd.ak.GetParams(ctx)
		if uint64(memoLength) > params.MaxMemoCharacters {
			return errorsmod.Wrapf(sdkerrors.ErrMemoTooLarge,
				"maximum number of characters is %d but received %d characters",
				params.MaxMemoCharacters, memoLength,
			)
		}
	}

	return nil
}

// ConsumeTxSizeGasDecorator will take in parameters and consume gas proportional
// to the size of tx before calling next AnteHandler. Note, the gas costs will be
// slightly over estimated due to the fact that any given signing account may need
// to be retrieved from state.
//
// CONTRACT: If exec mode = simulate, then signatures must either be completely filled
// in or empty.
// CONTRACT: To use this decorator, signatures of transaction must be represented
// as legacytx.StdSignature otherwise simulate mode will incorrectly estimate gas cost.
type ConsumeTxSizeGasDecorator struct {
	ak AccountKeeper
}

func NewConsumeGasForTxSizeDecorator(ak AccountKeeper) ConsumeTxSizeGasDecorator {
	return ConsumeTxSizeGasDecorator{
		ak: ak,
	}
}

// AnteHandle implements an AnteHandler decorator for the ConsumeTxSizeGasDecorator.
func (cgts ConsumeTxSizeGasDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, _ bool, next sdk.AnteHandler) (newCtx sdk.Context, err error) {
	if err := cgts.ValidateTx(ctx, tx); err != nil {
		return ctx, err
	}

	return next(ctx, tx, false)
}

// ValidateTx implements an TxValidator for ConsumeTxSizeGasDecorator
func (cgts ConsumeTxSizeGasDecorator) ValidateTx(ctx context.Context, tx sdk.Tx) error {
	sigTx, ok := tx.(authsigning.SigVerifiableTx)
	if !ok {
		return errorsmod.Wrap(sdkerrors.ErrTxDecode, "invalid tx type")
	}
	params := cgts.ak.GetParams(ctx)

	gasService := cgts.ak.GetEnvironment().GasService
	if err := gasService.GasMeter(ctx).Consume(params.TxSizeCostPerByte*storetypes.Gas(len(tx.Bytes())), "txSize"); err != nil {
		return err
	}

	// simulate gas cost for signatures in simulate mode
	txService := cgts.ak.GetEnvironment().TransactionService
	if txService.ExecMode(ctx) == transaction.ExecModeSimulate {
		// in simulate mode, each element should be a nil signature
		sigs, err := sigTx.GetSignaturesV2()
		if err != nil {
			return err
		}
		n := len(sigs)

		for i, signer := range sigs {
			// if signature is already filled in, no need to simulate gas cost
			if i < n && !isIncompleteSignature(sigs[i].Data) {
				continue
			}

			var pubkey cryptotypes.PubKey

			// use placeholder simSecp256k1Pubkey if sig is nil
			if signer.PubKey == nil {
				pubkey = simSecp256k1Pubkey
			} else {
				pubkey = signer.PubKey
			}

			// use stdsignature to mock the size of a full signature
			simSig := legacytx.StdSignature{ //nolint:staticcheck // SA1019: legacytx.StdSignature is deprecated
				Signature: simSecp256k1Sig[:],
				PubKey:    pubkey,
			}

			sigBz := legacy.Cdc.MustMarshal(simSig)
			cost := storetypes.Gas(len(sigBz) + 6)

			// If the pubkey is a multi-signature pubkey, then we estimate for the maximum
			// number of signers.
			if _, ok := pubkey.(*multisig.LegacyAminoPubKey); ok {
				cost *= params.TxSigLimit
			}

			if err := gasService.GasMeter(ctx).Consume(params.TxSizeCostPerByte*cost, "txSize"); err != nil {
				return err
			}
		}
	}

	return nil
}

// isIncompleteSignature tests whether SignatureData is fully filled in for simulation purposes
func isIncompleteSignature(data signing.SignatureData) bool {
	if data == nil {
		return true
	}

	switch data := data.(type) {
	case *signing.SingleSignatureData:
		return len(data.Signature) == 0
	case *signing.MultiSignatureData:
		if len(data.Signatures) == 0 {
			return true
		}
		for _, s := range data.Signatures {
			if isIncompleteSignature(s) {
				return true
			}
		}
	}

	return false
}

type (
	// TxTimeoutHeightDecorator defines an AnteHandler decorator that checks for a
	// tx height timeout.
	TxTimeoutHeightDecorator struct {
		env appmodulev2.Environment
	}

	// TxWithTimeoutHeight defines the interface a tx must implement in order for
	// TxTimeoutHeightDecorator to process the tx.
	TxWithTimeoutHeight interface {
		sdk.Tx

		GetTimeoutHeight() uint64
		GetTimeoutTimeStamp() time.Time
	}
)

// TxTimeoutHeightDecorator defines an AnteHandler decorator that checks for a
// tx height timeout.
func NewTxTimeoutHeightDecorator(env appmodulev2.Environment) TxTimeoutHeightDecorator {
	return TxTimeoutHeightDecorator{
		env: env,
	}
}

// AnteHandle implements an AnteHandler decorator for the TxHeightTimeoutDecorator.
func (txh TxTimeoutHeightDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, _ bool, next sdk.AnteHandler) (newCtx sdk.Context, err error) {
	if err := txh.ValidateTx(ctx, tx); err != nil {
		return ctx, err
	}

	return next(ctx, tx, false)
}

// ValidateTx implements an TxValidator for the TxHeightTimeoutDecorator
// type where the current block height is checked against the tx's height timeout.
// If a height timeout is provided (non-zero) and is less than the current block
// height, then an error is returned.
func (txh TxTimeoutHeightDecorator) ValidateTx(ctx context.Context, tx sdk.Tx) error {
	timeoutTx, ok := tx.(TxWithTimeoutHeight)
	if !ok {
		return errorsmod.Wrap(sdkerrors.ErrTxDecode, "expected tx to implement TxWithTimeoutHeight")
	}

	timeoutHeight := timeoutTx.GetTimeoutHeight()
	headerInfo := txh.env.HeaderService.HeaderInfo(ctx)

	if timeoutHeight > 0 && uint64(headerInfo.Height) > timeoutHeight {
		return errorsmod.Wrapf(
			sdkerrors.ErrTxTimeoutHeight, "block height: %d, timeout height: %d", headerInfo.Height, timeoutHeight,
		)
	}

	timeoutTimestamp := timeoutTx.GetTimeoutTimeStamp()
	if !timeoutTimestamp.IsZero() && timeoutTimestamp.Unix() != 0 && timeoutTimestamp.Before(headerInfo.Time) {
		return errorsmod.Wrapf(
			sdkerrors.ErrTxTimeout, "block time: %s, timeout timestamp: %s", headerInfo.Time.String(), timeoutTimestamp.String(),
		)
	}

	return nil
}
