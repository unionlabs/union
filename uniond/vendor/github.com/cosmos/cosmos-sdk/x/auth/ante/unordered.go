package ante

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/binary"
	"fmt"
	"sync"
	"time"

	"github.com/cosmos/gogoproto/proto"

	appmodulev2 "cosmossdk.io/core/appmodule/v2"
	"cosmossdk.io/core/transaction"
	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/auth/ante/unorderedtx"
)

// bufPool is a pool of bytes.Buffer objects to reduce memory allocations.
var bufPool = sync.Pool{
	New: func() interface{} {
		return new(bytes.Buffer)
	},
}

const DefaultSha256Cost = 25

var _ sdk.AnteDecorator = (*UnorderedTxDecorator)(nil)

// UnorderedTxDecorator defines an AnteHandler decorator that is responsible for
// checking if a transaction is intended to be unordered and if so, evaluates
// the transaction accordingly. An unordered transaction will bypass having it's
// nonce incremented, which allows fire-and-forget along with possible parallel
// transaction processing, without having to deal with nonces.
//
// The transaction sender must ensure that unordered=true and a timeout_height
// is appropriately set. The AnteHandler will check that the transaction is not
// a duplicate and will evict it from memory when the timeout is reached.
//
// The UnorderedTxDecorator should be placed as early as possible in the AnteHandler
// chain to ensure that during DeliverTx, the transaction is added to the UnorderedTxManager.
type UnorderedTxDecorator struct {
	// maxUnOrderedTTL defines the maximum TTL a transaction can define.
	maxTimeoutDuration time.Duration
	txManager          *unorderedtx.Manager
	env                appmodulev2.Environment
	sha256Cost         uint64
}

func NewUnorderedTxDecorator(
	maxDuration time.Duration,
	m *unorderedtx.Manager,
	env appmodulev2.Environment,
	gasCost uint64,
) *UnorderedTxDecorator {
	return &UnorderedTxDecorator{
		maxTimeoutDuration: maxDuration,
		txManager:          m,
		env:                env,
		sha256Cost:         gasCost,
	}
}

func (d *UnorderedTxDecorator) AnteHandle(
	ctx sdk.Context,
	tx sdk.Tx,
	_ bool,
	next sdk.AnteHandler,
) (sdk.Context, error) {
	if err := d.ValidateTx(ctx, tx); err != nil {
		return ctx, err
	}
	return next(ctx, tx, false)
}

func (d *UnorderedTxDecorator) ValidateTx(ctx context.Context, tx transaction.Tx) error {
	sdkTx, ok := tx.(sdk.Tx)
	if !ok {
		return fmt.Errorf("invalid tx type %T, expected sdk.Tx", tx)
	}

	unorderedTx, ok := tx.(sdk.TxWithUnordered)
	if !ok || !unorderedTx.GetUnordered() {
		// If the transaction does not implement unordered capabilities or has the
		// unordered value as false, we bypass.
		return nil
	}

	headerInfo := d.env.HeaderService.HeaderInfo(ctx)
	timeoutTimestamp := unorderedTx.GetTimeoutTimeStamp()
	if timeoutTimestamp.IsZero() || timeoutTimestamp.Unix() == 0 {
		return errorsmod.Wrap(
			sdkerrors.ErrInvalidRequest,
			"unordered transaction must have timeout_timestamp set",
		)
	}
	if timeoutTimestamp.Before(headerInfo.Time) {
		return errorsmod.Wrap(
			sdkerrors.ErrInvalidRequest,
			"unordered transaction has a timeout_timestamp that has already passed",
		)
	}
	if timeoutTimestamp.After(headerInfo.Time.Add(d.maxTimeoutDuration)) {
		return errorsmod.Wrapf(
			sdkerrors.ErrInvalidRequest,
			"unordered tx ttl exceeds %s",
			d.maxTimeoutDuration.String(),
		)
	}

	// consume gas in all exec modes to avoid gas estimation discrepancies
	if err := d.env.GasService.GasMeter(ctx).Consume(d.sha256Cost, "consume gas for calculating tx hash"); err != nil {
		return errorsmod.Wrap(sdkerrors.ErrOutOfGas, "out of gas")
	}

	// Avoid checking for duplicates and creating the identifier in simulation mode
	// This is done to avoid sha256 computation in simulation mode
	if d.env.TransactionService.ExecMode(ctx) == transaction.ExecModeSimulate {
		return nil
	}

	// calculate the tx hash
	txHash, err := TxIdentifier(uint64(timeoutTimestamp.Unix()), sdkTx)
	if err != nil {
		return err
	}

	// check for duplicates
	if d.txManager.Contains(txHash) {
		return errorsmod.Wrap(
			sdkerrors.ErrInvalidRequest,
			"tx %X is duplicated",
		)
	}
	if d.env.TransactionService.ExecMode(ctx) == transaction.ExecModeFinalize {
		// a new tx included in the block, add the hash to the unordered tx manager
		d.txManager.Add(txHash, timeoutTimestamp)
	}

	return nil
}

// TxIdentifier returns a unique identifier for a transaction that is intended to be unordered.
func TxIdentifier(timeout uint64, tx sdk.Tx) ([32]byte, error) {
	feetx := tx.(sdk.FeeTx)
	if feetx.GetFee().IsZero() {
		return [32]byte{}, errorsmod.Wrap(
			sdkerrors.ErrInvalidRequest,
			"unordered transaction must have a fee",
		)
	}

	buf := bufPool.Get().(*bytes.Buffer)
	// Make sure to reset the buffer
	buf.Reset()
	defer bufPool.Put(buf)

	// Use the buffer
	for _, msg := range tx.GetMsgs() {
		// loop through the messages and write them to the buffer
		// encoding the msg to bytes makes it deterministic within the state machine.
		// Malleability is not a concern here because the state machine will encode the transaction deterministically.
		bz, err := proto.Marshal(msg)
		if err != nil {
			return [32]byte{}, errorsmod.Wrap(
				sdkerrors.ErrInvalidRequest,
				"failed to marshal message",
			)
		}

		if _, err := buf.Write(bz); err != nil {
			return [32]byte{}, errorsmod.Wrap(
				sdkerrors.ErrInvalidRequest,
				"failed to write message to buffer",
			)
		}
	}

	// write the timeout height to the buffer
	if err := binary.Write(buf, binary.LittleEndian, timeout); err != nil {
		return [32]byte{}, errorsmod.Wrap(
			sdkerrors.ErrInvalidRequest,
			"failed to write timeout_height to buffer",
		)
	}

	// write gas to the buffer
	if err := binary.Write(buf, binary.LittleEndian, feetx.GetGas()); err != nil {
		return [32]byte{}, errorsmod.Wrap(
			sdkerrors.ErrInvalidRequest,
			"failed to write unordered to buffer",
		)
	}

	txHash := sha256.Sum256(buf.Bytes())

	// Return the Buffer to the pool
	return txHash, nil
}
