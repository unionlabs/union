package migration

import (
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpbv1 "github.com/prysmaticlabs/prysm/v4/proto/eth/v1"
	ethpbalpha "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

var (
	slot             = primitives.Slot(1)
	epoch            = primitives.Epoch(1)
	validatorIndex   = primitives.ValidatorIndex(1)
	committeeIndex   = primitives.CommitteeIndex(1)
	depositCount     = uint64(2)
	attestingIndices = []uint64{1, 2}
	blockNumber      = uint64(10)
	gasLimit         = uint64(10)
	gasUsed          = uint64(10)
	timestamp        = uint64(10)
	parentRoot       = bytesutil.PadTo([]byte("parentroot"), fieldparams.RootLength)
	stateRoot        = bytesutil.PadTo([]byte("stateroot"), fieldparams.RootLength)
	signature        = bytesutil.PadTo([]byte("signature"), 96)
	randaoReveal     = bytesutil.PadTo([]byte("randaoreveal"), 96)
	depositRoot      = bytesutil.PadTo([]byte("depositroot"), fieldparams.RootLength)
	blockHash        = bytesutil.PadTo([]byte("blockhash"), 32)
	beaconBlockRoot  = bytesutil.PadTo([]byte("beaconblockroot"), fieldparams.RootLength)
	sourceRoot       = bytesutil.PadTo([]byte("sourceroot"), fieldparams.RootLength)
	targetRoot       = bytesutil.PadTo([]byte("targetroot"), fieldparams.RootLength)
	bodyRoot         = bytesutil.PadTo([]byte("bodyroot"), fieldparams.RootLength)
	selectionProof   = bytesutil.PadTo([]byte("selectionproof"), 96)
	parentHash       = bytesutil.PadTo([]byte("parenthash"), 32)
	feeRecipient     = bytesutil.PadTo([]byte("feerecipient"), 20)
	receiptsRoot     = bytesutil.PadTo([]byte("receiptsroot"), 32)
	logsBloom        = bytesutil.PadTo([]byte("logsbloom"), 256)
	prevRandao       = bytesutil.PadTo([]byte("prevrandao"), 32)
	extraData        = bytesutil.PadTo([]byte("extradata"), 32)
	baseFeePerGas    = bytesutil.PadTo([]byte("basefeepergas"), 32)
	transactionsRoot = bytesutil.PadTo([]byte("transactions"), 32)
	aggregationBits  = bitfield.Bitlist{0x01}
)

func Test_BlockIfaceToV1BlockHeader(t *testing.T) {
	alphaBlock := util.HydrateSignedBeaconBlock(&ethpbalpha.SignedBeaconBlock{})
	alphaBlock.Block.Slot = slot
	alphaBlock.Block.ProposerIndex = validatorIndex
	alphaBlock.Block.ParentRoot = parentRoot
	alphaBlock.Block.StateRoot = stateRoot
	alphaBlock.Signature = signature

	wsb, err := blocks.NewSignedBeaconBlock(alphaBlock)
	require.NoError(t, err)
	v1Header, err := BlockIfaceToV1BlockHeader(wsb)
	require.NoError(t, err)
	bodyRoot, err := alphaBlock.Block.Body.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, bodyRoot[:], v1Header.Message.BodyRoot)
	assert.Equal(t, slot, v1Header.Message.Slot)
	assert.Equal(t, validatorIndex, v1Header.Message.ProposerIndex)
	assert.DeepEqual(t, parentRoot, v1Header.Message.ParentRoot)
	assert.DeepEqual(t, stateRoot, v1Header.Message.StateRoot)
	assert.DeepEqual(t, signature, v1Header.Signature)
}

