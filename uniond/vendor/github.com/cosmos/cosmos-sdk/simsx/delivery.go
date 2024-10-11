package simsx

import (
	"context"
	"errors"
	"fmt"
	"math/rand"

	"github.com/cosmos/cosmos-sdk/client"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

type (
	// AppEntrypoint is an alias to the simtype interface
	AppEntrypoint = simtypes.AppEntrypoint

	AccountSource interface {
		GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI
	}
	// SimDeliveryResultHandler processes the delivery response error. Some sims are supposed to fail and expect an error.
	// An unhandled error returned indicates a failure
	SimDeliveryResultHandler func(error) error
)

// DeliverSimsMsg delivers a simulation message by creating and signing a mock transaction,
// then delivering it to the application through the specified entrypoint. It returns a legacy
// operation message representing the result of the delivery.
//
// The function takes the following parameters:
// - reporter: SimulationReporter - Interface for reporting the result of the delivery
// - r: *rand.Rand - Random number generator used for creating the mock transaction
// - app: AppEntrypoint - Entry point for delivering the simulation transaction to the application
// - txGen: client.TxConfig - Configuration for generating transactions
// - ak: AccountSource - Source for retrieving accounts
// - msg: sdk.Msg - The simulation message to be delivered
// - ctx: sdk.Context - The simulation context
// - chainID: string - The chain ID
// - senders: ...SimAccount - Accounts from which to send the simulation message
//
// The function returns a simtypes.OperationMsg, which is a legacy representation of the result
// of the delivery.
func DeliverSimsMsg(
	ctx context.Context,
	reporter SimulationReporter,
	app AppEntrypoint,
	r *rand.Rand,
	txGen client.TxConfig,
	ak AccountSource,
	chainID string,
	msg sdk.Msg,
	deliveryResultHandler SimDeliveryResultHandler,
	senders ...SimAccount,
) simtypes.OperationMsg {
	if reporter.IsSkipped() {
		return reporter.ToLegacyOperationMsg()
	}
	if len(senders) == 0 {
		reporter.Fail(errors.New("no senders"), "encoding TX")
		return reporter.ToLegacyOperationMsg()
	}
	accountNumbers := make([]uint64, len(senders))
	sequenceNumbers := make([]uint64, len(senders))
	for i := 0; i < len(senders); i++ {
		acc := ak.GetAccount(ctx, senders[i].Address)
		accountNumbers[i] = acc.GetAccountNumber()
		sequenceNumbers[i] = acc.GetSequence()
	}
	fees := senders[0].LiquidBalance().RandFees()
	tx, err := sims.GenSignedMockTx(
		r,
		txGen,
		[]sdk.Msg{msg},
		fees,
		sims.DefaultGenTxGas,
		chainID,
		accountNumbers,
		sequenceNumbers,
		Collect(senders, func(a SimAccount) cryptotypes.PrivKey { return a.PrivKey })...,
	)
	if err != nil {
		reporter.Fail(err, "encoding TX")
		return reporter.ToLegacyOperationMsg()
	}
	_, _, err = app.SimDeliver(txGen.TxEncoder(), tx)
	if err2 := deliveryResultHandler(err); err2 != nil {
		var comment string
		for _, msg := range tx.GetMsgs() {
			comment += fmt.Sprintf("%#v", msg)
		}
		reporter.Fail(err2, fmt.Sprintf("delivering tx with msgs: %s", comment))
		return reporter.ToLegacyOperationMsg()
	}
	reporter.Success(msg)
	return reporter.ToLegacyOperationMsg()
}
