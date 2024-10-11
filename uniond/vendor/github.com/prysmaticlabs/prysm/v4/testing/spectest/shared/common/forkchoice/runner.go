package forkchoice

import (
	"context"
	"fmt"
	"os"
	"path"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/snappy"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/verification"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func init() {
	transition.SkipSlotCache.Disable()
}

// Run executes "forkchoice"  and "sync" test.
func Run(t *testing.T, config string, fork int) {
	runTest(t, config, fork, "fork_choice")
	if fork >= version.Bellatrix {
		runTest(t, config, fork, "sync")
	}
}

func runTest(t *testing.T, config string, fork int, basePath string) {
	require.NoError(t, utils.SetConfig(t, config))
	testFolders, _ := utils.TestFolders(t, config, version.String(fork), basePath)
	if len(testFolders) == 0 {
		t.Fatalf("No test folders found for %s/%s/%s", config, version.String(fork), basePath)
	}

	for _, folder := range testFolders {
		folderPath := path.Join(basePath, folder.Name(), "pyspec_tests")
		testFolders, testsFolderPath := utils.TestFolders(t, config, version.String(fork), folderPath)
		if len(testFolders) == 0 {
			t.Fatalf("No test folders found for %s/%s/%s", config, version.String(fork), folderPath)
		}

		for _, folder := range testFolders {
			t.Run(folder.Name(), func(t *testing.T) {
				preStepsFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), "steps.yaml")
				require.NoError(t, err)
				var steps []Step
				require.NoError(t, utils.UnmarshalYaml(preStepsFile, &steps))

				preBeaconStateFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), "anchor_state.ssz_snappy")
				require.NoError(t, err)
				preBeaconStateSSZ, err := snappy.Decode(nil /* dst */, preBeaconStateFile)
				require.NoError(t, err)

				blockFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), "anchor_block.ssz_snappy")
				require.NoError(t, err)
				blockSSZ, err := snappy.Decode(nil /* dst */, blockFile)
				require.NoError(t, err)

				var beaconState state.BeaconState
				var beaconBlock interfaces.ReadOnlySignedBeaconBlock
				switch fork {
				case version.Phase0:
					beaconState = unmarshalPhase0State(t, preBeaconStateSSZ)
					beaconBlock = unmarshalPhase0Block(t, blockSSZ)
				case version.Altair:
					beaconState = unmarshalAltairState(t, preBeaconStateSSZ)
					beaconBlock = unmarshalAltairBlock(t, blockSSZ)
				case version.Bellatrix:
					beaconState = unmarshalBellatrixState(t, preBeaconStateSSZ)
					beaconBlock = unmarshalBellatrixBlock(t, blockSSZ)
				case version.Capella:
					beaconState = unmarshalCapellaState(t, preBeaconStateSSZ)
					beaconBlock = unmarshalCapellaBlock(t, blockSSZ)
				case version.Deneb:
					beaconState = unmarshalDenebState(t, preBeaconStateSSZ)
					beaconBlock = unmarshalDenebBlock(t, blockSSZ)
				default:
					t.Fatalf("unknown fork version: %v", fork)
				}

				builder := NewBuilder(t, beaconState, beaconBlock)

				for _, step := range steps {
					if step.Tick != nil {
						builder.Tick(t, int64(*step.Tick))
					}
					var beaconBlock interfaces.ReadOnlySignedBeaconBlock
					if step.Block != nil {
						blockFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), fmt.Sprint(*step.Block, ".ssz_snappy"))
						require.NoError(t, err)
						blockSSZ, err := snappy.Decode(nil /* dst */, blockFile)
						require.NoError(t, err)
						switch fork {
						case version.Phase0:
							beaconBlock = unmarshalSignedPhase0Block(t, blockSSZ)
						case version.Altair:
							beaconBlock = unmarshalSignedAltairBlock(t, blockSSZ)
						case version.Bellatrix:
							beaconBlock = unmarshalSignedBellatrixBlock(t, blockSSZ)
						case version.Capella:
							beaconBlock = unmarshalSignedCapellaBlock(t, blockSSZ)
						case version.Deneb:
							beaconBlock = unmarshalSignedDenebBlock(t, blockSSZ)
						default:
							t.Fatalf("unknown fork version: %v", fork)
						}
					}
					runBlobStep(t, step.Blobs, beaconBlock, fork, folder, testsFolderPath, step.Proofs, builder)
					if beaconBlock != nil {
						if step.Valid != nil && !*step.Valid {
							builder.InvalidBlock(t, beaconBlock)
						} else {
							builder.ValidBlock(t, beaconBlock)
						}
					}
					if step.AttesterSlashing != nil {
						slashingFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), fmt.Sprint(*step.AttesterSlashing, ".ssz_snappy"))
						require.NoError(t, err)
						slashingSSZ, err := snappy.Decode(nil /* dst */, slashingFile)
						require.NoError(t, err)
						slashing := &ethpb.AttesterSlashing{}
						require.NoError(t, slashing.UnmarshalSSZ(slashingSSZ), "Failed to unmarshal")
						builder.AttesterSlashing(slashing)
					}
					if step.Attestation != nil {
						attFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), fmt.Sprint(*step.Attestation, ".ssz_snappy"))
						require.NoError(t, err)
						attSSZ, err := snappy.Decode(nil /* dst */, attFile)
						require.NoError(t, err)
						att := &ethpb.Attestation{}
						require.NoError(t, att.UnmarshalSSZ(attSSZ), "Failed to unmarshal")
						builder.Attestation(t, att)
					}
					if step.PayloadStatus != nil {
						require.NoError(t, builder.SetPayloadStatus(step.PayloadStatus))
					}
					if step.PowBlock != nil {
						powBlockFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), fmt.Sprint(*step.PowBlock, ".ssz_snappy"))
						require.NoError(t, err)
						p, err := snappy.Decode(nil /* dst */, powBlockFile)
						require.NoError(t, err)
						pb := &ethpb.PowBlock{}
						require.NoError(t, pb.UnmarshalSSZ(p), "Failed to unmarshal")
						builder.PoWBlock(pb)
					}
					builder.Check(t, step.Check)
				}
			})
		}
	}
}