func Test_V1Alpha1AggregateAttAndProofToV1(t *testing.T) {
	proof := [32]byte{1}
	att := util.HydrateAttestation(&ethpbalpha.Attestation{
		Data: &ethpbalpha.AttestationData{
			Slot: 5,
		},
	})
	alpha := &ethpbalpha.AggregateAttestationAndProof{
		AggregatorIndex: 1,
		Aggregate:       att,
		SelectionProof:  proof[:],
	}
	v1 := V1Alpha1AggregateAttAndProofToV1(alpha)
	assert.Equal(t, v1.AggregatorIndex, primitives.ValidatorIndex(1))
	assert.DeepSSZEqual(t, v1.Aggregate.Data.Slot, att.Data.Slot)
	assert.DeepEqual(t, v1.SelectionProof, proof[:])
}

func Test_V1Alpha1ToV1SignedBlock(t *testing.T) {
	alphaBlock := util.HydrateSignedBeaconBlock(&ethpbalpha.SignedBeaconBlock{})
	alphaBlock.Block.Slot = slot
	alphaBlock.Block.ProposerIndex = validatorIndex
	alphaBlock.Block.ParentRoot = parentRoot
	alphaBlock.Block.StateRoot = stateRoot
	alphaBlock.Block.Body.RandaoReveal = randaoReveal
	alphaBlock.Block.Body.Eth1Data = &ethpbalpha.Eth1Data{
		DepositRoot:  depositRoot,
		DepositCount: depositCount,
		BlockHash:    blockHash,
	}
	alphaBlock.Signature = signature

	v1Block, err := V1Alpha1ToV1SignedBlock(alphaBlock)
	require.NoError(t, err)
	alphaRoot, err := alphaBlock.HashTreeRoot()
	require.NoError(t, err)
	v1Root, err := v1Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, alphaRoot, v1Root)
}

func Test_V1ToV1Alpha1SignedBlock(t *testing.T) {
	v1Block := util.HydrateV1SignedBeaconBlock(&ethpbv1.SignedBeaconBlock{})
	v1Block.Block.Slot = slot
	v1Block.Block.ProposerIndex = validatorIndex
	v1Block.Block.ParentRoot = parentRoot
	v1Block.Block.StateRoot = stateRoot
	v1Block.Block.Body.RandaoReveal = randaoReveal
	v1Block.Block.Body.Eth1Data = &ethpbv1.Eth1Data{
		DepositRoot:  depositRoot,
		DepositCount: depositCount,
		BlockHash:    blockHash,
	}
	v1Block.Signature = signature

	alphaBlock, err := V1ToV1Alpha1SignedBlock(v1Block)
	require.NoError(t, err)
	alphaRoot, err := alphaBlock.HashTreeRoot()
	require.NoError(t, err)
	v1Root, err := v1Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, v1Root, alphaRoot)
}

func Test_V1ToV1Alpha1Block(t *testing.T) {
	alphaBlock := util.HydrateBeaconBlock(&ethpbalpha.BeaconBlock{})
	alphaBlock.Slot = slot
	alphaBlock.ProposerIndex = validatorIndex
	alphaBlock.ParentRoot = parentRoot
	alphaBlock.StateRoot = stateRoot
	alphaBlock.Body.RandaoReveal = randaoReveal
	alphaBlock.Body.Eth1Data = &ethpbalpha.Eth1Data{
		DepositRoot:  depositRoot,
		DepositCount: depositCount,
		BlockHash:    blockHash,
	}

	v1Block, err := V1Alpha1ToV1Block(alphaBlock)
	require.NoError(t, err)
	v1Root, err := v1Block.HashTreeRoot()
	require.NoError(t, err)
	alphaRoot, err := alphaBlock.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, alphaRoot, v1Root)
}

