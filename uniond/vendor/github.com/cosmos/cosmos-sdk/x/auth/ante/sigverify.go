package ante

import (
	"context"
	"encoding/base64"
	"encoding/hex"
	"errors"
	"fmt"

	secp256k1dcrd "github.com/decred/dcrd/dcrec/secp256k1/v4"
	"google.golang.org/protobuf/types/known/anypb"

	"cosmossdk.io/core/event"
	"cosmossdk.io/core/gas"
	"cosmossdk.io/core/transaction"
	errorsmod "cosmossdk.io/errors"
	txsigning "cosmossdk.io/x/tx/signing"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	kmultisig "github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256r1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/crypto/types/multisig"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	authsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

var (
	// simulation signature values used to estimate gas consumption
	key                = make([]byte, secp256k1.PubKeySize)
	simSecp256k1Pubkey = &secp256k1.PubKey{Key: key}
	simSecp256k1Sig    [64]byte
)

func init() {
	// This decodes a valid hex string into a sepc256k1Pubkey for use in transaction simulation
	bz, _ := hex.DecodeString("035AD6810A47F073553FF30D2FCC7E0D3B1C0B74B61A1AAA2582344037151E143A")
	copy(key, bz)
	simSecp256k1Pubkey.Key = key
}

// SignatureVerificationGasConsumer is the type of function that is used to both
// consume gas when verifying signatures and also to accept or reject different types of pubkeys
// This is where apps can define their own PubKey
type SignatureVerificationGasConsumer = func(meter gas.Meter, sig signing.SignatureV2, params types.Params) error

type AccountAbstractionKeeper interface {
	IsAbstractedAccount(ctx context.Context, addr []byte) (bool, error)
	AuthenticateAccount(ctx context.Context, signer []byte, bundler string, rawTx *tx.TxRaw, protoTx *tx.Tx, signIndex uint32) error
}

// SigVerificationDecorator verifies all signatures for a tx and returns an
// error if any are invalid.
// It will populate an account's public key if that is not present only if
// PubKey.Address() == Account.Address().
// Note, the SigVerificationDecorator will not check
// signatures on ReCheckTx. It will also increase the sequence number, and consume
// gas for signature verification.
//
// In cases where unordered or parallel transactions are desired, it is recommended
// to set unordered=true with a reasonable timeout_height value, in which case
// this nonce verification and increment will be skipped.
//
// CONTRACT: Tx must implement SigVerifiableTx interface
type SigVerificationDecorator struct {
	ak              AccountKeeper
	aaKeeper        AccountAbstractionKeeper
	signModeHandler *txsigning.HandlerMap
	sigGasConsumer  SignatureVerificationGasConsumer
}

func NewSigVerificationDecorator(ak AccountKeeper, signModeHandler *txsigning.HandlerMap, sigGasConsumer SignatureVerificationGasConsumer, aaKeeper AccountAbstractionKeeper) SigVerificationDecorator {
	return SigVerificationDecorator{
		aaKeeper:        aaKeeper,
		ak:              ak,
		signModeHandler: signModeHandler,
		sigGasConsumer:  sigGasConsumer,
	}
}

// OnlyLegacyAminoSigners checks SignatureData to see if all
// signers are using SIGN_MODE_LEGACY_AMINO_JSON. If this is the case
// then the corresponding SignatureV2 struct will not have account sequence
// explicitly set, and we should skip the explicit verification of sig.Sequence
// in the SigVerificationDecorator's AnteHandler function.
func OnlyLegacyAminoSigners(sigData signing.SignatureData) bool {
	switch v := sigData.(type) {
	case *signing.SingleSignatureData:
		return v.SignMode == signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON
	case *signing.MultiSignatureData:
		for _, s := range v.Signatures {
			if !OnlyLegacyAminoSigners(s) {
				return false
			}
		}
		return true
	default:
		return false
	}
}

func verifyIsOnCurve(pubKey cryptotypes.PubKey) (err error) {
	// when simulating pubKey.Key will always be nil
	if pubKey.Bytes() == nil {
		return nil
	}

	switch typedPubKey := pubKey.(type) {
	case *secp256k1.PubKey:
		pubKeyObject, err := secp256k1dcrd.ParsePubKey(typedPubKey.Bytes())
		if err != nil {
			if errors.Is(err, secp256k1dcrd.ErrPubKeyNotOnCurve) {
				return errorsmod.Wrap(sdkerrors.ErrInvalidPubKey, "secp256k1 key is not on curve")
			}
			return err
		}
		if !pubKeyObject.IsOnCurve() {
			return errorsmod.Wrap(sdkerrors.ErrInvalidPubKey, "secp256k1 key is not on curve")
		}

	case *secp256r1.PubKey:
		pubKeyObject := typedPubKey.Key.PublicKey
		if !pubKeyObject.IsOnCurve(pubKeyObject.X, pubKeyObject.Y) {
			return errorsmod.Wrap(sdkerrors.ErrInvalidPubKey, "secp256r1 key is not on curve")
		}

	case multisig.PubKey:
		pubKeysObjects := typedPubKey.GetPubKeys()
		ok := true
		for _, pubKeyObject := range pubKeysObjects {
			if err := verifyIsOnCurve(pubKeyObject); err != nil {
				ok = false
				break
			}
		}
		if !ok {
			return errorsmod.Wrap(sdkerrors.ErrInvalidPubKey, "some keys are not on curve")
		}

	default:
		return errorsmod.Wrapf(sdkerrors.ErrInvalidPubKey, "unsupported key type: %T", typedPubKey)
	}

	return nil
}

func (svd SigVerificationDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, _ bool, next sdk.AnteHandler) (newCtx sdk.Context, err error) {
	if err := svd.ValidateTx(ctx, tx); err != nil {
		return ctx, err
	}
	return next(ctx, tx, false)
}

func (svd SigVerificationDecorator) ValidateTx(ctx context.Context, tx transaction.Tx) error {
	sigTx, ok := tx.(authsigning.Tx)
	if !ok {
		return errorsmod.Wrap(sdkerrors.ErrTxDecode, "invalid transaction type")
	}

	// stdSigs contains the sequence number, account number, and signatures.
	// When simulating, this would just be a 0-length slice.
	signatures, err := sigTx.GetSignaturesV2()
	if err != nil {
		return err
	}

	signers, err := sigTx.GetSigners()
	if err != nil {
		return err
	}

	// check that signer length and signature length are the same
	if len(signatures) != len(signers) {
		return errorsmod.Wrapf(sdkerrors.ErrUnauthorized, "invalid number of signer;  expected: %d, got %d", len(signers), len(signatures))
	}

	pubKeys, err := sigTx.GetPubKeys()
	if err != nil {
		return err
	}

	// NOTE: the tx_wrapper implementation returns nil, in case the pubkey is not populated.
	// so we can always expect the pubkey of the signer to be at the same index as the signer
	// itself. If this does not work, it's a failure in the implementation of the interface.
	// we're erroring, but most likely we should be panicking.
	if len(pubKeys) != len(signers) {
		return errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "invalid number of pubkeys; expected %d, got %d", len(signers), len(pubKeys))
	}

	for i := range signers {
		err = svd.authenticate(ctx, sigTx, signers[i], signatures[i], pubKeys[i], i)
		if err != nil {
			return err
		}
	}

	eventMgr := svd.ak.GetEnvironment().EventService.EventManager(ctx)
	events := [][]event.Attribute{}
	for i, sig := range signatures {
		signerStr, err := svd.ak.AddressCodec().BytesToString(signers[i])
		if err != nil {
			return err
		}

		events = append(events, []event.Attribute{event.NewAttribute(sdk.AttributeKeyAccountSequence, fmt.Sprintf("%s/%d", signerStr, sig.Sequence))})

		sigBzs, err := signatureDataToBz(sig.Data)
		if err != nil {
			return err
		}
		for _, sigBz := range sigBzs {
			events = append(events, []event.Attribute{event.NewAttribute(sdk.AttributeKeySignature, base64.StdEncoding.EncodeToString(sigBz))})
		}
	}

	for _, v := range events {
		if err := eventMgr.EmitKV(sdk.EventTypeTx, v...); err != nil {
			return err
		}
	}

	return nil
}

