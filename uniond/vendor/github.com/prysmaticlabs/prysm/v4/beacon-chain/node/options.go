package node

import (
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/builder"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

// Option for beacon node configuration.
type Option func(bn *BeaconNode) error

// WithBlockchainFlagOptions includes functional options for the blockchain service related to CLI flags.
func WithBlockchainFlagOptions(opts []blockchain.Option) Option {
	return func(bn *BeaconNode) error {
		bn.serviceFlagOpts.blockchainFlagOpts = opts
		return nil
	}
}

// WithExecutionChainOptions includes functional options for the execution chain service related to CLI flags.
func WithExecutionChainOptions(opts []execution.Option) Option {
	return func(bn *BeaconNode) error {
		bn.serviceFlagOpts.executionChainFlagOpts = opts
		return nil
	}
}

// WithBuilderFlagOptions includes functional options for the builder service related to CLI flags.
func WithBuilderFlagOptions(opts []builder.Option) Option {
	return func(bn *BeaconNode) error {
		bn.serviceFlagOpts.builderOpts = opts
		return nil
	}
}

// WithBlobStorage sets the BlobStorage backend for the BeaconNode
func WithBlobStorage(bs *filesystem.BlobStorage) Option {
	return func(bn *BeaconNode) error {
		bn.BlobStorage = bs
		return nil
	}
}

// WithBlobRetentionEpochs sets the blobRetentionEpochs value, used in kv store initialization.
func WithBlobRetentionEpochs(e primitives.Epoch) Option {
	return func(bn *BeaconNode) error {
		bn.blobRetentionEpochs = e
		return nil
	}
}