func Test_V1Alpha1AttSlashingToV1(t *testing.T) {
	alphaAttestation := &ethpbalpha.IndexedAttestation{
		AttestingIndices: attestingIndices,
		Data: &ethpbalpha.AttestationData{
			Slot:            slot,
			CommitteeIndex:  committeeIndex,
			BeaconBlockRoot: beaconBlockRoot,
			Source: &ethpbalpha.Checkpoint{
				Epoch: epoch,
				Root:  sourceRoot,
			},
			Target: &ethpbalpha.Checkpoint{
				Epoch: epoch,
				Root:  targetRoot,
			},
		},
		Signature: signature,
	}
	alphaSlashing := &ethpbalpha.AttesterSlashing{
		Attestation_1: alphaAttestation,
		Attestation_2: alphaAttestation,
	}

	v1Slashing := V1Alpha1AttSlashingToV1(alphaSlashing)
	alphaRoot, err := alphaSlashing.HashTreeRoot()
	require.NoError(t, err)
	v1Root, err := v1Slashing.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, alphaRoot, v1Root)
}

func Test_V1Alpha1ProposerSlashingToV1(t *testing.T) {
	alphaHeader := util.HydrateSignedBeaconHeader(&ethpbalpha.SignedBeaconBlockHeader{})
	alphaHeader.Header.Slot = slot
	alphaHeader.Header.ProposerIndex = validatorIndex
	alphaHeader.Header.ParentRoot = parentRoot
	alphaHeader.Header.StateRoot = stateRoot
	alphaHeader.Header.BodyRoot = bodyRoot
	alphaHeader.Signature = signature
	alphaSlashing := &ethpbalpha.ProposerSlashing{
		Header_1: alphaHeader,
		Header_2: alphaHeader,
	}

	v1Slashing := V1Alpha1ProposerSlashingToV1(alphaSlashing)
	alphaRoot, err := alphaSlashing.HashTreeRoot()
	require.NoError(t, err)
	v1Root, err := v1Slashing.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, alphaRoot, v1Root)
}

func Test_V1Alpha1ExitToV1(t *testing.T) {
	alphaExit := &ethpbalpha.SignedVoluntaryExit{
		Exit: &ethpbalpha.VoluntaryExit{
			Epoch:          epoch,
			ValidatorIndex: validatorIndex,
		},
		Signature: signature,
	}

	v1Exit := V1Alpha1ExitToV1(alphaExit)
	alphaRoot, err := alphaExit.HashTreeRoot()
	require.NoError(t, err)
	v1Root, err := v1Exit.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, alphaRoot, v1Root)
}

func Test_V1ExitToV1Alpha1(t *testing.T) {
	v1Exit := &ethpbv1.SignedVoluntaryExit{
		Message: &ethpbv1.VoluntaryExit{
			Epoch:          epoch,
			ValidatorIndex: validatorIndex,
		},
		Signature: signature,
	}

	alphaExit := V1ExitToV1Alpha1(v1Exit)
	alphaRoot, err := alphaExit.HashTreeRoot()
	require.NoError(t, err)
	v1Root, err := v1Exit.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, alphaRoot, v1Root)
}

func Test_V1AttSlashingToV1Alpha1(t *testing.T) {
	v1Attestation := &ethpbv1.IndexedAttestation{
		AttestingIndices: attestingIndices,
		Data: &ethpbv1.AttestationData{
			Slot:            slot,
			Index:           committeeIndex,
			BeaconBlockRoot: beaconBlockRoot,
			Source: &ethpbv1.Checkpoint{
				Epoch: epoch,
				Root:  sourceRoot,
			},
			Target: &ethpbv1.Checkpoint{
				Epoch: epoch,
				Root:  targetRoot,
			},
		},
		Signature: signature,
	}
	v1Slashing := &ethpbv1.AttesterSlashing{
		Attestation_1: v1Attestation,
		Attestation_2: v1Attestation,
	}

	alphaSlashing := V1AttSlashingToV1Alpha1(v1Slashing)
	alphaRoot, err := alphaSlashing.HashTreeRoot()
	require.NoError(t, err)
	v1Root, err := v1Slashing.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, v1Root, alphaRoot)
}

