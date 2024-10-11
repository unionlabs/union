package client

import (
	"bytes"
	"context"
	"encoding/hex"
	"encoding/json"
	"strings"
	"testing"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	history "github.com/prysmaticlabs/prysm/v4/validator/slashing-protection-history"
)

type eip3076TestCase struct {
	Name                  string `json:"name"`
	GenesisValidatorsRoot string `json:"genesis_validators_root"`
	Steps                 []struct {
		ShouldSucceed      bool `json:"should_succeed"`
		AllowPartialImport bool `json:"allow_partial_import"`
		Interchange        struct {
			Metadata struct {
				InterchangeFormatVersion string `json:"interchange_format_version"`
				GenesisValidatorsRoot    string `json:"genesis_validators_root"`
			} `json:"metadata"`
			Data []struct {
				Pubkey       string `json:"pubkey"`
				SignedBlocks []struct {
					Slot        string `json:"slot"`
					SigningRoot string `json:"signing_root"`
				} `json:"signed_blocks"`
				SignedAttestations []struct {
					SourceEpoch string `json:"source_epoch"`
					TargetEpoch string `json:"target_epoch"`
					SigningRoot string `json:"signing_root"`
				} `json:"signed_attestations"`
			} `json:"data"`
		} `json:"interchange"`
		Blocks []struct {
			Pubkey                string `json:"pubkey"`
			Slot                  string `json:"slot"`
			SigningRoot           string `json:"signing_root"`
			ShouldSucceedComplete bool   `json:"should_succeed_complete"`
		} `json:"blocks"`
		Attestations []struct {
			Pubkey                string `json:"pubkey"`
			SourceEpoch           string `json:"source_epoch"`
			TargetEpoch           string `json:"target_epoch"`
			SigningRoot           string `json:"signing_root"`
			ShouldSucceedComplete bool   `json:"should_succeed_complete"`
		} `json:"attestations"`
	} `json:"steps"`
}

func setupEIP3076SpecTests(t *testing.T) []*eip3076TestCase {
	testFolders, err := bazel.ListRunfiles()
	require.NoError(t, err)
	testCases := make([]*eip3076TestCase, 0)
	for _, ff := range testFolders {
		if strings.Contains(ff.ShortPath, "eip3076_spec_tests") &&
			strings.Contains(ff.ShortPath, "generated/") {
			enc, err := file.ReadFileAsBytes(ff.Path)
			require.NoError(t, err)
			testCase := &eip3076TestCase{}
			require.NoError(t, json.Unmarshal(enc, testCase))
			testCases = append(testCases, testCase)
		}
	}
	return testCases
}

func TestEIP3076SpecTests(t *testing.T) {
	testCases := setupEIP3076SpecTests(t)
	for _, tt := range testCases {
		t.Run(tt.Name, func(t *testing.T) {
			if tt.Name == "" {
				t.Skip("Skipping eip3076TestCase with empty name")
			}

			// Set up validator client, one new validator client per eip3076TestCase.
			// This ensures we initialize a new (empty) slashing protection database.
			validator, _, _, _ := setup(t)

			for _, step := range tt.Steps {
				if tt.GenesisValidatorsRoot != "" {
					r, err := history.RootFromHex(tt.GenesisValidatorsRoot)
					require.NoError(t, validator.db.SaveGenesisValidatorsRoot(context.Background(), r[:]))
					require.NoError(t, err)
				}

				// The eip3076TestCase config contains the interchange config in json.
				// This loads the interchange data via ImportStandardProtectionJSON.
				interchangeBytes, err := json.Marshal(step.Interchange)
				if err != nil {
					t.Fatal(err)
				}
				b := bytes.NewBuffer(interchangeBytes)
				if err := history.ImportStandardProtectionJSON(context.Background(), validator.db, b); err != nil {
					if step.ShouldSucceed {
						t.Fatal(err)
					}
				} else if !step.ShouldSucceed {
					require.NotNil(t, err, "import standard protection json should have failed")
				}

				// This loops through a list of block signings to attempt after importing the interchange data above.
				for _, sb := range step.Blocks {
					bSlot, err := history.SlotFromString(sb.Slot)
					require.NoError(t, err)
					pk, err := history.PubKeyFromHex(sb.Pubkey)
					require.NoError(t, err)
					b := util.NewBeaconBlock()
					b.Block.Slot = bSlot

					var signingRoot [32]byte
					if sb.SigningRoot != "" {
						signingRootBytes, err := hex.DecodeString(strings.TrimPrefix(sb.SigningRoot, "0x"))
						require.NoError(t, err)
						copy(signingRoot[:], signingRootBytes)
					}

					wsb, err := blocks.NewSignedBeaconBlock(b)
					require.NoError(t, err)
					err = validator.slashableProposalCheck(context.Background(), pk, wsb, signingRoot)
					if sb.ShouldSucceedComplete {
						require.NoError(t, err)
					} else {
						require.NotEqual(t, nil, err, "pre validation should have failed for block")
					}
				}

				// This loops through a list of attestation signings to attempt after importing the interchange data above.
				for _, sa := range step.Attestations {
					target, err := history.EpochFromString(sa.TargetEpoch)
					require.NoError(t, err)
					source, err := history.EpochFromString(sa.SourceEpoch)
					require.NoError(t, err)
					pk, err := history.PubKeyFromHex(sa.Pubkey)
					require.NoError(t, err)
					ia := &ethpb.IndexedAttestation{
						Data: &ethpb.AttestationData{
							BeaconBlockRoot: make([]byte, 32),
							Target:          &ethpb.Checkpoint{Epoch: target, Root: make([]byte, 32)},
							Source:          &ethpb.Checkpoint{Epoch: source, Root: make([]byte, 32)},
						},
						Signature: make([]byte, fieldparams.BLSSignatureLength),
					}

					var signingRoot [32]byte
					if sa.SigningRoot != "" {
						signingRootBytes, err := hex.DecodeString(strings.TrimPrefix(sa.SigningRoot, "0x"))
						require.NoError(t, err)
						copy(signingRoot[:], signingRootBytes)
					}

					err = validator.slashableAttestationCheck(context.Background(), ia, pk, signingRoot)
					if sa.ShouldSucceedComplete {
						require.NoError(t, err)
					} else {
						require.NotNil(t, err, "pre validation should have failed for attestation")
					}
				}
			}

			require.NoError(t, validator.db.Close(), "failed to close slashing protection database")
		})
	}
}
