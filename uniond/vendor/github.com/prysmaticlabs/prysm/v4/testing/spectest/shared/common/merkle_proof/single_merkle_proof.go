package merkle_proof

import (
	"encoding/hex"
	"os"
	"path"
	"testing"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/golang/snappy"
	fssz "github.com/prysmaticlabs/fastssz"
	field_params "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	consensus_blocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/shared/common/ssz_static"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

// SingleMerkleProof is the format used to read spectest Merkle Proof test data.
type SingleMerkleProof struct {
	Leaf      string   `json:"leaf"`
	LeafIndex uint64   `json:"leaf_index"`
	Branch    []string `json:"branch"`
}

func RunMerkleProofTests(t *testing.T, config, forkOrPhase string, unmarshaller ssz_static.Unmarshaller) {
	t.Skip("testvectors are not available yet")
	runSingleMerkleProofTests(t, config, forkOrPhase, unmarshaller)
}

func runSingleMerkleProofTests(t *testing.T, config, forkOrPhase string, unmarshaller ssz_static.Unmarshaller) {
	require.NoError(t, utils.SetConfig(t, config))

	testFolders, basePath := utils.TestFolders(t, config, forkOrPhase, "merkle_proof/single_merkle_proof")

	if len(testFolders) == 0 {
		t.Fatalf("No test folders found for %s/%s/merkle_proof", config, forkOrPhase)
	}

	for _, folder := range testFolders {
		typeFolderBase := path.Join(basePath, folder.Name())
		typeFolder, err := bazel.Runfile(typeFolderBase)
		require.NoError(t, err)
		modeFolders, err := os.ReadDir(typeFolder)
		require.NoError(t, err)

		if len(modeFolders) == 0 {
			t.Fatalf("No test folders found for %s", typeFolder)
		}

		for _, modeFolder := range modeFolders {
			t.Run(path.Join(folder.Name(), modeFolder.Name()), func(t *testing.T) {
				serializedBytes, err := util.BazelFileBytes(typeFolder, modeFolder.Name(), "object.ssz_snappy")
				require.NoError(t, err)
				serializedSSZ, err := snappy.Decode(nil /* dst */, serializedBytes)
				require.NoError(t, err, "Failed to decompress")
				object, err := unmarshaller(t, serializedSSZ, folder.Name())
				require.NoError(t, err, "Could not unmarshall serialized SSZ")
				sszObj, ok := object.(fssz.HashRoot)
				require.Equal(t, true, ok)
				root, err := sszObj.HashTreeRoot()
				require.NoError(t, err)

				proofYamlFile, err := util.BazelFileBytes(typeFolder, modeFolder.Name(), "proof.yaml")
				require.NoError(t, err)
				proof := &SingleMerkleProof{}
				require.NoError(t, utils.UnmarshalYaml(proofYamlFile, proof), "Failed to Unmarshal single Merkle proof")
				branch := make([][]byte, len(proof.Branch))
				for i, proofRoot := range proof.Branch {
					branch[i], err = hex.DecodeString(proofRoot[2:])
					require.NoError(t, err)
				}
				leaf, err := hex.DecodeString(proof.Leaf[2:])
				require.NoError(t, err)

				index := proof.LeafIndex
				require.Equal(t, true, trie.VerifyMerkleProof(root[:], leaf, index, branch))
				body, err := consensus_blocks.NewBeaconBlockBody(object)
				if err != nil {
					return
				}
				if index < consensus_blocks.KZGOffset || index > consensus_blocks.KZGOffset+field_params.MaxBlobsPerBlock {
					return
				}
				localProof, err := consensus_blocks.MerkleProofKZGCommitment(body, int(index-consensus_blocks.KZGOffset))
				require.NoError(t, err)
				require.Equal(t, len(branch), len(localProof))
				for i, root := range localProof {
					require.DeepEqual(t, branch[i], root)
				}
			})
		}
	}
}
