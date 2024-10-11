package blocks_test

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestProcessDeposits_SameValidatorMultipleDepositsSameBlock(t *testing.T) {
	// Same validator created 3 valid deposits within the same block

	dep, _, err := util.DeterministicDepositsAndKeysSameValidator(3)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(dep))
	require.NoError(t, err)
	b := util.NewBeaconBlock()
	b.Block = &ethpb.BeaconBlock{
		Body: &ethpb.BeaconBlockBody{
			// 3 deposits from the same validator
			Deposits: []*ethpb.Deposit{dep[0], dep[1], dep[2]},
		},
	}
	registry := []*ethpb.Validator{
		{
			PublicKey:             []byte{1},
			WithdrawalCredentials: []byte{1, 2, 3},
		},
	}
	balances := []uint64{0}
	beaconState, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators: registry,
		Balances:   balances,
		Eth1Data:   eth1Data,
		Fork: &ethpb.Fork{
			PreviousVersion: params.BeaconConfig().GenesisForkVersion,
			CurrentVersion:  params.BeaconConfig().GenesisForkVersion,
		},
	})
	require.NoError(t, err)
	newState, err := blocks.ProcessDeposits(context.Background(), beaconState, b.Block.Body.Deposits)
	require.NoError(t, err, "Expected block deposits to process correctly")

	assert.Equal(t, 2, len(newState.Validators()), "Incorrect validator count")
}

func TestProcessDeposits_MerkleBranchFailsVerification(t *testing.T) {
	deposit := &ethpb.Deposit{
		Data: &ethpb.Deposit_Data{
			PublicKey:             bytesutil.PadTo([]byte{1, 2, 3}, fieldparams.BLSPubkeyLength),
			WithdrawalCredentials: make([]byte, 32),
			Signature:             make([]byte, fieldparams.BLSSignatureLength),
		},
	}
	leaf, err := deposit.Data.HashTreeRoot()
	require.NoError(t, err)

	// We then create a merkle branch for the test.
	depositTrie, err := trie.GenerateTrieFromItems([][]byte{leaf[:]}, params.BeaconConfig().DepositContractTreeDepth)
	require.NoError(t, err, "Could not generate trie")
	proof, err := depositTrie.MerkleProof(0)
	require.NoError(t, err, "Could not generate proof")

	deposit.Proof = proof
	b := util.NewBeaconBlock()
	b.Block = &ethpb.BeaconBlock{
		Body: &ethpb.BeaconBlockBody{
			Deposits: []*ethpb.Deposit{deposit},
		},
	}
	beaconState, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Eth1Data: &ethpb.Eth1Data{
			DepositRoot: []byte{0},
			BlockHash:   []byte{1},
		},
	})
	require.NoError(t, err)
	want := "deposit root did not verify"
	_, err = blocks.ProcessDeposits(context.Background(), beaconState, b.Block.Body.Deposits)
	assert.ErrorContains(t, want, err)
}

func TestProcessDeposits_AddsNewValidatorDeposit(t *testing.T) {
	dep, _, err := util.DeterministicDepositsAndKeys(1)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(dep))
	require.NoError(t, err)

	b := util.NewBeaconBlock()
	b.Block = &ethpb.BeaconBlock{
		Body: &ethpb.BeaconBlockBody{
			Deposits: []*ethpb.Deposit{dep[0]},
		},
	}
	registry := []*ethpb.Validator{
		{
			PublicKey:             []byte{1},
			WithdrawalCredentials: []byte{1, 2, 3},
		},
	}
	balances := []uint64{0}
	beaconState, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators: registry,
		Balances:   balances,
		Eth1Data:   eth1Data,
		Fork: &ethpb.Fork{
			PreviousVersion: params.BeaconConfig().GenesisForkVersion,
			CurrentVersion:  params.BeaconConfig().GenesisForkVersion,
		},
	})
	require.NoError(t, err)
	newState, err := blocks.ProcessDeposits(context.Background(), beaconState, b.Block.Body.Deposits)
	require.NoError(t, err, "Expected block deposits to process correctly")
	if newState.Balances()[1] != dep[0].Data.Amount {
		t.Errorf(
			"Expected state validator balances index 0 to equal %d, received %d",
			dep[0].Data.Amount,
			newState.Balances()[1],
		)
	}
}