func unmarshalPhase0State(t *testing.T, raw []byte) state.BeaconState {
	base := &ethpb.BeaconState{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	st, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)
	return st
}

func unmarshalPhase0Block(t *testing.T, raw []byte) interfaces.ReadOnlySignedBeaconBlock {
	base := &ethpb.BeaconBlock{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	blk, err := blocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlock{Block: base, Signature: make([]byte, fieldparams.BLSSignatureLength)})
	require.NoError(t, err)
	return blk
}

func unmarshalSignedPhase0Block(t *testing.T, raw []byte) interfaces.ReadOnlySignedBeaconBlock {
	base := &ethpb.SignedBeaconBlock{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	blk, err := blocks.NewSignedBeaconBlock(base)
	require.NoError(t, err)
	return blk
}

func unmarshalAltairState(t *testing.T, raw []byte) state.BeaconState {
	base := &ethpb.BeaconStateAltair{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	st, err := state_native.InitializeFromProtoAltair(base)
	require.NoError(t, err)
	return st
}

func unmarshalAltairBlock(t *testing.T, raw []byte) interfaces.ReadOnlySignedBeaconBlock {
	base := &ethpb.BeaconBlockAltair{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	blk, err := blocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlockAltair{Block: base, Signature: make([]byte, fieldparams.BLSSignatureLength)})
	require.NoError(t, err)
	return blk
}

func unmarshalSignedAltairBlock(t *testing.T, raw []byte) interfaces.ReadOnlySignedBeaconBlock {
	base := &ethpb.SignedBeaconBlockAltair{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	blk, err := blocks.NewSignedBeaconBlock(base)
	require.NoError(t, err)
	return blk
}

func unmarshalBellatrixState(t *testing.T, raw []byte) state.BeaconState {
	base := &ethpb.BeaconStateBellatrix{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	st, err := state_native.InitializeFromProtoBellatrix(base)
	require.NoError(t, err)
	return st
}

func unmarshalBellatrixBlock(t *testing.T, raw []byte) interfaces.ReadOnlySignedBeaconBlock {
	base := &ethpb.BeaconBlockBellatrix{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	blk, err := blocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlockBellatrix{Block: base, Signature: make([]byte, fieldparams.BLSSignatureLength)})
	require.NoError(t, err)
	return blk
}

func unmarshalSignedBellatrixBlock(t *testing.T, raw []byte) interfaces.ReadOnlySignedBeaconBlock {
	base := &ethpb.SignedBeaconBlockBellatrix{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	blk, err := blocks.NewSignedBeaconBlock(base)
	require.NoError(t, err)
	return blk
}

func unmarshalCapellaState(t *testing.T, raw []byte) state.BeaconState {
	base := &ethpb.BeaconStateCapella{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	st, err := state_native.InitializeFromProtoCapella(base)
	require.NoError(t, err)
	return st
}

func unmarshalCapellaBlock(t *testing.T, raw []byte) interfaces.ReadOnlySignedBeaconBlock {
	base := &ethpb.BeaconBlockCapella{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	blk, err := blocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlockCapella{Block: base, Signature: make([]byte, fieldparams.BLSSignatureLength)})
	require.NoError(t, err)
	return blk
}

func unmarshalSignedCapellaBlock(t *testing.T, raw []byte) interfaces.ReadOnlySignedBeaconBlock {
	base := &ethpb.SignedBeaconBlockCapella{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	blk, err := blocks.NewSignedBeaconBlock(base)
	require.NoError(t, err)
	return blk
}

func unmarshalDenebState(t *testing.T, raw []byte) state.BeaconState {
	base := &ethpb.BeaconStateDeneb{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	st, err := state_native.InitializeFromProtoDeneb(base)
	require.NoError(t, err)
	return st
}

func unmarshalDenebBlock(t *testing.T, raw []byte) interfaces.SignedBeaconBlock {
	base := &ethpb.BeaconBlockDeneb{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	blk, err := blocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlockDeneb{Block: base, Signature: make([]byte, fieldparams.BLSSignatureLength)})
	require.NoError(t, err)
	return blk
}

func unmarshalSignedDenebBlock(t *testing.T, raw []byte) interfaces.SignedBeaconBlock {
	base := &ethpb.SignedBeaconBlockDeneb{}
	require.NoError(t, base.UnmarshalSSZ(raw))
	blk, err := blocks.NewSignedBeaconBlock(base)
	require.NoError(t, err)
	return blk
}

func runBlobStep(t *testing.T,
	blobs *string,
	beaconBlock interfaces.ReadOnlySignedBeaconBlock,
	fork int,
	folder os.DirEntry,
	testsFolderPath string,
	proofs []*string,
	builder *Builder,
) {
	if blobs != nil && *blobs != "null" {
		require.NotNil(t, beaconBlock)
		require.Equal(t, true, fork >= version.Deneb)

		block := beaconBlock.Block()
		root, err := block.HashTreeRoot()
		require.NoError(t, err)
		kzgs, err := block.Body().BlobKzgCommitments()
		require.NoError(t, err)

		blobsFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), fmt.Sprint(*blobs, ".ssz_snappy"))
		require.NoError(t, err)
		blobsSSZ, err := snappy.Decode(nil /* dst */, blobsFile)
		require.NoError(t, err)
		sh, err := beaconBlock.Header()
		require.NoError(t, err)
		for index := uint64(0); index*fieldparams.BlobLength < uint64(len(blobsSSZ)); index++ {
			var proof []byte
			if index < uint64(len(proofs)) {
				proofPTR := proofs[index]
				require.NotNil(t, proofPTR)
				proof, err = hexutil.Decode(*proofPTR)
				require.NoError(t, err)
			}

			var kzg []byte
			if uint64(len(kzgs)) < index {
				kzg = kzgs[index]
			}
			if len(kzg) == 0 {
				kzg = make([]byte, 48)
			}
			blob := [fieldparams.BlobLength]byte{}
			copy(blob[:], blobsSSZ[index*fieldparams.BlobLength:])
			fakeProof := make([][]byte, fieldparams.KzgCommitmentInclusionProofDepth)
			for i := range fakeProof {
				fakeProof[i] = make([]byte, fieldparams.RootLength)
			}
			if len(proof) == 0 {
				proof = make([]byte, 48)
			}
			pb := &ethpb.BlobSidecar{
				Index:                    index,
				Blob:                     blob[:],
				KzgCommitment:            kzg,
				KzgProof:                 proof,
				SignedBlockHeader:        sh,
				CommitmentInclusionProof: fakeProof,
			}
			ro, err := blocks.NewROBlobWithRoot(pb, root)
			require.NoError(t, err)
			vsc, err := verification.BlobSidecarNoop(ro)
			require.NoError(t, err)
			require.NoError(t, builder.service.ReceiveBlob(context.Background(), vsc))
		}
	}
}