func Test_V1ProposerSlashingToV1Alpha1(t *testing.T) {
	v1Header := &ethpbv1.SignedBeaconBlockHeader{
		Message: &ethpbv1.BeaconBlockHeader{
			Slot:          slot,
			ProposerIndex: validatorIndex,
			ParentRoot:    parentRoot,
			StateRoot:     stateRoot,
			BodyRoot:      bodyRoot,
		},
		Signature: signature,
	}
	v1Slashing := &ethpbv1.ProposerSlashing{
		SignedHeader_1: v1Header,
		SignedHeader_2: v1Header,
	}

	alphaSlashing := V1ProposerSlashingToV1Alpha1(v1Slashing)
	alphaRoot, err := alphaSlashing.HashTreeRoot()
	require.NoError(t, err)
	v1Root, err := v1Slashing.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, alphaRoot, v1Root)
}

func Test_V1Alpha1AttToV1(t *testing.T) {
	alphaAtt := &ethpbalpha.Attestation{
		AggregationBits: aggregationBits,
		Data: &ethpbalpha.AttestationData{
			Slot:            slot,
			CommitteeIndex:  committeeIndex,
			BeaconBlockRoot: beaconBlockRoot,
			Source: &ethpbalpha.Checkpoint{
				Epoch: epoch,
				Root:  sourceRoot,
			},
			Target: &ethpbalpha.Checkpoint{
				Epoch: epoch,
				Root:  targetRoot,
			},
		},
		Signature: signature,
	}

	v1Att := V1Alpha1AttestationToV1(alphaAtt)
	v1Root, err := v1Att.HashTreeRoot()
	require.NoError(t, err)
	alphaRoot, err := alphaAtt.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, v1Root, alphaRoot)
}

func Test_V1AttToV1Alpha1(t *testing.T) {
	v1Att := &ethpbv1.Attestation{
		AggregationBits: aggregationBits,
		Data: &ethpbv1.AttestationData{
			Slot:            slot,
			Index:           committeeIndex,
			BeaconBlockRoot: beaconBlockRoot,
			Source: &ethpbv1.Checkpoint{
				Epoch: epoch,
				Root:  sourceRoot,
			},
			Target: &ethpbv1.Checkpoint{
				Epoch: epoch,
				Root:  targetRoot,
			},
		},
		Signature: signature,
	}

	alphaAtt := V1AttToV1Alpha1(v1Att)
	alphaRoot, err := alphaAtt.HashTreeRoot()
	require.NoError(t, err)
	v1Root, err := v1Att.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, v1Root, alphaRoot)
}

func Test_BlockInterfaceToV1Block(t *testing.T) {
	v1Alpha1Block := util.HydrateSignedBeaconBlock(&ethpbalpha.SignedBeaconBlock{})
	v1Alpha1Block.Block.Slot = slot
	v1Alpha1Block.Block.ProposerIndex = validatorIndex
	v1Alpha1Block.Block.ParentRoot = parentRoot
	v1Alpha1Block.Block.StateRoot = stateRoot
	v1Alpha1Block.Block.Body.RandaoReveal = randaoReveal
	v1Alpha1Block.Block.Body.Eth1Data = &ethpbalpha.Eth1Data{
		DepositRoot:  depositRoot,
		DepositCount: depositCount,
		BlockHash:    blockHash,
	}
	v1Alpha1Block.Signature = signature

	wsb, err := blocks.NewSignedBeaconBlock(v1Alpha1Block)
	require.NoError(t, err)
	v1Block, err := SignedBeaconBlock(wsb)
	require.NoError(t, err)
	v1Root, err := v1Block.HashTreeRoot()
	require.NoError(t, err)
	v1Alpha1Root, err := v1Alpha1Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, v1Root, v1Alpha1Root)
}

