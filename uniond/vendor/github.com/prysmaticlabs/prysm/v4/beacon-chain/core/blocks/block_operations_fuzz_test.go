package blocks

import (
	"context"
	"testing"

	fuzz "github.com/google/gofuzz"
	v "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/validators"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestFuzzProcessAttestationNoVerify_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	ctx := context.Background()
	state := &ethpb.BeaconState{}
	att := &ethpb.Attestation{}

	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(att)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		_, err = ProcessAttestationNoVerifySignature(ctx, s, att)
		_ = err
	}
}

func TestFuzzProcessBlockHeader_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	block := &ethpb.SignedBeaconBlock{}

	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(block)

		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		if block.Block == nil || block.Block.Body == nil || block.Block.Body.Eth1Data == nil {
			continue
		}
		wsb, err := blocks.NewSignedBeaconBlock(block)
		require.NoError(t, err)
		_, err = ProcessBlockHeader(context.Background(), s, wsb)
		_ = err
	}
}

func TestFuzzverifyDepositDataSigningRoot_10000(_ *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	var ba []byte
	var pubkey [fieldparams.BLSPubkeyLength]byte
	var sig [96]byte
	var domain [4]byte
	var p []byte
	var s []byte
	var d []byte
	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(&ba)
		fuzzer.Fuzz(&pubkey)
		fuzzer.Fuzz(&sig)
		fuzzer.Fuzz(&domain)
		fuzzer.Fuzz(&p)
		fuzzer.Fuzz(&s)
		fuzzer.Fuzz(&d)
		err := verifySignature(ba, pubkey[:], sig[:], domain[:])
		_ = err
		err = verifySignature(ba, p, s, d)
		_ = err
	}
}

func TestFuzzProcessEth1DataInBlock_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	e := &ethpb.Eth1Data{}
	state, err := state_native.InitializeFromProtoUnsafePhase0(&ethpb.BeaconState{})
	require.NoError(t, err)
	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(e)
		s, err := ProcessEth1DataInBlock(context.Background(), state, e)
		if err != nil && s != nil {
			t.Fatalf("state should be nil on err. found: %v on error: %v for state: %v and eth1data: %v", s, err, state, e)
		}
	}
}

func TestFuzzareEth1DataEqual_10000(_ *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	eth1data := &ethpb.Eth1Data{}
	eth1data2 := &ethpb.Eth1Data{}

	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(eth1data)
		fuzzer.Fuzz(eth1data2)
		AreEth1DataEqual(eth1data, eth1data2)
		AreEth1DataEqual(eth1data, eth1data)
	}
}

func TestFuzzEth1DataHasEnoughSupport_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	eth1data := &ethpb.Eth1Data{}
	var stateVotes []*ethpb.Eth1Data
	for i := 0; i < 100000; i++ {
		fuzzer.Fuzz(eth1data)
		fuzzer.Fuzz(&stateVotes)
		s, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
			Eth1DataVotes: stateVotes,
		})
		require.NoError(t, err)
		_, err = Eth1DataHasEnoughSupport(s, eth1data)
		_ = err
	}

}

func TestFuzzProcessBlockHeaderNoVerify_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	block := &ethpb.BeaconBlock{}

	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(block)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		_, err = ProcessBlockHeaderNoVerify(context.Background(), s, block.Slot, block.ProposerIndex, block.ParentRoot, []byte{})
		_ = err
	}
}

func TestFuzzProcessRandao_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	b := &ethpb.SignedBeaconBlock{}

	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(b)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		if b.Block == nil || b.Block.Body == nil {
			continue
		}
		wsb, err := blocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
		r, err := ProcessRandao(context.Background(), s, wsb)
		if err != nil && r != nil {
			t.Fatalf("return value should be nil on err. found: %v on error: %v for state: %v and block: %v", r, err, state, b)
		}
	}
}

func TestFuzzProcessRandaoNoVerify_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	blockBody := &ethpb.BeaconBlockBody{}

	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(blockBody)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		r, err := ProcessRandaoNoVerify(s, blockBody.RandaoReveal)
		if err != nil && r != nil {
			t.Fatalf("return value should be nil on err. found: %v on error: %v for state: %v and block: %v", r, err, state, blockBody)
		}
	}
}

func TestFuzzProcessProposerSlashings_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	p := &ethpb.ProposerSlashing{}
	ctx := context.Background()
	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(p)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		r, err := ProcessProposerSlashings(ctx, s, []*ethpb.ProposerSlashing{p}, v.SlashValidator)
		if err != nil && r != nil {
			t.Fatalf("return value should be nil on err. found: %v on error: %v for state: %v and slashing: %v", r, err, state, p)
		}
	}
}

func TestFuzzVerifyProposerSlashing_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	proposerSlashing := &ethpb.ProposerSlashing{}
	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(proposerSlashing)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		err = VerifyProposerSlashing(s, proposerSlashing)
		_ = err
	}
}

func TestFuzzProcessAttesterSlashings_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	a := &ethpb.AttesterSlashing{}
	ctx := context.Background()
	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(a)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		r, err := ProcessAttesterSlashings(ctx, s, []*ethpb.AttesterSlashing{a}, v.SlashValidator)
		if err != nil && r != nil {
			t.Fatalf("return value should be nil on err. found: %v on error: %v for state: %v and slashing: %v", r, err, state, a)
		}
	}
}