// authenticate the authentication of the TX for a specific tx signer.
func (svd SigVerificationDecorator) authenticate(ctx context.Context, tx authsigning.Tx, signer []byte, sig signing.SignatureV2, txPubKey cryptotypes.PubKey, signerIndex int) error {
	// first we check if it's an AA
	if svd.aaKeeper != nil {
		isAa, err := svd.aaKeeper.IsAbstractedAccount(ctx, signer)
		if err != nil {
			return err
		}
		if isAa {
			return svd.authenticateAbstractedAccount(ctx, tx, signer, signerIndex)
		}
	}

	// not an AA, proceed with standard auth flow.

	// newlyCreated is a flag that indicates if the account was newly created.
	// This is only the case when the user is sending their first tx.
	newlyCreated := false
	acc := GetSignerAcc(ctx, svd.ak, signer)
	if acc == nil {
		// If the account is nil, we assume this is the account's first tx. In this case, the account needs to be
		// created, but the sign doc should use account number 0. This is because the account number is
		// not known until the account is created when the tx was signed, the account number was unknown
		// and 0 was set.
		acc = svd.ak.NewAccountWithAddress(ctx, txPubKey.Address().Bytes())
		newlyCreated = true
	}

	// the account is without a pubkey, let's attempt to check if in the
	// tx we were correctly provided a valid pubkey.
	if acc.GetPubKey() == nil {
		err := svd.setPubKey(ctx, acc, txPubKey)
		if err != nil {
			return err
		}
	}

	err := svd.consumeSignatureGas(ctx, acc.GetPubKey(), sig)
	if err != nil {
		return err
	}

	err = svd.verifySig(ctx, tx, acc, sig, newlyCreated)
	if err != nil {
		return err
	}

	err = svd.increaseSequence(tx, acc)
	if err != nil {
		return err
	}
	// update account changes in state.
	svd.ak.SetAccount(ctx, acc)
	return nil
}

