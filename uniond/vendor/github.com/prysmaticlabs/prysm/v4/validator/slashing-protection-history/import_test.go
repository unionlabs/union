package history

import (
	"bytes"
	"context"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"reflect"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/db/kv"
	dbtest "github.com/prysmaticlabs/prysm/v4/validator/db/testing"
	"github.com/prysmaticlabs/prysm/v4/validator/slashing-protection-history/format"
	valtest "github.com/prysmaticlabs/prysm/v4/validator/testing"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestStore_ImportInterchangeData_BadJSON(t *testing.T) {
	ctx := context.Background()
	validatorDB := dbtest.SetupDB(t, nil)

	buf := bytes.NewBuffer([]byte("helloworld"))
	err := ImportStandardProtectionJSON(ctx, validatorDB, buf)
	require.ErrorContains(t, "could not unmarshal slashing protection JSON file", err)
}

func TestStore_ImportInterchangeData_NilData_FailsSilently(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	validatorDB := dbtest.SetupDB(t, nil)

	interchangeJSON := &format.EIPSlashingProtectionFormat{}
	encoded, err := json.Marshal(interchangeJSON)
	require.NoError(t, err)

	buf := bytes.NewBuffer(encoded)
	err = ImportStandardProtectionJSON(ctx, validatorDB, buf)
	require.NoError(t, err)
	require.LogsContain(t, hook, "No slashing protection data to import")
}

func TestStore_ImportInterchangeData_BadFormat_PreventsDBWrites(t *testing.T) {
	ctx := context.Background()
	numValidators := 10
	publicKeys, err := valtest.CreateRandomPubKeys(numValidators)
	require.NoError(t, err)
	validatorDB := dbtest.SetupDB(t, publicKeys)

	// First we setup some mock attesting and proposal histories and create a mock
	// standard slashing protection format JSON struct.
	attestingHistory, proposalHistory := valtest.MockAttestingAndProposalHistories(publicKeys)
	standardProtectionFormat, err := valtest.MockSlashingProtectionJSON(publicKeys, attestingHistory, proposalHistory)
	require.NoError(t, err)

	// We replace a slot of one of the blocks with junk data.
	standardProtectionFormat.Data[0].SignedBlocks[0].Slot = "BadSlot"

	// We encode the standard slashing protection struct into a JSON format.
	blob, err := json.Marshal(standardProtectionFormat)
	require.NoError(t, err)
	buf := bytes.NewBuffer(blob)

	// Next, we attempt to import it into our validator database and check that
	// we obtain an error during the import process.
	err = ImportStandardProtectionJSON(ctx, validatorDB, buf)
	assert.NotNil(t, err)

	// Next, we attempt to retrieve the attesting and proposals histories from our database and
	// verify nothing was saved to the DB. If there is an error in the import process, we need to make
	// sure writing is an atomic operation: either the import succeeds and saves the slashing protection
	// data to our DB, or it does not.
	for i := 0; i < len(publicKeys); i++ {
		for _, att := range attestingHistory[i] {
			indexedAtt := &ethpb.IndexedAttestation{
				Data: &ethpb.AttestationData{
					Source: &ethpb.Checkpoint{
						Epoch: att.Source,
					},
					Target: &ethpb.Checkpoint{
						Epoch: att.Target,
					},
				},
			}
			slashingKind, err := validatorDB.CheckSlashableAttestation(ctx, publicKeys[i], []byte{}, indexedAtt)
			// We expect we do not have an attesting history for each attestation
			require.NoError(t, err)
			require.Equal(t, kv.NotSlashable, slashingKind)
		}
		receivedHistory, err := validatorDB.ProposalHistoryForPubKey(ctx, publicKeys[i])
		require.NoError(t, err)
		require.DeepEqual(
			t,
			make([]*kv.Proposal, 0),
			receivedHistory,
			"Imported proposal signing root is different than the empty default",
		)
	}
}

func TestStore_ImportInterchangeData_OK(t *testing.T) {
	ctx := context.Background()
	numValidators := 10
	publicKeys, err := valtest.CreateRandomPubKeys(numValidators)
	require.NoError(t, err)
	validatorDB := dbtest.SetupDB(t, publicKeys)

	// First we setup some mock attesting and proposal histories and create a mock
	// standard slashing protection format JSON struct.
	attestingHistory, proposalHistory := valtest.MockAttestingAndProposalHistories(publicKeys)
	standardProtectionFormat, err := valtest.MockSlashingProtectionJSON(publicKeys, attestingHistory, proposalHistory)
	require.NoError(t, err)

	// We encode the standard slashing protection struct into a JSON format.
	blob, err := json.Marshal(standardProtectionFormat)
	require.NoError(t, err)
	buf := bytes.NewBuffer(blob)

	// Next, we attempt to import it into our validator database.
	err = ImportStandardProtectionJSON(ctx, validatorDB, buf)
	require.NoError(t, err)

	// Next, we attempt to retrieve the attesting and proposals histories from our database and
	// verify those indeed match the originally generated mock histories.
	for i := 0; i < len(publicKeys); i++ {
		for _, att := range attestingHistory[i] {
			indexedAtt := &ethpb.IndexedAttestation{
				Data: &ethpb.AttestationData{
					Source: &ethpb.Checkpoint{
						Epoch: att.Source,
					},
					Target: &ethpb.Checkpoint{
						Epoch: att.Target,
					},
				},
			}
			slashingKind, err := validatorDB.CheckSlashableAttestation(ctx, publicKeys[i], []byte{}, indexedAtt)
			// We expect we have an attesting history for the attestation and when
			// attempting to verify the same att is slashable with a different signing root,
			// we expect to receive a double vote slashing kind.
			require.NotNil(t, err)
			require.Equal(t, kv.DoubleVote, slashingKind)
		}

		proposals := proposalHistory[i].Proposals

		receivedProposalHistory, err := validatorDB.ProposalHistoryForPubKey(ctx, publicKeys[i])
		require.NoError(t, err)
		rootsBySlot := make(map[primitives.Slot][]byte)
		for _, proposal := range receivedProposalHistory {
			rootsBySlot[proposal.Slot] = proposal.SigningRoot
		}
		for _, proposal := range proposals {
			receivedRoot, ok := rootsBySlot[proposal.Slot]
			require.DeepEqual(t, true, ok)
			require.DeepEqual(
				t,
				receivedRoot,
				proposal.SigningRoot,
				"Imported proposals are different then the generated ones",
			)
		}
	}
}

func Test_validateMetadata(t *testing.T) {
	goodRoot := [32]byte{1}
	goodStr := make([]byte, hex.EncodedLen(len(goodRoot)))
	hex.Encode(goodStr, goodRoot[:])
	tests := []struct {
		name                    string
		interchangeJSON         *format.EIPSlashingProtectionFormat
		dbGenesisValidatorsRoot []byte
		wantErr                 bool
		wantFatal               string
	}{
		{
			name: "Incorrect version for EIP format should fail",
			interchangeJSON: &format.EIPSlashingProtectionFormat{
				Metadata: struct {
					InterchangeFormatVersion string `json:"interchange_format_version"`
					GenesisValidatorsRoot    string `json:"genesis_validators_root"`
				}{
					InterchangeFormatVersion: "1",
					GenesisValidatorsRoot:    string(goodStr),
				},
			},
			wantErr: true,
		},
		{
			name: "Junk data for version should fail",
			interchangeJSON: &format.EIPSlashingProtectionFormat{
				Metadata: struct {
					InterchangeFormatVersion string `json:"interchange_format_version"`
					GenesisValidatorsRoot    string `json:"genesis_validators_root"`
				}{
					InterchangeFormatVersion: "asdljas$d",
					GenesisValidatorsRoot:    string(goodStr),
				},
			},
			wantErr: true,
		},
		{
			name: "Proper version field should pass",
			interchangeJSON: &format.EIPSlashingProtectionFormat{
				Metadata: struct {
					InterchangeFormatVersion string `json:"interchange_format_version"`
					GenesisValidatorsRoot    string `json:"genesis_validators_root"`
				}{
					InterchangeFormatVersion: format.InterchangeFormatVersion,
					GenesisValidatorsRoot:    string(goodStr),
				},
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			validatorDB := dbtest.SetupDB(t, nil)
			ctx := context.Background()
			if err := validateMetadata(ctx, validatorDB, tt.interchangeJSON); (err != nil) != tt.wantErr {
				t.Errorf("validateMetadata() error = %v, wantErr %v", err, tt.wantErr)
			}

		})
	}
}

func Test_validateMetadataGenesisValidatorsRoot(t *testing.T) {
	goodRoot := [32]byte{1}
	goodStr := make([]byte, hex.EncodedLen(len(goodRoot)))
	hex.Encode(goodStr, goodRoot[:])
	secondRoot := [32]byte{2}
	secondStr := make([]byte, hex.EncodedLen(len(secondRoot)))
	hex.Encode(secondStr, secondRoot[:])

	tests := []struct {
		name                    string
		interchangeJSON         *format.EIPSlashingProtectionFormat
		dbGenesisValidatorsRoot []byte
		wantErr                 bool
	}{
		{
			name: "Same genesis roots should not fail",
			interchangeJSON: &format.EIPSlashingProtectionFormat{
				Metadata: struct {
					InterchangeFormatVersion string `json:"interchange_format_version"`
					GenesisValidatorsRoot    string `json:"genesis_validators_root"`
				}{
					InterchangeFormatVersion: format.InterchangeFormatVersion,
					GenesisValidatorsRoot:    string(goodStr),
				},
			},
			dbGenesisValidatorsRoot: goodRoot[:],
			wantErr:                 false,
		},
		{
			name: "Different genesis roots should not fail",
			interchangeJSON: &format.EIPSlashingProtectionFormat{
				Metadata: struct {
					InterchangeFormatVersion string `json:"interchange_format_version"`
					GenesisValidatorsRoot    string `json:"genesis_validators_root"`
				}{
					InterchangeFormatVersion: format.InterchangeFormatVersion,
					GenesisValidatorsRoot:    string(secondStr),
				},
			},
			dbGenesisValidatorsRoot: goodRoot[:],
			wantErr:                 true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			validatorDB := dbtest.SetupDB(t, nil)
			ctx := context.Background()
			require.NoError(t, validatorDB.SaveGenesisValidatorsRoot(ctx, tt.dbGenesisValidatorsRoot))
			err := validateMetadata(ctx, validatorDB, tt.interchangeJSON)
			if tt.wantErr {
				require.ErrorContains(t, "genesis validators root doesn't match the one that is stored", err)
			} else {
				require.NoError(t, err)
			}

		})
	}
}

