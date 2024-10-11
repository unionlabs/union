package simsx

import (
	"context"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// SimMsgFactoryX is an interface for creating and handling fuzz test-like simulation messages in the system.
type SimMsgFactoryX interface {
	// MsgType returns an empty instance of the concrete message type that the factory provides.
	// This instance is primarily used for deduplication and reporting purposes.
	// The result must not be nil
	MsgType() sdk.Msg

	// Create returns a FactoryMethod implementation which is responsible for constructing new instances of the message
	// on each invocation.
	Create() FactoryMethod

	// DeliveryResultHandler returns a SimDeliveryResultHandler instance which processes the delivery
	// response error object. While most simulation factories anticipate successful message delivery,
	// certain factories employ this handler to validate execution errors, thereby covering negative
	// test scenarios.
	DeliveryResultHandler() SimDeliveryResultHandler
}

type (
	// FactoryMethod is a method signature implemented by concrete message factories for SimMsgFactoryX
	//
	// This factory method is responsible for creating a new `sdk.Msg` instance and determining the
	// proposed signers who are expected to successfully sign the message for delivery.
	//
	// Parameters:
	// - ctx: The context for the operation
	// - testData: A pointer to a `ChainDataSource` which provides helper methods and simple access to accounts
	//   and balances within the chain.
	// - reporter: An instance of `SimulationReporter` used to report the results of the simulation.
	//   If no valid message can be provided, the factory method should call `reporter.Skip("some comment")`
	//   with both `signer` and `msg` set to nil.
	//
	// Returns:
	// - signer: A slice of `SimAccount` representing the proposed signers.
	// - msg: An instance of `sdk.Msg` representing the message to be delivered.
	FactoryMethod func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg sdk.Msg)

	// FactoryMethodWithFutureOps extended message factory method for the gov module or others that have to schedule operations for a future block.
	FactoryMethodWithFutureOps[T sdk.Msg] func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter, fOpsReg FutureOpsRegistry) ([]SimAccount, T)

	// FactoryMethodWithDeliveryResultHandler extended factory method that can return a result handler, that is executed on the delivery tx error result.
	// This is used in staking for example to validate negative execution results.
	FactoryMethodWithDeliveryResultHandler[T sdk.Msg] func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg T, handler SimDeliveryResultHandler)
)

var _ SimMsgFactoryX = &ResultHandlingSimMsgFactory[sdk.Msg]{}

// ResultHandlingSimMsgFactory message factory with a delivery error result handler configured.
type ResultHandlingSimMsgFactory[T sdk.Msg] struct {
	SimMsgFactoryFn[T]
	resultHandler SimDeliveryResultHandler
}

// NewSimMsgFactoryWithDeliveryResultHandler constructor
func NewSimMsgFactoryWithDeliveryResultHandler[T sdk.Msg](f FactoryMethodWithDeliveryResultHandler[T]) *ResultHandlingSimMsgFactory[T] {
	// the result handler is always called after the factory. so we initialize it lazy for syntactic sugar and simplicity
	// in the message factory function that is implemented by the users
	var lazyResultHandler SimDeliveryResultHandler
	r := &ResultHandlingSimMsgFactory[T]{
		resultHandler: func(err error) error {
			return lazyResultHandler(err)
		},
	}
	r.SimMsgFactoryFn = func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg T) {
		signer, msg, lazyResultHandler = f(ctx, testData, reporter)
		if lazyResultHandler == nil {
			lazyResultHandler = expectNoError
		}
		return
	}
	return r
}

func (f ResultHandlingSimMsgFactory[T]) DeliveryResultHandler() SimDeliveryResultHandler {
	return f.resultHandler
}

var (
	_ SimMsgFactoryX       = &LazyStateSimMsgFactory[sdk.Msg]{}
	_ HasFutureOpsRegistry = &LazyStateSimMsgFactory[sdk.Msg]{}
)

// LazyStateSimMsgFactory stateful message factory with weighted proposals and future operation
// registry initialized lazy before execution.
type LazyStateSimMsgFactory[T sdk.Msg] struct {
	SimMsgFactoryFn[T]
	fsOpsReg FutureOpsRegistry
}

func NewSimMsgFactoryWithFutureOps[T sdk.Msg](f FactoryMethodWithFutureOps[T]) *LazyStateSimMsgFactory[T] {
	r := &LazyStateSimMsgFactory[T]{}
	r.SimMsgFactoryFn = func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg T) {
		signer, msg = f(ctx, testData, reporter, r.fsOpsReg)
		return
	}
	return r
}

func (c *LazyStateSimMsgFactory[T]) SetFutureOpsRegistry(registry FutureOpsRegistry) {
	c.fsOpsReg = registry
}

// pass errors through and don't handle them
func expectNoError(err error) error {
	return err
}

var _ SimMsgFactoryX = SimMsgFactoryFn[sdk.Msg](nil)

// SimMsgFactoryFn is the default factory for most cases. It does not create future operations but ensures successful message delivery.
type SimMsgFactoryFn[T sdk.Msg] func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg T)

// MsgType returns an empty instance of type T, which implements `sdk.Msg`.
func (f SimMsgFactoryFn[T]) MsgType() sdk.Msg {
	var x T
	return x
}

func (f SimMsgFactoryFn[T]) Create() FactoryMethod {
	// adapter to return sdk.Msg instead of typed result to match FactoryMethod signature
	return func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) ([]SimAccount, sdk.Msg) {
		return f(ctx, testData, reporter)
	}
}

func (f SimMsgFactoryFn[T]) DeliveryResultHandler() SimDeliveryResultHandler {
	return expectNoError
}

func (f SimMsgFactoryFn[T]) Cast(msg sdk.Msg) T {
	return msg.(T)
}

type tuple struct {
	signer []SimAccount
	msg    sdk.Msg
}

// SafeRunFactoryMethod runs the factory method on a separate goroutine to abort early when the context is canceled via reporter skip
func SafeRunFactoryMethod(
	ctx context.Context,
	data *ChainDataSource,
	reporter SimulationReporter,
	f FactoryMethod,
) (signer []SimAccount, msg sdk.Msg) {
	r := make(chan tuple)
	go func() {
		defer recoverPanicForSkipped(reporter, r)
		signer, msg := f(ctx, data, reporter)
		r <- tuple{signer: signer, msg: msg}
	}()
	select {
	case t, ok := <-r:
		if !ok {
			return nil, nil
		}
		return t.signer, t.msg
	case <-ctx.Done():
		reporter.Skip("context closed")
		return nil, nil
	}
}

func recoverPanicForSkipped(reporter SimulationReporter, resultChan chan tuple) {
	if r := recover(); r != nil {
		if !reporter.IsSkipped() {
			panic(r)
		}
		close(resultChan)
	}
}