// consumeSignatureGas will consume gas according to the pub-key being verified.
func (svd SigVerificationDecorator) consumeSignatureGas(
	ctx context.Context,
	pubKey cryptotypes.PubKey,
	signature signing.SignatureV2,
) error {
	if svd.ak.GetEnvironment().TransactionService.ExecMode(ctx) == transaction.ExecModeSimulate && pubKey == nil {
		pubKey = simSecp256k1Pubkey
	}

	// make a SignatureV2 with PubKey filled in from above
	signature = signing.SignatureV2{
		PubKey:   pubKey,
		Data:     signature.Data,
		Sequence: signature.Sequence,
	}

	return svd.sigGasConsumer(svd.ak.GetEnvironment().GasService.GasMeter(ctx), signature, svd.ak.GetParams(ctx))
}

// verifySig will verify the signature of the provided signer account.
func (svd SigVerificationDecorator) verifySig(ctx context.Context, tx sdk.Tx, acc sdk.AccountI, sig signing.SignatureV2, newlyCreated bool) error {
	if sig.Sequence != acc.GetSequence() {
		return errorsmod.Wrapf(
			sdkerrors.ErrWrongSequence,
			"account sequence mismatch, expected %d, got %d", acc.GetSequence(), sig.Sequence,
		)
	}

	// we're in simulation mode, or in ReCheckTx, or context is not
	// on sig verify tx, then we do not need to verify the signatures
	// in the tx.
	if svd.ak.GetEnvironment().TransactionService.ExecMode(ctx) == transaction.ExecModeSimulate ||
		isRecheckTx(ctx, svd.ak.GetEnvironment().TransactionService) ||
		!isSigverifyTx(ctx) {
		return nil
	}

	// retrieve pubkey
	pubKey := acc.GetPubKey()
	if pubKey == nil {
		return errorsmod.Wrap(sdkerrors.ErrInvalidPubKey, "pubkey on account is not set")
	}

	// retrieve signer data
	hinfo := svd.ak.GetEnvironment().HeaderService.HeaderInfo(ctx)
	genesis := hinfo.Height == 0
	chainID := hinfo.ChainID
	var accNum uint64
	// if we are not in genesis use the account number from the account
	if !genesis {
		accNum = acc.GetAccountNumber()
	}

	// if the account number is 0 and the account is signing, the sign doc will not have an account number
	if acc.GetSequence() == 0 && newlyCreated {
		// If the account sequence is 0, and we're in genesis, then we're
		// dealing with an account that has been generated but never used.
		// in this case, we should not verify signatures.
		accNum = 0
	}

	anyPk, _ := codectypes.NewAnyWithValue(pubKey)

	signerData := txsigning.SignerData{
		Address:       acc.GetAddress().String(),
		ChainID:       chainID,
		AccountNumber: accNum,
		Sequence:      acc.GetSequence(),
		PubKey: &anypb.Any{
			TypeUrl: anyPk.TypeUrl,
			Value:   anyPk.Value,
		},
	}
	adaptableTx, ok := tx.(authsigning.V2AdaptableTx)
	if !ok {
		return fmt.Errorf("expected tx to implement V2AdaptableTx, got %T", tx)
	}
	txData := adaptableTx.GetSigningTxData()
	err := authsigning.VerifySignature(ctx, pubKey, signerData, sig.Data, svd.signModeHandler, txData)
	if err != nil {
		var errMsg string
		if OnlyLegacyAminoSigners(sig.Data) {
			// If all signers are using SIGN_MODE_LEGACY_AMINO, we rely on VerifySignature to check account sequence number,
			// and therefore communicate sequence number as a potential cause of error.
			errMsg = fmt.Sprintf("signature verification failed; please verify account number (%d), sequence (%d) and chain-id (%s): (%s)", accNum, acc.GetSequence(), chainID, err.Error())
		} else {
			errMsg = fmt.Sprintf("signature verification failed; please verify account number (%d) and chain-id (%s): (%s)", accNum, chainID, err.Error())
		}
		return errorsmod.Wrap(sdkerrors.ErrUnauthorized, errMsg)
	}

	return nil
}

