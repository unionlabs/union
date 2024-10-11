package accounts

import (
	"context"
	"errors"
	"fmt"

	"cosmossdk.io/collections"
	aa_interface_v1 "cosmossdk.io/x/accounts/interfaces/account_abstraction/v1"

	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/types/tx"
)

var (
	// ErrAuthentication is returned when the authentication fails.
	ErrAuthentication = errors.New("authentication failed")
	// ErrBundlerPayment is returned when the bundler payment fails.
	ErrBundlerPayment = errors.New("bundler payment failed")
	// ErrExecution is returned when the execution fails.
	ErrExecution = errors.New("execution failed")
)

// IsAbstractedAccount returns if the provided address is an abstracted account or not.
func (k Keeper) IsAbstractedAccount(ctx context.Context, addr []byte) (bool, error) {
	accType, err := k.AccountsByType.Get(ctx, addr)
	switch {
	case errors.Is(err, collections.ErrNotFound):
		return false, nil
	case err != nil:
		return false, err
	}

	impl, ok := k.accounts[accType]
	if !ok {
		return false, fmt.Errorf("%w: %s", errAccountTypeNotFound, accType)
	}
	return impl.HasExec(&aa_interface_v1.MsgAuthenticate{}), nil
}

func (k Keeper) AuthenticateAccount(ctx context.Context, signer []byte, bundler string, rawTx *tx.TxRaw, protoTx *tx.Tx, signIndex uint32) error {
	msg := &aa_interface_v1.MsgAuthenticate{
		Bundler:     bundler,
		RawTx:       rawTx,
		Tx:          protoTx,
		SignerIndex: signIndex,
	}
	_, err := k.Execute(ctx, signer, address.Module("accounts"), msg, nil)
	if err != nil {
		return fmt.Errorf("%w: %w", ErrAuthentication, err)
	}
	return nil
}