func Test_V1Alpha1ValidatorToV1(t *testing.T) {
	v1Alpha1Validator := &ethpbalpha.Validator{
		PublicKey:                  []byte("pubkey"),
		WithdrawalCredentials:      []byte("withdraw"),
		EffectiveBalance:           99,
		Slashed:                    true,
		ActivationEligibilityEpoch: 1,
		ActivationEpoch:            11,
		ExitEpoch:                  111,
		WithdrawableEpoch:          1111,
	}

	v1Validator := V1Alpha1ValidatorToV1(v1Alpha1Validator)
	require.NotNil(t, v1Validator)
	assert.DeepEqual(t, []byte("pubkey"), v1Validator.Pubkey)
	assert.DeepEqual(t, []byte("withdraw"), v1Validator.WithdrawalCredentials)
	assert.Equal(t, uint64(99), v1Validator.EffectiveBalance)
	assert.Equal(t, true, v1Validator.Slashed)
	assert.Equal(t, primitives.Epoch(1), v1Validator.ActivationEligibilityEpoch)
	assert.Equal(t, primitives.Epoch(11), v1Validator.ActivationEpoch)
	assert.Equal(t, primitives.Epoch(111), v1Validator.ExitEpoch)
	assert.Equal(t, primitives.Epoch(1111), v1Validator.WithdrawableEpoch)
}

func Test_V1ValidatorToV1Alpha1(t *testing.T) {
	v1Validator := &ethpbv1.Validator{
		Pubkey:                     []byte("pubkey"),
		WithdrawalCredentials:      []byte("withdraw"),
		EffectiveBalance:           99,
		Slashed:                    true,
		ActivationEligibilityEpoch: 1,
		ActivationEpoch:            11,
		ExitEpoch:                  111,
		WithdrawableEpoch:          1111,
	}

	v1Alpha1Validator := V1ValidatorToV1Alpha1(v1Validator)
	require.NotNil(t, v1Alpha1Validator)
	assert.DeepEqual(t, []byte("pubkey"), v1Alpha1Validator.PublicKey)
	assert.DeepEqual(t, []byte("withdraw"), v1Alpha1Validator.WithdrawalCredentials)
	assert.Equal(t, uint64(99), v1Alpha1Validator.EffectiveBalance)
	assert.Equal(t, true, v1Alpha1Validator.Slashed)
	assert.Equal(t, primitives.Epoch(1), v1Alpha1Validator.ActivationEligibilityEpoch)
	assert.Equal(t, primitives.Epoch(11), v1Alpha1Validator.ActivationEpoch)
	assert.Equal(t, primitives.Epoch(111), v1Alpha1Validator.ExitEpoch)
	assert.Equal(t, primitives.Epoch(1111), v1Alpha1Validator.WithdrawableEpoch)
}

func Test_V1SignedAggregateAttAndProofToV1Alpha1(t *testing.T) {
	v1Att := &ethpbv1.SignedAggregateAttestationAndProof{
		Message: &ethpbv1.AggregateAttestationAndProof{
			AggregatorIndex: 1,
			Aggregate:       util.HydrateV1Attestation(&ethpbv1.Attestation{}),
			SelectionProof:  selectionProof,
		},
		Signature: signature,
	}
	v1Alpha1Att := V1SignedAggregateAttAndProofToV1Alpha1(v1Att)

	v1Root, err := v1Att.HashTreeRoot()
	require.NoError(t, err)
	v1Alpha1Root, err := v1Alpha1Att.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, v1Root, v1Alpha1Root)
}

