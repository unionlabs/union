package network

import (
	"time"

	"github.com/spf13/viper"

	"cosmossdk.io/log"

	"github.com/cosmos/cosmos-sdk/client"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	srvconfig "github.com/cosmos/cosmos-sdk/server/config"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// NetworkI is an interface for a network of validators.
// It is used to abstract over the different network types (in-process, docker, etc.).
// if used there is a requirement to expose query and tx client for the nodes
type NetworkI interface {
	// GetValidators returns the validators in the network
	GetValidators() []ValidatorI
	// WaitForNextBlock waits for the network to reach the next block
	WaitForNextBlock() error
	// WaitForHeight waits for the network to reach the given height
	WaitForHeight(height int64) (int64, error)
	// WaitForHeightWithTimeout waits for the network to reach the given height or times out
	WaitForHeightWithTimeout(int64, time.Duration) (int64, error)
	// RetryForBlocks retries the given function until it returns no error or the given number of blocks has passed
	RetryForBlocks(retryFunc func() error, blocks int) error
	// LatestHeight returns the latest height of the network
	LatestHeight() (int64, error)

	Cleanup()
}

// ValidatorI expose a validator's context and configuration
type ValidatorI interface {
	GetViper() *viper.Viper
	GetLogger() log.Logger
	GetAppConfig() *srvconfig.Config
	GetAddress() sdk.AccAddress
	GetValAddress() sdk.ValAddress
	GetClientCtx() client.Context
	GetAPIAddress() string
	GetRPCAddress() string
	GetPubKey() cryptotypes.PubKey
	GetMoniker() string
}
