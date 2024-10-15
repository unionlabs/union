package appmodulev2

import (
	"context"

	"cosmossdk.io/core/transaction"
)

// TxValidator represent the method that a TxValidator should implement.
// It was previously known as AnteHandler/Decorator.AnteHandle
type TxValidator[T transaction.Tx] interface {
	ValidateTx(ctx context.Context, tx T) error
}