func Test_V1AttestationToV1Alpha1(t *testing.T) {
	v1Att := util.HydrateV1Attestation(&ethpbv1.Attestation{})
	v1Alpha1Att := V1AttToV1Alpha1(v1Att)

	v1Root, err := v1Att.HashTreeRoot()
	require.NoError(t, err)
	v1Alpha1Root, err := v1Alpha1Att.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, v1Root, v1Alpha1Root)
}
func TestBeaconStateToProto(t *testing.T) {
	source, err := util.NewBeaconState(util.FillRootsNaturalOpt, func(state *ethpbalpha.BeaconState) error {
		state.GenesisTime = 1
		state.GenesisValidatorsRoot = bytesutil.PadTo([]byte("genesisvalidatorsroot"), 32)
		state.Slot = 2
		state.Fork = &ethpbalpha.Fork{
			PreviousVersion: bytesutil.PadTo([]byte("123"), 4),
			CurrentVersion:  bytesutil.PadTo([]byte("456"), 4),
			Epoch:           3,
		}
		state.LatestBlockHeader = &ethpbalpha.BeaconBlockHeader{
			Slot:          4,
			ProposerIndex: 5,
			ParentRoot:    bytesutil.PadTo([]byte("lbhparentroot"), 32),
			StateRoot:     bytesutil.PadTo([]byte("lbhstateroot"), 32),
			BodyRoot:      bytesutil.PadTo([]byte("lbhbodyroot"), 32),
		}
		state.BlockRoots = [][]byte{bytesutil.PadTo([]byte("blockroots"), 32)}
		state.StateRoots = [][]byte{bytesutil.PadTo([]byte("stateroots"), 32)}
		state.HistoricalRoots = [][]byte{bytesutil.PadTo([]byte("historicalroots"), 32)}
		state.Eth1Data = &ethpbalpha.Eth1Data{
			DepositRoot:  bytesutil.PadTo([]byte("e1ddepositroot"), 32),
			DepositCount: 6,
			BlockHash:    bytesutil.PadTo([]byte("e1dblockhash"), 32),
		}
		state.Eth1DataVotes = []*ethpbalpha.Eth1Data{{
			DepositRoot:  bytesutil.PadTo([]byte("e1dvdepositroot"), 32),
			DepositCount: 7,
			BlockHash:    bytesutil.PadTo([]byte("e1dvblockhash"), 32),
		}}
		state.Eth1DepositIndex = 8
		state.Validators = []*ethpbalpha.Validator{{
			PublicKey:                  bytesutil.PadTo([]byte("publickey"), 48),
			WithdrawalCredentials:      bytesutil.PadTo([]byte("withdrawalcredentials"), 32),
			EffectiveBalance:           9,
			Slashed:                    true,
			ActivationEligibilityEpoch: 10,
			ActivationEpoch:            11,
			ExitEpoch:                  12,
			WithdrawableEpoch:          13,
		}}
		state.Balances = []uint64{14}
		state.RandaoMixes = [][]byte{bytesutil.PadTo([]byte("randaomixes"), 32)}
		state.Slashings = []uint64{15}
		state.PreviousEpochAttestations = []*ethpbalpha.PendingAttestation{{
			AggregationBits: bitfield.Bitlist{16},
			Data: &ethpbalpha.AttestationData{
				Slot:            17,
				CommitteeIndex:  18,
				BeaconBlockRoot: bytesutil.PadTo([]byte("peabeaconblockroot"), 32),
				Source: &ethpbalpha.Checkpoint{
					Epoch: 19,
					Root:  bytesutil.PadTo([]byte("peasroot"), 32),
				},
				Target: &ethpbalpha.Checkpoint{
					Epoch: 20,
					Root:  bytesutil.PadTo([]byte("peatroot"), 32),
				},
			},
			InclusionDelay: 21,
			ProposerIndex:  22,
		}}
		state.CurrentEpochAttestations = []*ethpbalpha.PendingAttestation{{
			AggregationBits: bitfield.Bitlist{23},
			Data: &ethpbalpha.AttestationData{
				Slot:            24,
				CommitteeIndex:  25,
				BeaconBlockRoot: bytesutil.PadTo([]byte("ceabeaconblockroot"), 32),
				Source: &ethpbalpha.Checkpoint{
					Epoch: 26,
					Root:  bytesutil.PadTo([]byte("ceasroot"), 32),
				},
				Target: &ethpbalpha.Checkpoint{
					Epoch: 27,
					Root:  bytesutil.PadTo([]byte("ceatroot"), 32),
				},
			},
			InclusionDelay: 28,
			ProposerIndex:  29,
		}}
		state.JustificationBits = bitfield.Bitvector4{1}
		state.PreviousJustifiedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 30,
			Root:  bytesutil.PadTo([]byte("pjcroot"), 32),
		}
		state.CurrentJustifiedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 31,
			Root:  bytesutil.PadTo([]byte("cjcroot"), 32),
		}
		state.FinalizedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 32,
			Root:  bytesutil.PadTo([]byte("fcroot"), 32),
		}
		return nil
	})
	require.NoError(t, err)

	result, err := BeaconStateToProto(source)
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, uint64(1), result.GenesisTime)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("genesisvalidatorsroot"), 32), result.GenesisValidatorsRoot)
	assert.Equal(t, primitives.Slot(2), result.Slot)
	resultFork := result.Fork
	require.NotNil(t, resultFork)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("123"), 4), resultFork.PreviousVersion)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("456"), 4), resultFork.CurrentVersion)
	assert.Equal(t, primitives.Epoch(3), resultFork.Epoch)
	resultLatestBlockHeader := result.LatestBlockHeader
	require.NotNil(t, resultLatestBlockHeader)
	assert.Equal(t, primitives.Slot(4), resultLatestBlockHeader.Slot)
	assert.Equal(t, primitives.ValidatorIndex(5), resultLatestBlockHeader.ProposerIndex)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhparentroot"), 32), resultLatestBlockHeader.ParentRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhstateroot"), 32), resultLatestBlockHeader.StateRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhbodyroot"), 32), resultLatestBlockHeader.BodyRoot)
	assert.Equal(t, 8192, len(result.BlockRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("blockroots"), 32), result.BlockRoots[0])
	assert.Equal(t, 8192, len(result.StateRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("stateroots"), 32), result.StateRoots[0])
	assert.Equal(t, 1, len(result.HistoricalRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("historicalroots"), 32), result.HistoricalRoots[0])
	resultEth1Data := result.Eth1Data
	require.NotNil(t, resultEth1Data)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1ddepositroot"), 32), resultEth1Data.DepositRoot)
	assert.Equal(t, uint64(6), resultEth1Data.DepositCount)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dblockhash"), 32), resultEth1Data.BlockHash)
	require.Equal(t, 1, len(result.Eth1DataVotes))
	resultEth1DataVote := result.Eth1DataVotes[0]
	require.NotNil(t, resultEth1DataVote)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dvdepositroot"), 32), resultEth1DataVote.DepositRoot)
	assert.Equal(t, uint64(7), resultEth1DataVote.DepositCount)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dvblockhash"), 32), resultEth1DataVote.BlockHash)
	assert.Equal(t, uint64(8), result.Eth1DepositIndex)
	require.Equal(t, 1, len(result.Validators))
	resultValidator := result.Validators[0]
	require.NotNil(t, resultValidator)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("publickey"), 48), resultValidator.Pubkey)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("withdrawalcredentials"), 32), resultValidator.WithdrawalCredentials)
	assert.Equal(t, uint64(9), resultValidator.EffectiveBalance)
	assert.Equal(t, true, resultValidator.Slashed)
	assert.Equal(t, primitives.Epoch(10), resultValidator.ActivationEligibilityEpoch)
	assert.Equal(t, primitives.Epoch(11), resultValidator.ActivationEpoch)
	assert.Equal(t, primitives.Epoch(12), resultValidator.ExitEpoch)
	assert.Equal(t, primitives.Epoch(13), resultValidator.WithdrawableEpoch)
	assert.DeepEqual(t, []uint64{14}, result.Balances)
	assert.Equal(t, 65536, len(result.RandaoMixes))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("randaomixes"), 32), result.RandaoMixes[0])
	assert.DeepEqual(t, []uint64{15}, result.Slashings)
	require.Equal(t, 1, len(result.PreviousEpochAttestations))
	resultPrevEpochAtt := result.PreviousEpochAttestations[0]
	require.NotNil(t, resultPrevEpochAtt)
	assert.DeepEqual(t, bitfield.Bitlist{16}, resultPrevEpochAtt.AggregationBits)
	resultPrevEpochAttData := resultPrevEpochAtt.Data
	require.NotNil(t, resultPrevEpochAttData)
	assert.Equal(t, primitives.Slot(17), resultPrevEpochAttData.Slot)
	assert.Equal(t, primitives.CommitteeIndex(18), resultPrevEpochAttData.Index)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("peabeaconblockroot"), 32), resultPrevEpochAttData.BeaconBlockRoot)
	resultPrevEpochAttSource := resultPrevEpochAttData.Source
	require.NotNil(t, resultPrevEpochAttSource)
	assert.Equal(t, primitives.Epoch(19), resultPrevEpochAttSource.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("peasroot"), 32), resultPrevEpochAttSource.Root)
	resultPrevEpochAttTarget := resultPrevEpochAttData.Target
	require.NotNil(t, resultPrevEpochAttTarget)
	assert.Equal(t, primitives.Epoch(20), resultPrevEpochAttTarget.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("peatroot"), 32), resultPrevEpochAttTarget.Root)
	assert.Equal(t, primitives.Slot(21), resultPrevEpochAtt.InclusionDelay)
	assert.Equal(t, primitives.ValidatorIndex(22), resultPrevEpochAtt.ProposerIndex)
	resultCurrEpochAtt := result.CurrentEpochAttestations[0]
	require.NotNil(t, resultCurrEpochAtt)
	assert.DeepEqual(t, bitfield.Bitlist{23}, resultCurrEpochAtt.AggregationBits)
	resultCurrEpochAttData := resultCurrEpochAtt.Data
	require.NotNil(t, resultCurrEpochAttData)
	assert.Equal(t, primitives.Slot(24), resultCurrEpochAttData.Slot)
	assert.Equal(t, primitives.CommitteeIndex(25), resultCurrEpochAttData.Index)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("ceabeaconblockroot"), 32), resultCurrEpochAttData.BeaconBlockRoot)
	resultCurrEpochAttSource := resultCurrEpochAttData.Source
	require.NotNil(t, resultCurrEpochAttSource)
	assert.Equal(t, primitives.Epoch(26), resultCurrEpochAttSource.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("ceasroot"), 32), resultCurrEpochAttSource.Root)
	resultCurrEpochAttTarget := resultCurrEpochAttData.Target
	require.NotNil(t, resultCurrEpochAttTarget)
	assert.Equal(t, primitives.Epoch(27), resultCurrEpochAttTarget.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("ceatroot"), 32), resultCurrEpochAttTarget.Root)
	assert.Equal(t, primitives.Slot(28), resultCurrEpochAtt.InclusionDelay)
	assert.Equal(t, primitives.ValidatorIndex(29), resultCurrEpochAtt.ProposerIndex)
	assert.DeepEqual(t, bitfield.Bitvector4{1}, result.JustificationBits)
	resultPrevJustifiedCheckpoint := result.PreviousJustifiedCheckpoint
	require.NotNil(t, resultPrevJustifiedCheckpoint)
	assert.Equal(t, primitives.Epoch(30), resultPrevJustifiedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("pjcroot"), 32), resultPrevJustifiedCheckpoint.Root)
	resultCurrJustifiedCheckpoint := result.CurrentJustifiedCheckpoint
	require.NotNil(t, resultCurrJustifiedCheckpoint)
	assert.Equal(t, primitives.Epoch(31), resultCurrJustifiedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("cjcroot"), 32), resultCurrJustifiedCheckpoint.Root)
	resultFinalizedCheckpoint := result.FinalizedCheckpoint
	require.NotNil(t, resultFinalizedCheckpoint)
	assert.Equal(t, primitives.Epoch(32), resultFinalizedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("fcroot"), 32), resultFinalizedCheckpoint.Root)
}