// setPubKey will attempt to set the pubkey for the account given the list of available public keys.
// This must be called only in case the account has not a pubkey set yet.
func (svd SigVerificationDecorator) setPubKey(ctx context.Context, acc sdk.AccountI, txPubKey cryptotypes.PubKey) error {
	// if we're not in sig verify then we can just skip.
	if !isSigverifyTx(ctx) {
		return nil
	}

	// if the pubkey is nil then we don't have any pubkey to set
	// for this account, which also means we cannot do signature
	// verification.
	if txPubKey == nil {
		// if we're not in simulation mode, and we do not have a valid pubkey
		// for this signer, then we simply error.
		if svd.ak.GetEnvironment().TransactionService.ExecMode(ctx) != transaction.ExecModeSimulate {
			return fmt.Errorf("the account %s is without a pubkey and did not provide a pubkey in the tx to set it", acc.GetAddress().String())
		}
		// if we're in simulation mode, then we can populate the pubkey with the
		// sim one and simply return.
		txPubKey = simSecp256k1Pubkey
		return acc.SetPubKey(txPubKey)
	}

	// this code path is taken when a user has received tokens but not submitted their first transaction
	// if the address does not match the pubkey, then we error.
	// TODO: in the future the relationship between address and pubkey should be more flexible.
	if !acc.GetAddress().Equals(sdk.AccAddress(txPubKey.Address().Bytes())) {
		return sdkerrors.ErrInvalidPubKey.Wrapf("the account %s cannot be claimed by public key with address %x", acc.GetAddress(), txPubKey.Address())
	}

	err := verifyIsOnCurve(txPubKey)
	if err != nil {
		return err
	}

	// we set the pubkey in the account, without setting it in state.
	// this will be done by the increaseSequenceAndUpdateAccount method.
	return acc.SetPubKey(txPubKey)
}

// increaseSequence will increase the provided account interface sequence, unless
// the tx is unordered.
func (svd SigVerificationDecorator) increaseSequence(tx authsigning.Tx, acc sdk.AccountI) error {
	// Bypass incrementing sequence for transactions with unordered set to true.
	// The actual parameters of the un-ordered tx will be checked in a separate
	// decorator.
	unorderedTx, ok := tx.(sdk.TxWithUnordered)
	if ok && unorderedTx.GetUnordered() {
		return nil
	}

	return acc.SetSequence(acc.GetSequence() + 1)
}