func TestProcessDeposits_RepeatedDeposit_IncreasesValidatorBalance(t *testing.T) {
	sk, err := bls.RandKey()
	require.NoError(t, err)
	deposit := &ethpb.Deposit{
		Data: &ethpb.Deposit_Data{
			PublicKey:             sk.PublicKey().Marshal(),
			Amount:                1000,
			WithdrawalCredentials: make([]byte, 32),
			Signature:             make([]byte, fieldparams.BLSSignatureLength),
		},
	}
	sr, err := signing.ComputeSigningRoot(deposit.Data, bytesutil.ToBytes(3, 32))
	require.NoError(t, err)
	sig := sk.Sign(sr[:])
	deposit.Data.Signature = sig.Marshal()
	leaf, err := deposit.Data.HashTreeRoot()
	require.NoError(t, err)

	// We then create a merkle branch for the test.
	depositTrie, err := trie.GenerateTrieFromItems([][]byte{leaf[:]}, params.BeaconConfig().DepositContractTreeDepth)
	require.NoError(t, err, "Could not generate trie")
	proof, err := depositTrie.MerkleProof(0)
	require.NoError(t, err, "Could not generate proof")

	deposit.Proof = proof
	b := util.NewBeaconBlock()
	b.Block = &ethpb.BeaconBlock{
		Body: &ethpb.BeaconBlockBody{
			Deposits: []*ethpb.Deposit{deposit},
		},
	}
	registry := []*ethpb.Validator{
		{
			PublicKey: []byte{1, 2, 3},
		},
		{
			PublicKey:             sk.PublicKey().Marshal(),
			WithdrawalCredentials: []byte{1},
		},
	}
	balances := []uint64{0, 50}
	root, err := depositTrie.HashTreeRoot()
	require.NoError(t, err)
	beaconState, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators: registry,
		Balances:   balances,
		Eth1Data: &ethpb.Eth1Data{
			DepositRoot: root[:],
			BlockHash:   root[:],
		},
	})
	require.NoError(t, err)
	newState, err := blocks.ProcessDeposits(context.Background(), beaconState, b.Block.Body.Deposits)
	require.NoError(t, err, "Process deposit failed")
	assert.Equal(t, uint64(1000+50), newState.Balances()[1], "Expected balance at index 1 to be 1050")
}

func TestProcessDeposit_AddsNewValidatorDeposit(t *testing.T) {
	// Similar to TestProcessDeposits_AddsNewValidatorDeposit except that this test directly calls ProcessDeposit
	dep, _, err := util.DeterministicDepositsAndKeys(1)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(dep))
	require.NoError(t, err)

	registry := []*ethpb.Validator{
		{
			PublicKey:             []byte{1},
			WithdrawalCredentials: []byte{1, 2, 3},
		},
	}
	balances := []uint64{0}
	beaconState, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators: registry,
		Balances:   balances,
		Eth1Data:   eth1Data,
		Fork: &ethpb.Fork{
			PreviousVersion: params.BeaconConfig().GenesisForkVersion,
			CurrentVersion:  params.BeaconConfig().GenesisForkVersion,
		},
	})
	require.NoError(t, err)
	newState, isNewValidator, err := blocks.ProcessDeposit(beaconState, dep[0], true)
	require.NoError(t, err, "Process deposit failed")
	assert.Equal(t, true, isNewValidator, "Expected isNewValidator to be true")
	assert.Equal(t, 2, len(newState.Validators()), "Expected validator list to have length 2")
	assert.Equal(t, 2, len(newState.Balances()), "Expected validator balances list to have length 2")
	if newState.Balances()[1] != dep[0].Data.Amount {
		t.Errorf(
			"Expected state validator balances index 1 to equal %d, received %d",
			dep[0].Data.Amount,
			newState.Balances()[1],
		)
	}
}

func TestProcessDeposit_SkipsInvalidDeposit(t *testing.T) {
	// Same test settings as in TestProcessDeposit_AddsNewValidatorDeposit, except that we use an invalid signature
	dep, _, err := util.DeterministicDepositsAndKeys(1)
	require.NoError(t, err)
	dep[0].Data.Signature = make([]byte, 96)
	dt, _, err := util.DepositTrieFromDeposits(dep)
	require.NoError(t, err)
	root, err := dt.HashTreeRoot()
	require.NoError(t, err)
	eth1Data := &ethpb.Eth1Data{
		DepositRoot:  root[:],
		DepositCount: 1,
	}
	registry := []*ethpb.Validator{
		{
			PublicKey:             []byte{1},
			WithdrawalCredentials: []byte{1, 2, 3},
		},
	}
	balances := []uint64{0}
	beaconState, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators: registry,
		Balances:   balances,
		Eth1Data:   eth1Data,
		Fork: &ethpb.Fork{
			PreviousVersion: params.BeaconConfig().GenesisForkVersion,
			CurrentVersion:  params.BeaconConfig().GenesisForkVersion,
		},
	})
	require.NoError(t, err)
	newState, isNewValidator, err := blocks.ProcessDeposit(beaconState, dep[0], true)
	require.NoError(t, err, "Expected invalid block deposit to be ignored without error")
	assert.Equal(t, false, isNewValidator, "Expected isNewValidator to be false")

	if newState.Eth1DepositIndex() != 1 {
		t.Errorf(
			"Expected Eth1DepositIndex to be increased by 1 after processing an invalid deposit, received change: %v",
			newState.Eth1DepositIndex(),
		)
	}
	if len(newState.Validators()) != 1 {
		t.Errorf("Expected validator list to have length 1, received: %v", len(newState.Validators()))
	}
	if len(newState.Balances()) != 1 {
		t.Errorf("Expected validator balances list to have length 1, received: %v", len(newState.Balances()))
	}
	if newState.Balances()[0] != 0 {
		t.Errorf("Expected validator balance at index 0 to stay 0, received: %v", newState.Balances()[0])
	}
}

