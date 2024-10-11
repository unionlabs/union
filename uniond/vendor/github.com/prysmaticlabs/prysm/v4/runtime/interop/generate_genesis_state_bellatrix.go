// Package interop contains deterministic utilities for generating
// genesis states and keys.
package interop

import (
	"context"

	"github.com/pkg/errors"
	coreState "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	statenative "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time"
)

// GenerateGenesisStateBellatrix deterministically given a genesis time and number of validators.
// If a genesis time of 0 is supplied it is set to the current time.
func GenerateGenesisStateBellatrix(ctx context.Context, genesisTime, numValidators uint64, ep *enginev1.ExecutionPayload, ed *ethpb.Eth1Data) (*ethpb.BeaconStateBellatrix, []*ethpb.Deposit, error) {
	privKeys, pubKeys, err := DeterministicallyGenerateKeys(0 /*startIndex*/, numValidators)
	if err != nil {
		return nil, nil, errors.Wrapf(err, "could not deterministically generate keys for %d validators", numValidators)
	}
	depositDataItems, depositDataRoots, err := DepositDataFromKeys(privKeys, pubKeys)
	if err != nil {
		return nil, nil, errors.Wrap(err, "could not generate deposit data from keys")
	}
	return GenerateGenesisStateBellatrixFromDepositData(ctx, genesisTime, depositDataItems, depositDataRoots, ep, ed)
}

// GenerateGenesisStateBellatrixFromDepositData creates a genesis state given a list of
// deposit data items and their corresponding roots.
func GenerateGenesisStateBellatrixFromDepositData(
	ctx context.Context, genesisTime uint64, depositData []*ethpb.Deposit_Data, depositDataRoots [][]byte, ep *enginev1.ExecutionPayload, e1d *ethpb.Eth1Data,
) (*ethpb.BeaconStateBellatrix, []*ethpb.Deposit, error) {
	t, err := trie.GenerateTrieFromItems(depositDataRoots, params.BeaconConfig().DepositContractTreeDepth)
	if err != nil {
		return nil, nil, errors.Wrap(err, "could not generate Merkle trie for deposit proofs")
	}
	deposits, err := GenerateDepositsFromData(depositData, t)
	if err != nil {
		return nil, nil, errors.Wrap(err, "could not generate deposits from the deposit data provided")
	}
	if genesisTime == 0 {
		genesisTime = uint64(time.Now().Unix())
	}
	beaconState, err := coreState.GenesisBeaconStateBellatrix(ctx, deposits, genesisTime, e1d, ep)
	if err != nil {
		return nil, nil, errors.Wrap(err, "could not generate genesis state")
	}
	bsi := beaconState.ToProtoUnsafe()
	pbb, ok := bsi.(*ethpb.BeaconStateBellatrix)
	if !ok {
		return nil, nil, errors.New("unexpected BeaconState version")
	}
	pbState, err := statenative.ProtobufBeaconStateBellatrix(pbb)
	if err != nil {
		return nil, nil, err
	}
	return pbState, deposits, nil
}