// authenticateAbstractedAccount computes an AA authentication instruction and invokes the auth flow on the AA.
func (svd SigVerificationDecorator) authenticateAbstractedAccount(ctx context.Context, authTx authsigning.Tx, signer []byte, index int) error {
	// the bundler is the AA itself.
	selfBundler, err := svd.ak.AddressCodec().BytesToString(signer)
	if err != nil {
		return err
	}

	infoTx := authTx.(interface {
		AsTxRaw() (*tx.TxRaw, error)
		AsTx() (*tx.Tx, error)
	})

	txRaw, err := infoTx.AsTxRaw()
	if err != nil {
		return fmt.Errorf("unable to get raw tx: %w", err)
	}

	protoTx, err := infoTx.AsTx()
	if err != nil {
		return fmt.Errorf("unable to get proto tx: %w", err)
	}

	return svd.aaKeeper.AuthenticateAccount(ctx, signer, selfBundler, txRaw, protoTx, uint32(index))
}

// ValidateSigCountDecorator takes in Params and returns errors if there are too many signatures in the tx for the given params
// otherwise it calls next AnteHandler
// Use this decorator to set parameterized limit on number of signatures in tx
// CONTRACT: Tx must implement SigVerifiableTx interface
type ValidateSigCountDecorator struct {
	ak AccountKeeper
}

func NewValidateSigCountDecorator(ak AccountKeeper) ValidateSigCountDecorator {
	return ValidateSigCountDecorator{
		ak: ak,
	}
}

// AnteHandle implements an ante decorator for ValidateSigCountDecorator
func (vscd ValidateSigCountDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, _ bool, next sdk.AnteHandler) (newCtx sdk.Context, err error) {
	if err := vscd.ValidateTx(ctx, tx); err != nil {
		return ctx, err
	}

	return next(ctx, tx, false)
}

// ValidateTx implements an TxValidator for ValidateSigCountDecorator
func (vscd ValidateSigCountDecorator) ValidateTx(ctx context.Context, tx sdk.Tx) error {
	sigTx, ok := tx.(authsigning.SigVerifiableTx)
	if !ok {
		return errorsmod.Wrap(sdkerrors.ErrTxDecode, "Tx must be a sigTx")
	}

	params := vscd.ak.GetParams(ctx)
	pubKeys, err := sigTx.GetPubKeys()
	if err != nil {
		return err
	}

	sigCount := 0
	for _, pk := range pubKeys {
		sigCount += CountSubKeys(pk)
		if uint64(sigCount) > params.TxSigLimit {
			return errorsmod.Wrapf(sdkerrors.ErrTooManySignatures, "signatures: %d, limit: %d", sigCount, params.TxSigLimit)
		}
	}

	return nil
}

// DefaultSigVerificationGasConsumer is the default implementation of SignatureVerificationGasConsumer. It consumes gas
// for signature verification based upon the public key type. The cost is fetched from the given params and is matched
// by the concrete type.
func DefaultSigVerificationGasConsumer(meter gas.Meter, sig signing.SignatureV2, params types.Params) error {
	pubkey := sig.PubKey

	switch pubkey := pubkey.(type) {
	case *ed25519.PubKey:
		if err := meter.Consume(params.SigVerifyCostED25519, "ante verify: ed25519"); err != nil {
			return err
		}
		return errorsmod.Wrap(sdkerrors.ErrInvalidPubKey, "ED25519 public keys are unsupported")

	case *secp256k1.PubKey:
		return meter.Consume(params.SigVerifyCostSecp256k1, "ante verify: secp256k1")

	case *secp256r1.PubKey:
		return meter.Consume(params.SigVerifyCostSecp256r1(), "ante verify: secp256r1")

	case multisig.PubKey:
		multisignature, ok := sig.Data.(*signing.MultiSignatureData)
		if !ok {
			return fmt.Errorf("expected %T, got, %T", &signing.MultiSignatureData{}, sig.Data)
		}

		err := ConsumeMultisignatureVerificationGas(meter, multisignature, pubkey, params, sig.Sequence)
		if err != nil {
			return err
		}

		return nil
	default:
		return errorsmod.Wrapf(sdkerrors.ErrInvalidPubKey, "unrecognized public key type: %T", pubkey)
	}
}

