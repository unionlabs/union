package types

// AnteHandler authenticates transactions, before their internal messages are
// executed. The provided ctx is expected to contain all relevant information
// needed to process the transaction, e.g. fee payment information. If new data
// is required for the remainder of the AnteHandler execution, a new Context should
// be created off of the provided Context and returned as <newCtx>.
//
// When exec module is in simulation mode (ctx.ExecMode() == ExecModeSimulate), it indicates if the AnteHandler is
// being executed in simulation mode, which attempts to estimate a gas cost for the tx.
// Any state modifications made will be discarded in simulation mode.
type AnteHandler func(ctx Context, tx Tx, _ bool) (newCtx Context, err error)

// PostHandler like AnteHandler but it executes after RunMsgs. Runs on success
// or failure and enables use cases like gas refunding.
type PostHandler func(ctx Context, tx Tx, _, success bool) (newCtx Context, err error)

// AnteDecorator wraps the next AnteHandler to perform custom pre-processing.
type AnteDecorator interface {
	AnteHandle(ctx Context, tx Tx, _ bool, next AnteHandler) (newCtx Context, err error)
}

// PostDecorator wraps the next PostHandler to perform custom post-processing.
type PostDecorator interface {
	PostHandle(ctx Context, tx Tx, _, success bool, next PostHandler) (newCtx Context, err error)
}

// ChainAnteDecorators ChainDecorator chains AnteDecorators together with each AnteDecorator
// wrapping over the decorators further along chain and returns a single AnteHandler.
//
// NOTE: The first element is outermost decorator, while the last element is innermost
// decorator. Decorator ordering is critical since some decorators will expect
// certain checks and updates to be performed (e.g. the Context) before the decorator
// is run. These expectations should be documented clearly in a CONTRACT docline
// in the decorator's godoc.
//
// NOTE: Any application that uses GasMeter to limit transaction processing cost
// MUST set GasMeter with the FIRST AnteDecorator. Failing to do so will cause
// transactions to be processed with an infinite gasmeter and open a DOS attack vector.
// Use `ante.SetUpContextDecorator` or a custom Decorator with similar functionality.
// Returns nil when no AnteDecorator are supplied.
func ChainAnteDecorators(chain ...AnteDecorator) AnteHandler {
	if len(chain) == 0 {
		return nil
	}

	handlerChain := make([]AnteHandler, len(chain)+1)
	// set the terminal AnteHandler decorator
	handlerChain[len(chain)] = func(ctx Context, tx Tx, _ bool) (Context, error) {
		return ctx, nil
	}
	for i := 0; i < len(chain); i++ {
		ii := i
		handlerChain[ii] = func(ctx Context, tx Tx, _ bool) (Context, error) {
			return chain[ii].AnteHandle(ctx, tx, ctx.ExecMode() == ExecModeSimulate, handlerChain[ii+1])
		}
	}

	return handlerChain[0]
}

// ChainPostDecorators chains PostDecorators together with each PostDecorator
// wrapping over the decorators further along chain and returns a single PostHandler.
//
// NOTE: The first element is outermost decorator, while the last element is innermost
// decorator. Decorator ordering is critical since some decorators will expect
// certain checks and updates to be performed (e.g. the Context) before the decorator
// is run. These expectations should be documented clearly in a CONTRACT docline
// in the decorator's godoc.
func ChainPostDecorators(chain ...PostDecorator) PostHandler {
	if len(chain) == 0 {
		return nil
	}

	handlerChain := make([]PostHandler, len(chain)+1)
	// set the terminal PostHandler decorator
	handlerChain[len(chain)] = func(ctx Context, tx Tx, _, success bool) (Context, error) {
		return ctx, nil
	}
	for i := 0; i < len(chain); i++ {
		ii := i
		handlerChain[ii] = func(ctx Context, tx Tx, _, success bool) (Context, error) {
			return chain[ii].PostHandle(ctx, tx, ctx.ExecMode() == ExecModeSimulate, success, handlerChain[ii+1])
		}
	}
	return handlerChain[0]
}

// Terminator AnteDecorator will get added to the chain to simplify decorator code
// Don't need to check if next == nil further up the chain
//
//	                      ______
//	                   <((((((\\\
//	                   /      . }\
//	                   ;--..--._|}
//	(\                 '--/\--'  )
//	 \\                | '-'  :'|
//	  \\               . -==- .-|
//	   \\               \.__.'   \--._
//	   [\\          __.--|       //  _/'--.
//	   \ \\       .'-._ ('-----'/ __/      \
//	    \ \\     /   __>|      | '--.       |
//	     \ \\   |   \   |     /    /       /
//	      \ '\ /     \  |     |  _/       /
//	       \  \       \ |     | /        /
//	 snd    \  \      \        /
//
// Deprecated: Terminator is retired (ref https://github.com/cosmos/cosmos-sdk/pull/16076).
type Terminator struct{}

// AnteHandle returns the provided Context and nil error
func (t Terminator) AnteHandle(ctx Context, _ Tx, _ bool, _ AnteHandler) (Context, error) {
	return ctx, nil
}

// PostHandle returns the provided Context and nil error
func (t Terminator) PostHandle(ctx Context, _ Tx, _, _ bool, _ PostHandler) (Context, error) {
	return ctx, nil
}