func TestPreGenesisDeposits_SkipInvalidDeposit(t *testing.T) {

	dep, _, err := util.DeterministicDepositsAndKeys(100)
	require.NoError(t, err)
	dep[0].Data.Signature = make([]byte, 96)
	dt, _, err := util.DepositTrieFromDeposits(dep)
	require.NoError(t, err)

	for i := range dep {
		proof, err := dt.MerkleProof(i)
		require.NoError(t, err)
		dep[i].Proof = proof
	}
	root, err := dt.HashTreeRoot()
	require.NoError(t, err)

	eth1Data := &ethpb.Eth1Data{
		DepositRoot:  root[:],
		DepositCount: 1,
	}
	registry := []*ethpb.Validator{
		{
			PublicKey:             []byte{1},
			WithdrawalCredentials: []byte{1, 2, 3},
		},
	}
	balances := []uint64{0}
	beaconState, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators: registry,
		Balances:   balances,
		Eth1Data:   eth1Data,
		Fork: &ethpb.Fork{
			PreviousVersion: params.BeaconConfig().GenesisForkVersion,
			CurrentVersion:  params.BeaconConfig().GenesisForkVersion,
		},
	})
	require.NoError(t, err)
	newState, err := blocks.ProcessPreGenesisDeposits(context.Background(), beaconState, dep)
	require.NoError(t, err, "Expected invalid block deposit to be ignored without error")

	_, ok := newState.ValidatorIndexByPubkey(bytesutil.ToBytes48(dep[0].Data.PublicKey))
	require.Equal(t, false, ok, "bad pubkey should not exist in state")

	for i := 1; i < newState.NumValidators(); i++ {
		val, err := newState.ValidatorAtIndex(primitives.ValidatorIndex(i))
		require.NoError(t, err)
		require.Equal(t, params.BeaconConfig().MaxEffectiveBalance, val.EffectiveBalance, "unequal effective balance")
		require.Equal(t, primitives.Epoch(0), val.ActivationEpoch)
		require.Equal(t, primitives.Epoch(0), val.ActivationEligibilityEpoch)
	}
	if newState.Eth1DepositIndex() != 100 {
		t.Errorf(
			"Expected Eth1DepositIndex to be increased by 99 after processing an invalid deposit, received change: %v",
			newState.Eth1DepositIndex(),
		)
	}
	if len(newState.Validators()) != 100 {
		t.Errorf("Expected validator list to have length 100, received: %v", len(newState.Validators()))
	}
	if len(newState.Balances()) != 100 {
		t.Errorf("Expected validator balances list to have length 100, received: %v", len(newState.Balances()))
	}
	if newState.Balances()[0] != 0 {
		t.Errorf("Expected validator balance at index 0 to stay 0, received: %v", newState.Balances()[0])
	}
}

func TestProcessDeposit_RepeatedDeposit_IncreasesValidatorBalance(t *testing.T) {
	sk, err := bls.RandKey()
	require.NoError(t, err)
	deposit := &ethpb.Deposit{
		Data: &ethpb.Deposit_Data{
			PublicKey:             sk.PublicKey().Marshal(),
			Amount:                1000,
			WithdrawalCredentials: make([]byte, 32),
			Signature:             make([]byte, 96),
		},
	}
	sr, err := signing.ComputeSigningRoot(deposit.Data, bytesutil.ToBytes(3, 32))
	require.NoError(t, err)
	sig := sk.Sign(sr[:])
	deposit.Data.Signature = sig.Marshal()
	leaf, err := deposit.Data.HashTreeRoot()
	require.NoError(t, err)

	// We then create a merkle branch for the test.
	depositTrie, err := trie.GenerateTrieFromItems([][]byte{leaf[:]}, params.BeaconConfig().DepositContractTreeDepth)
	require.NoError(t, err, "Could not generate trie")
	proof, err := depositTrie.MerkleProof(0)
	require.NoError(t, err, "Could not generate proof")

	deposit.Proof = proof
	registry := []*ethpb.Validator{
		{
			PublicKey: []byte{1, 2, 3},
		},
		{
			PublicKey:             sk.PublicKey().Marshal(),
			WithdrawalCredentials: []byte{1},
		},
	}
	balances := []uint64{0, 50}
	root, err := depositTrie.HashTreeRoot()
	require.NoError(t, err)

	beaconState, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators: registry,
		Balances:   balances,
		Eth1Data: &ethpb.Eth1Data{
			DepositRoot: root[:],
			BlockHash:   root[:],
		},
	})
	require.NoError(t, err)
	newState, isNewValidator, err := blocks.ProcessDeposit(beaconState, deposit, true /*verifySignature*/)
	require.NoError(t, err, "Process deposit failed")
	assert.Equal(t, false, isNewValidator, "Expected isNewValidator to be false")
	assert.Equal(t, uint64(1000+50), newState.Balances()[1], "Expected balance at index 1 to be 1050")
}
