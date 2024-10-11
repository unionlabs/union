package testing

import (
	"context"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// FaultyExecutionChain defines an incorrectly functioning powchain service.
type FaultyExecutionChain struct {
	ChainFeed      *event.Feed
	HashesByHeight map[int][]byte
}

// Eth2GenesisPowchainInfo --
func (*FaultyExecutionChain) Eth2GenesisPowchainInfo() (uint64, *big.Int) {
	return 0, big.NewInt(0)
}

// BlockExists --
func (f *FaultyExecutionChain) BlockExists(context.Context, common.Hash) (bool, *big.Int, error) {
	if f.HashesByHeight == nil {
		return false, big.NewInt(1), errors.New("failed")
	}

	return true, big.NewInt(1), nil
}

// BlockHashByHeight --
func (*FaultyExecutionChain) BlockHashByHeight(context.Context, *big.Int) (common.Hash, error) {
	return [32]byte{}, errors.New("failed")
}

// BlockTimeByHeight --
func (*FaultyExecutionChain) BlockTimeByHeight(context.Context, *big.Int) (uint64, error) {
	return 0, errors.New("failed")
}

// BlockByTimestamp --
func (*FaultyExecutionChain) BlockByTimestamp(context.Context, uint64) (*types.HeaderInfo, error) {
	return &types.HeaderInfo{Number: big.NewInt(0)}, nil
}

// ChainStartEth1Data --
func (*FaultyExecutionChain) ChainStartEth1Data() *ethpb.Eth1Data {
	return &ethpb.Eth1Data{}
}

// PreGenesisState --
func (*FaultyExecutionChain) PreGenesisState() state.BeaconState {
	s, err := state_native.InitializeFromProtoUnsafePhase0(&ethpb.BeaconState{})
	if err != nil {
		panic("could not initialize state")
	}
	return s
}

// ClearPreGenesisData --
func (*FaultyExecutionChain) ClearPreGenesisData() {
	// no-op
}

// IsConnectedToETH1 --
func (*FaultyExecutionChain) IsConnectedToETH1() bool {
	return true
}
