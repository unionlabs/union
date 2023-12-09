package autocli

import (
	"errors"

	"github.com/spf13/cobra"
	"google.golang.org/grpc"

	"cosmossdk.io/client/v2/autocli/flag"
	"cosmossdk.io/client/v2/autocli/keyring"

	"github.com/cosmos/cosmos-sdk/client"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
)

// Builder manages options for building CLI commands.
type Builder struct {
	// flag.Builder embeds the flag builder and its options.
	flag.Builder

	// GetClientConn specifies how CLI commands will resolve a grpc.ClientConnInterface
	// from a given context.
	GetClientConn func(*cobra.Command) (grpc.ClientConnInterface, error)

	// ClientCtx contains the necessary information needed to execute the commands.
	ClientCtx client.Context

	// TxConfigOptions is required to support sign mode textual
	TxConfigOpts authtx.ConfigOptions

	// AddQueryConnFlags and AddTxConnFlags are functions that add flags to query and transaction commands
	AddQueryConnFlags func(*cobra.Command)
	AddTxConnFlags    func(*cobra.Command)
}

// ValidateAndComplete the builder fields.
// It returns an error if any of the required fields are missing.
// If the Logger is nil, it will be set to a nop logger.
// If the keyring is nil, it will be set to a no keyring.
func (b *Builder) ValidateAndComplete() error {
	if b.Builder.AddressCodec == nil {
		return errors.New("address codec is required in flag builder")
	}

	if b.Builder.ValidatorAddressCodec == nil {
		return errors.New("validator address codec is required in flag builder")
	}

	if b.Builder.ConsensusAddressCodec == nil {
		return errors.New("consensus address codec is required in flag builder")
	}

	if b.Builder.Keyring == nil {
		b.Keyring = keyring.NoKeyring{}
	}

	if b.Builder.TypeResolver == nil {
		return errors.New("type resolver is required in flag builder")
	}

	if b.Builder.FileResolver == nil {
		return errors.New("file resolver is required in flag builder")
	}

	return nil
}