func Test_parseUniqueSignedBlocksByPubKey(t *testing.T) {
	numValidators := 4
	publicKeys, err := valtest.CreateRandomPubKeys(numValidators)
	require.NoError(t, err)
	roots := valtest.CreateMockRoots(numValidators)
	tests := []struct {
		name    string
		data    []*format.ProtectionData
		want    map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedBlock
		wantErr bool
	}{
		{
			name: "nil values are skipped",
			data: []*format.ProtectionData{
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedBlocks: []*format.SignedBlock{
						{
							Slot:        "1",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
						nil,
					},
				},
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedBlocks: []*format.SignedBlock{
						{
							Slot:        "3",
							SigningRoot: fmt.Sprintf("%x", roots[2]),
						},
					},
				},
			},
			want: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedBlock{
				publicKeys[0]: {
					{
						Slot:        "1",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
					{
						Slot:        "3",
						SigningRoot: fmt.Sprintf("%x", roots[2]),
					},
				},
			},
		},
		{
			name: "same blocks but different public keys are parsed correctly",
			data: []*format.ProtectionData{
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedBlocks: []*format.SignedBlock{
						{
							Slot:        "1",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
						{
							Slot:        "2",
							SigningRoot: fmt.Sprintf("%x", roots[1]),
						},
					},
				},
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[1]),
					SignedBlocks: []*format.SignedBlock{
						{
							Slot:        "1",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
						{
							Slot:        "2",
							SigningRoot: fmt.Sprintf("%x", roots[1]),
						},
					},
				},
			},
			want: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedBlock{
				publicKeys[0]: {
					{
						Slot:        "1",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
					{
						Slot:        "2",
						SigningRoot: fmt.Sprintf("%x", roots[1]),
					},
				},
				publicKeys[1]: {
					{
						Slot:        "1",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
					{
						Slot:        "2",
						SigningRoot: fmt.Sprintf("%x", roots[1]),
					},
				},
			},
		},
		{
			name: "disjoint sets of signed blocks by the same public key are parsed correctly",
			data: []*format.ProtectionData{
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedBlocks: []*format.SignedBlock{
						{
							Slot:        "1",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
						{
							Slot:        "2",
							SigningRoot: fmt.Sprintf("%x", roots[1]),
						},
					},
				},
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedBlocks: []*format.SignedBlock{
						{
							Slot:        "3",
							SigningRoot: fmt.Sprintf("%x", roots[2]),
						},
					},
				},
			},
			want: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedBlock{
				publicKeys[0]: {
					{
						Slot:        "1",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
					{
						Slot:        "2",
						SigningRoot: fmt.Sprintf("%x", roots[1]),
					},
					{
						Slot:        "3",
						SigningRoot: fmt.Sprintf("%x", roots[2]),
					},
				},
			},
		},
		{
			name: "full duplicate entries are uniquely parsed",
			data: []*format.ProtectionData{
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedBlocks: []*format.SignedBlock{
						{
							Slot:        "1",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
					},
				},
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedBlocks: []*format.SignedBlock{
						{
							Slot:        "1",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
					},
				},
			},
			want: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedBlock{
				publicKeys[0]: {
					{
						Slot:        "1",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
					{
						Slot:        "1",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
				},
			},
		},
		{
			name: "intersecting duplicate public key entries are handled properly",
			data: []*format.ProtectionData{
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedBlocks: []*format.SignedBlock{
						{
							Slot:        "1",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
						{
							Slot:        "2",
							SigningRoot: fmt.Sprintf("%x", roots[1]),
						},
					},
				},
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedBlocks: []*format.SignedBlock{
						{
							Slot:        "2",
							SigningRoot: fmt.Sprintf("%x", roots[1]),
						},
						{
							Slot:        "3",
							SigningRoot: fmt.Sprintf("%x", roots[2]),
						},
					},
				},
			},
			want: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedBlock{
				publicKeys[0]: {
					{
						Slot:        "1",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
					{
						Slot:        "2",
						SigningRoot: fmt.Sprintf("%x", roots[1]),
					},
					{
						Slot:        "2",
						SigningRoot: fmt.Sprintf("%x", roots[1]),
					},
					{
						Slot:        "3",
						SigningRoot: fmt.Sprintf("%x", roots[2]),
					},
				},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := parseBlocksForUniquePublicKeys(tt.data)
			if (err != nil) != tt.wantErr {
				t.Errorf("parseBlocksForUniquePublicKeys() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("parseBlocksForUniquePublicKeys() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_parseUniqueSignedAttestationsByPubKey(t *testing.T) {
	numValidators := 4
	publicKeys, err := valtest.CreateRandomPubKeys(numValidators)
	require.NoError(t, err)
	roots := valtest.CreateMockRoots(numValidators)
	tests := []struct {
		name    string
		data    []*format.ProtectionData
		want    map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedAttestation
		wantErr bool
	}{
		{
			name: "nil values are skipped",
			data: []*format.ProtectionData{
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedAttestations: []*format.SignedAttestation{
						{
							SourceEpoch: "1",
							TargetEpoch: "3",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
						nil,
					},
				},
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedAttestations: []*format.SignedAttestation{
						{
							SourceEpoch: "3",
							TargetEpoch: "5",
							SigningRoot: fmt.Sprintf("%x", roots[2]),
						},
					},
				},
			},
			want: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedAttestation{
				publicKeys[0]: {
					{
						SourceEpoch: "1",
						TargetEpoch: "3",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
					{
						SourceEpoch: "3",
						TargetEpoch: "5",
						SigningRoot: fmt.Sprintf("%x", roots[2]),
					},
				},
			},
		},
		{
			name: "same attestations but different public keys are parsed correctly",
			data: []*format.ProtectionData{
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedAttestations: []*format.SignedAttestation{
						{
							SourceEpoch: "1",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
						{
							SourceEpoch: "2",
							SigningRoot: fmt.Sprintf("%x", roots[1]),
						},
					},
				},
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[1]),
					SignedAttestations: []*format.SignedAttestation{
						{
							SourceEpoch: "1",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
						{
							SourceEpoch: "2",
							SigningRoot: fmt.Sprintf("%x", roots[1]),
						},
					},
				},
			},
			want: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedAttestation{
				publicKeys[0]: {
					{
						SourceEpoch: "1",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
					{
						SourceEpoch: "2",
						SigningRoot: fmt.Sprintf("%x", roots[1]),
					},
				},
				publicKeys[1]: {
					{
						SourceEpoch: "1",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
					{
						SourceEpoch: "2",
						SigningRoot: fmt.Sprintf("%x", roots[1]),
					},
				},
			},
		},
		{
			name: "disjoint sets of signed attestations by the same public key are parsed correctly",
			data: []*format.ProtectionData{
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedAttestations: []*format.SignedAttestation{
						{
							SourceEpoch: "1",
							TargetEpoch: "3",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
						{
							SourceEpoch: "2",
							TargetEpoch: "4",
							SigningRoot: fmt.Sprintf("%x", roots[1]),
						},
					},
				},
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedAttestations: []*format.SignedAttestation{
						{
							SourceEpoch: "3",
							TargetEpoch: "5",
							SigningRoot: fmt.Sprintf("%x", roots[2]),
						},
					},
				},
			},
			want: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedAttestation{
				publicKeys[0]: {
					{
						SourceEpoch: "1",
						TargetEpoch: "3",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
					{
						SourceEpoch: "2",
						TargetEpoch: "4",
						SigningRoot: fmt.Sprintf("%x", roots[1]),
					},
					{
						SourceEpoch: "3",
						TargetEpoch: "5",
						SigningRoot: fmt.Sprintf("%x", roots[2]),
					},
				},
			},
		},
		{
			name: "full duplicate entries are uniquely parsed",
			data: []*format.ProtectionData{
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedAttestations: []*format.SignedAttestation{
						{
							SourceEpoch: "1",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
					},
				},
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedAttestations: []*format.SignedAttestation{
						{
							SourceEpoch: "1",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
					},
				},
			},
			want: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedAttestation{
				publicKeys[0]: {
					{
						SourceEpoch: "1",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
					{
						SourceEpoch: "1",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
				},
			},
		},
		{
			name: "intersecting duplicate public key entries are handled properly",
			data: []*format.ProtectionData{
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedAttestations: []*format.SignedAttestation{
						{
							SourceEpoch: "1",
							SigningRoot: fmt.Sprintf("%x", roots[0]),
						},
						{
							SourceEpoch: "2",
							SigningRoot: fmt.Sprintf("%x", roots[1]),
						},
					},
				},
				{
					Pubkey: fmt.Sprintf("%x", publicKeys[0]),
					SignedAttestations: []*format.SignedAttestation{
						{
							SourceEpoch: "2",
							SigningRoot: fmt.Sprintf("%x", roots[1]),
						},
						{
							SourceEpoch: "3",
							SigningRoot: fmt.Sprintf("%x", roots[2]),
						},
					},
				},
			},
			want: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedAttestation{
				publicKeys[0]: {
					{
						SourceEpoch: "1",
						SigningRoot: fmt.Sprintf("%x", roots[0]),
					},
					{
						SourceEpoch: "2",
						SigningRoot: fmt.Sprintf("%x", roots[1]),
					},
					{
						SourceEpoch: "2",
						SigningRoot: fmt.Sprintf("%x", roots[1]),
					},
					{
						SourceEpoch: "3",
						SigningRoot: fmt.Sprintf("%x", roots[2]),
					},
				},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := parseAttestationsForUniquePublicKeys(tt.data)
			if (err != nil) != tt.wantErr {
				t.Errorf("parseAttestationsForUniquePublicKeys() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("parseAttestationsForUniquePublicKeys() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_filterSlashablePubKeysFromBlocks(t *testing.T) {
	var tests = []struct {
		name     string
		expected [][fieldparams.BLSPubkeyLength]byte
		given    map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedBlock
	}{
		{
			name:     "No slashable keys returns empty",
			expected: make([][fieldparams.BLSPubkeyLength]byte, 0),
			given: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedBlock{
				{1}: {
					{
						Slot: "1",
					},
					{
						Slot: "2",
					},
				},
				{2}: {
					{
						Slot: "2",
					},
					{
						Slot: "3",
					},
				},
			},
		},
		{
			name:     "Empty data returns empty",
			expected: make([][fieldparams.BLSPubkeyLength]byte, 0),
			given:    make(map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedBlock),
		},
		{
			name: "Properly finds public keys with slashable data",
			expected: [][fieldparams.BLSPubkeyLength]byte{
				{1}, {3},
			},
			given: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedBlock{
				{1}: {
					{
						Slot: "1",
					},
					{
						Slot: "1",
					},
					{
						Slot: "2",
					},
				},
				{2}: {
					{
						Slot: "2",
					},
					{
						Slot: "3",
					},
				},
				{3}: {
					{
						Slot: "3",
					},
					{
						Slot: "3",
					},
				},
			},
		},
		{
			name: "Considers nil signing roots and mismatched signing roots when determining slashable keys",
			expected: [][fieldparams.BLSPubkeyLength]byte{
				{2}, {3},
			},
			given: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedBlock{
				// Different signing roots and same slot should not be slashable.
				{1}: {
					{
						Slot:        "1",
						SigningRoot: fmt.Sprintf("%#x", [32]byte{1}),
					},
					{
						Slot:        "1",
						SigningRoot: fmt.Sprintf("%#x", [32]byte{1}),
					},
				},
				// No signing root specified but same slot should be slashable.
				{2}: {
					{
						Slot: "2",
					},
					{
						Slot: "2",
					},
				},
				// No signing root in one slot, and same slot with signing root should be slashable.
				{3}: {
					{
						Slot: "3",
					},
					{
						Slot:        "3",
						SigningRoot: fmt.Sprintf("%#x", [32]byte{3}),
					},
				},
			},
		},
	}
	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()
			historyByPubKey := make(map[[fieldparams.BLSPubkeyLength]byte]kv.ProposalHistoryForPubkey)
			for pubKey, signedBlocks := range tt.given {
				proposalHistory, err := transformSignedBlocks(ctx, signedBlocks)
				require.NoError(t, err)
				historyByPubKey[pubKey] = *proposalHistory
			}
			slashablePubKeys := filterSlashablePubKeysFromBlocks(context.Background(), historyByPubKey)
			wantedPubKeys := make(map[[fieldparams.BLSPubkeyLength]byte]bool)
			for _, pk := range tt.expected {
				wantedPubKeys[pk] = true
				wantedPubKeys[pk] = true
			}
			for _, pk := range slashablePubKeys {
				ok := wantedPubKeys[pk]
				require.Equal(t, true, ok)
			}
		})
	}
}

func Test_filterSlashablePubKeysFromAttestations(t *testing.T) {
	ctx := context.Background()
	tests := []struct {
		name                 string
		previousAttsByPubKey map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedAttestation
		incomingAttsByPubKey map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedAttestation
		want                 map[[fieldparams.BLSPubkeyLength]byte]bool
		wantErr              bool
	}{
		{
			name: "Properly filters out double voting attester keys",
			previousAttsByPubKey: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedAttestation{
				{1}: {
					{
						SourceEpoch: "2",
						TargetEpoch: "4",
					},
					{
						SourceEpoch: "2",
						TargetEpoch: "4",
					},
				},
				{2}: {
					{
						SourceEpoch: "2",
						TargetEpoch: "4",
					},
					{
						SourceEpoch: "2",
						TargetEpoch: "5",
					},
				},
				{3}: {
					{
						SourceEpoch: "2",
						TargetEpoch: "4",
					},
					{
						SourceEpoch: "2",
						TargetEpoch: "4",
					},
				},
			},
			want: map[[fieldparams.BLSPubkeyLength]byte]bool{
				{1}: true,
				{3}: true,
			},
		},
		{
			name: "Returns empty if no keys are slashable",
			previousAttsByPubKey: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedAttestation{
				{1}: {
					{
						SourceEpoch: "2",
						TargetEpoch: "4",
					},
				},
				{2}: {
					{
						SourceEpoch: "2",
						TargetEpoch: "4",
					},
					{
						SourceEpoch: "2",
						TargetEpoch: "5",
					},
				},
				{3}: {
					{
						SourceEpoch: "2",
						TargetEpoch: "4",
					},
					{
						SourceEpoch: "3",
						TargetEpoch: "6",
					},
				},
			},
			want: map[[fieldparams.BLSPubkeyLength]byte]bool{},
		},
		{
			name: "Properly filters out surround voting attester keys",
			previousAttsByPubKey: map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedAttestation{
				{1}: {
					{
						SourceEpoch: "2",
						TargetEpoch: "4",
					},
					{
						SourceEpoch: "1",
						TargetEpoch: "5",
					},
				},
				{2}: {
					{
						SourceEpoch: "2",
						TargetEpoch: "4",
					},
					{
						SourceEpoch: "2",
						TargetEpoch: "5",
					},
				},
				{3}: {
					{
						SourceEpoch: "2",
						TargetEpoch: "5",
					},
					{
						SourceEpoch: "3",
						TargetEpoch: "4",
					},
				},
			},
			want: map[[fieldparams.BLSPubkeyLength]byte]bool{
				{1}: true,
				{3}: true,
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			attestingHistoriesByPubKey := make(map[[fieldparams.BLSPubkeyLength]byte][]*kv.AttestationRecord)
			pubKeys := make([][fieldparams.BLSPubkeyLength]byte, 0)
			for pubKey := range tt.incomingAttsByPubKey {
				pubKeys = append(pubKeys, pubKey)
			}
			validatorDB := dbtest.SetupDB(t, pubKeys)
			for pubKey, signedAtts := range tt.incomingAttsByPubKey {
				attestingHistory, err := transformSignedAttestations(pubKey, signedAtts)
				require.NoError(t, err)
				for _, att := range attestingHistory {
					var signingRoot [32]byte
					copy(signingRoot[:], att.SigningRoot)

					indexedAtt := createAttestation(att.Source, att.Target)
					err := validatorDB.SaveAttestationForPubKey(ctx, pubKey, signingRoot, indexedAtt)
					require.NoError(t, err)
				}
			}
			got, err := filterSlashablePubKeysFromAttestations(ctx, validatorDB, attestingHistoriesByPubKey)
			if (err != nil) != tt.wantErr {
				t.Errorf("filterSlashablePubKeysFromAttestations() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			for _, pubKey := range got {
				ok := tt.want[pubKey]
				assert.Equal(t, true, ok)
			}
		})
	}
}
