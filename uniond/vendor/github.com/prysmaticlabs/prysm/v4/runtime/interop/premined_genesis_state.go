package interop

import (
	"context"
	"time"

	"github.com/pkg/errors"
	coreState "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	statenative "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// GeneratePreminedGenesisState deterministically given a genesis time and number of validators.
// If a genesis time of 0 is supplied it is set to the current time.
func GeneratePreminedGenesisState(ctx context.Context, genesisTime, numValidators uint64, e1d *ethpb.Eth1Data) (*ethpb.BeaconState, []*ethpb.Deposit, error) {
	privKeys, pubKeys, err := DeterministicallyGenerateKeys(0 /*startIndex*/, numValidators)
	if err != nil {
		return nil, nil, errors.Wrapf(err, "could not deterministically generate keys for %d validators", numValidators)
	}
	depositDataItems, depositDataRoots, err := DepositDataFromKeys(privKeys, pubKeys)
	if err != nil {
		return nil, nil, errors.Wrap(err, "could not generate deposit data from keys")
	}
	return GeneratePreminedGenesisStateFromDepositData(ctx, genesisTime, depositDataItems, depositDataRoots, e1d)
}

// GeneratePreminedGenesisStateFromDepositData creates a genesis state given a list of
// deposit data items and their corresponding roots.
func GeneratePreminedGenesisStateFromDepositData(
	ctx context.Context, genesisTime uint64, depositData []*ethpb.Deposit_Data, depositDataRoots [][]byte, e1d *ethpb.Eth1Data,
) (*ethpb.BeaconState, []*ethpb.Deposit, error) {
	t, err := trie.GenerateTrieFromItems(depositDataRoots, params.BeaconConfig().DepositContractTreeDepth)
	if err != nil {
		return nil, nil, errors.Wrap(err, "could not generate Merkle trie for deposit proofs")
	}
	deposits, err := GenerateDepositsFromData(depositData, t)
	if err != nil {
		return nil, nil, errors.Wrap(err, "could not generate deposits from the deposit data provided")
	}
	root, err := t.HashTreeRoot()
	if err != nil {
		return nil, nil, errors.Wrap(err, "could not hash tree root of deposit trie")
	}
	if genesisTime == 0 {
		genesisTime = uint64(time.Now().Unix())
	}
	beaconState, err := coreState.PreminedGenesisBeaconState(ctx, deposits, genesisTime, &ethpb.Eth1Data{
		DepositRoot:  root[:],
		DepositCount: uint64(len(deposits)),
		BlockHash:    e1d.BlockHash,
	})
	if err != nil {
		return nil, nil, errors.Wrap(err, "could not generate genesis state")
	}

	pbState, err := statenative.ProtobufBeaconStatePhase0(beaconState.ToProtoUnsafe())
	if err != nil {
		return nil, nil, err
	}
	return pbState, deposits, nil
}