func TestFuzzVerifyAttesterSlashing_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	attesterSlashing := &ethpb.AttesterSlashing{}
	ctx := context.Background()
	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(attesterSlashing)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		err = VerifyAttesterSlashing(ctx, s, attesterSlashing)
		_ = err
	}
}

func TestFuzzIsSlashableAttestationData_10000(_ *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	attestationData := &ethpb.AttestationData{}
	attestationData2 := &ethpb.AttestationData{}

	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(attestationData)
		fuzzer.Fuzz(attestationData2)
		IsSlashableAttestationData(attestationData, attestationData2)
	}
}

func TestFuzzslashableAttesterIndices_10000(_ *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	attesterSlashing := &ethpb.AttesterSlashing{}

	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(attesterSlashing)
		SlashableAttesterIndices(attesterSlashing)
	}
}

func TestFuzzProcessAttestationsNoVerify_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	b := &ethpb.SignedBeaconBlock{}
	ctx := context.Background()
	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(b)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		if b.Block == nil || b.Block.Body == nil {
			continue
		}
		wsb, err := blocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
		r, err := ProcessAttestationsNoVerifySignature(ctx, s, wsb.Block())
		if err != nil && r != nil {
			t.Fatalf("return value should be nil on err. found: %v on error: %v for state: %v and block: %v", r, err, state, b)
		}
	}
}

func TestFuzzVerifyIndexedAttestationn_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	idxAttestation := &ethpb.IndexedAttestation{}
	ctx := context.Background()
	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(idxAttestation)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		err = VerifyIndexedAttestation(ctx, s, idxAttestation)
		_ = err
	}
}

func TestFuzzVerifyAttestation_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	attestation := &ethpb.Attestation{}
	ctx := context.Background()
	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(attestation)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		err = VerifyAttestationSignature(ctx, s, attestation)
		_ = err
	}
}

func TestFuzzProcessDeposits_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	deposits := make([]*ethpb.Deposit, 100)
	ctx := context.Background()
	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		for i := range deposits {
			fuzzer.Fuzz(deposits[i])
		}
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		r, err := ProcessDeposits(ctx, s, deposits)
		if err != nil && r != nil {
			t.Fatalf("return value should be nil on err. found: %v on error: %v for state: %v and block: %v", r, err, state, deposits)
		}
	}
}

func TestFuzzProcessPreGenesisDeposit_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	deposit := &ethpb.Deposit{}
	ctx := context.Background()

	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(deposit)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		r, err := ProcessPreGenesisDeposits(ctx, s, []*ethpb.Deposit{deposit})
		if err != nil && r != nil {
			t.Fatalf("return value should be nil on err. found: %v on error: %v for state: %v and block: %v", r, err, state, deposit)
		}
	}
}

func TestFuzzProcessDeposit_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	deposit := &ethpb.Deposit{}

	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(deposit)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		r, _, err := ProcessDeposit(s, deposit, true)
		if err != nil && r != nil {
			t.Fatalf("return value should be nil on err. found: %v on error: %v for state: %v and block: %v", r, err, state, deposit)
		}
	}
}

func TestFuzzverifyDeposit_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	deposit := &ethpb.Deposit{}
	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(deposit)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		err = verifyDeposit(s, deposit)
		_ = err
	}
}

func TestFuzzProcessVoluntaryExits_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	e := &ethpb.SignedVoluntaryExit{}
	ctx := context.Background()
	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(e)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		r, err := ProcessVoluntaryExits(ctx, s, []*ethpb.SignedVoluntaryExit{e})
		if err != nil && r != nil {
			t.Fatalf("return value should be nil on err. found: %v on error: %v for state: %v and exit: %v", r, err, state, e)
		}
	}
}

func TestFuzzProcessVoluntaryExitsNoVerify_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	state := &ethpb.BeaconState{}
	e := &ethpb.SignedVoluntaryExit{}
	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(state)
		fuzzer.Fuzz(e)
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)
		r, err := ProcessVoluntaryExits(context.Background(), s, []*ethpb.SignedVoluntaryExit{e})
		if err != nil && r != nil {
			t.Fatalf("return value should be nil on err. found: %v on error: %v for state: %v and block: %v", r, err, state, e)
		}
	}
}

func TestFuzzVerifyExit_10000(t *testing.T) {
	fuzzer := fuzz.NewWithSeed(0)
	ve := &ethpb.SignedVoluntaryExit{}
	rawVal := &ethpb.Validator{}
	fork := &ethpb.Fork{}
	var slot primitives.Slot

	for i := 0; i < 10000; i++ {
		fuzzer.Fuzz(ve)
		fuzzer.Fuzz(rawVal)
		fuzzer.Fuzz(fork)
		fuzzer.Fuzz(&slot)

		state := &ethpb.BeaconState{
			Slot:                  slot,
			Fork:                  fork,
			GenesisValidatorsRoot: params.BeaconConfig().ZeroHash[:],
		}
		s, err := state_native.InitializeFromProtoUnsafePhase0(state)
		require.NoError(t, err)

		val, err := state_native.NewValidator(&ethpb.Validator{})
		_ = err
		err = VerifyExitAndSignature(val, s, ve)
		_ = err
	}
}