// ConsumeMultisignatureVerificationGas consumes gas from a GasMeter for verifying a multisig pubKey signature.
func ConsumeMultisignatureVerificationGas(
	meter gas.Meter, sig *signing.MultiSignatureData, pubKey multisig.PubKey,
	params types.Params, accSeq uint64,
) error {
	// if BitArray is nil, it means tx has been built for simulation.
	if sig.BitArray == nil {
		return multisignatureSimulationVerificationGas(meter, sig, pubKey, params, accSeq)
	}

	size := sig.BitArray.Count()
	sigIndex := 0

	for i := 0; i < size; i++ {
		if !sig.BitArray.GetIndex(i) {
			continue
		}
		sigV2 := signing.SignatureV2{
			PubKey:   pubKey.GetPubKeys()[i],
			Data:     sig.Signatures[sigIndex],
			Sequence: accSeq,
		}

		err := DefaultSigVerificationGasConsumer(meter, sigV2, params)
		if err != nil {
			return err
		}

		sigIndex++
	}

	return nil
}

// multisignatureSimulationVerificationGas consume gas for verifying a simulation multisig pubKey signature. As it's
// a simulation tx the number of signatures its equal to the multisig threshold.
func multisignatureSimulationVerificationGas(
	meter gas.Meter, sig *signing.MultiSignatureData, pubKey multisig.PubKey,
	params types.Params, accSeq uint64,
) error {
	for i := 0; i < len(sig.Signatures); i++ {
		sigV2 := signing.SignatureV2{
			PubKey:   pubKey.GetPubKeys()[i],
			Data:     sig.Signatures[i],
			Sequence: accSeq,
		}

		err := DefaultSigVerificationGasConsumer(meter, sigV2, params)
		if err != nil {
			return err
		}
	}

	return nil
}

// GetSignerAcc returns an account for a given address that is expected to sign
// a transaction.
func GetSignerAcc(ctx context.Context, ak AccountKeeper, addr sdk.AccAddress) sdk.AccountI {
	return ak.GetAccount(ctx, addr)
}

// CountSubKeys counts the total number of keys for a multi-sig public key.
// A non-multisig, i.e. a regular signature, it naturally a count of 1. If it is a multisig,
// then it recursively calls it on its pubkeys.
func CountSubKeys(pub cryptotypes.PubKey) int {
	if pub == nil {
		return 0
	}

	v, ok := pub.(*kmultisig.LegacyAminoPubKey)
	if !ok {
		return 1
	}

	numKeys := 0
	for _, subkey := range v.GetPubKeys() {
		numKeys += CountSubKeys(subkey)
	}

	return numKeys
}

// signatureDataToBz converts a SignatureData into raw bytes signature.
// For SingleSignatureData, it returns the signature raw bytes.
// For MultiSignatureData, it returns an array of all individual signatures,
// as well as the aggregated signature.
func signatureDataToBz(data signing.SignatureData) ([][]byte, error) {
	if data == nil {
		return nil, errors.New("got empty SignatureData")
	}

	switch data := data.(type) {
	case *signing.SingleSignatureData:
		return [][]byte{data.Signature}, nil

	case *signing.MultiSignatureData:
		sigs := [][]byte{}
		var err error

		for _, d := range data.Signatures {
			nestedSigs, err := signatureDataToBz(d)
			if err != nil {
				return nil, err
			}

			sigs = append(sigs, nestedSigs...)
		}

		multiSignature := cryptotypes.MultiSignature{
			Signatures: sigs,
		}

		aggregatedSig, err := multiSignature.Marshal()
		if err != nil {
			return nil, err
		}

		sigs = append(sigs, aggregatedSig)
		return sigs, nil

	default:
		return nil, sdkerrors.ErrInvalidType.Wrapf("unexpected signature data type %T", data)
	}
}

// isSigverifyTx will always return true, unless the context is a sdk.Context, in which case we will return the
// value of IsSigverifyTx.
func isSigverifyTx(ctx context.Context) bool {
	if sdkCtx, ok := sdk.TryUnwrapSDKContext(ctx); ok {
		return sdkCtx.IsSigverifyTx()
	}
	return true
}

func isRecheckTx(ctx context.Context, txSvc transaction.Service) bool {
	if sdkCtx, ok := sdk.TryUnwrapSDKContext(ctx); ok {
		return sdkCtx.IsReCheckTx()
	}
	return txSvc.ExecMode(ctx) == transaction.ExecModeReCheck
}
