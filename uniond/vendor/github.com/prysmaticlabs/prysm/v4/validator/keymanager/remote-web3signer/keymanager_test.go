package remote_web3signer

import (
	"context"
	"encoding/hex"
	"fmt"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	validatorpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/validator-client"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/remote-web3signer/internal"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/remote-web3signer/v1/mock"
	"github.com/stretchr/testify/assert"
)

type MockClient struct {
	Signature       string
	PublicKeys      []string
	isThrowingError bool
}

func (mc *MockClient) Sign(_ context.Context, _ string, _ internal.SignRequestJson) (bls.Signature, error) {
	decoded, err := hexutil.Decode(mc.Signature)
	if err != nil {
		return nil, err
	}
	return bls.SignatureFromBytes(decoded)
}
func (mc *MockClient) GetPublicKeys(_ context.Context, _ string) ([][48]byte, error) {
	var keys [][48]byte
	for _, pk := range mc.PublicKeys {
		decoded, err := hex.DecodeString(strings.TrimPrefix(pk, "0x"))
		if err != nil {
			return nil, err
		}
		keys = append(keys, bytesutil.ToBytes48(decoded))
	}
	if mc.isThrowingError {
		return nil, fmt.Errorf("mock error")
	}
	return keys, nil
}

func TestKeymanager_Sign(t *testing.T) {
	client := &MockClient{
		Signature: "0xb3baa751d0a9132cfe93e4e3d5ff9075111100e3789dca219ade5a24d27e19d16b3353149da1833e9b691bb38634e8dc04469be7032132906c927d7e1a49b414730612877bc6b2810c8f202daf793d1ab0d6b5cb21d52f9e52e883859887a5d9",
	}
	ctx := context.Background()
	root, err := hexutil.Decode("0x270d43e74ce340de4bca2b1936beca0f4f5408d9e78aec4850920baf659d5b69")
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	config := &SetupConfig{
		BaseEndpoint:          "http://example.com",
		GenesisValidatorsRoot: root,
		PublicKeysURL:         "http://example2.com/api/v1/eth2/publicKeys",
	}
	km, err := NewKeymanager(ctx, config)
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	km.client = client
	desiredSigBytes, err := hexutil.Decode(client.Signature)
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	desiredSig, err := bls.SignatureFromBytes(desiredSigBytes)
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	type args struct {
		request *validatorpb.SignRequest
	}
	tests := []struct {
		name    string
		args    args
		want    bls.Signature
		wantErr bool
	}{
		{
			name: "AGGREGATION_SLOT",
			args: args{
				request: mock.GetMockSignRequest("AGGREGATION_SLOT"),
			},
			want:    desiredSig,
			wantErr: false,
		},
		{
			name: "AGGREGATE_AND_PROOF",
			args: args{
				request: mock.GetMockSignRequest("AGGREGATE_AND_PROOF"),
			},
			want:    desiredSig,
			wantErr: false,
		},
		{
			name: "ATTESTATION",
			args: args{
				request: mock.GetMockSignRequest("ATTESTATION"),
			},
			want:    desiredSig,
			wantErr: false,
		},
		{
			name: "BLOCK",
			args: args{
				request: mock.GetMockSignRequest("BLOCK"),
			},
			want:    desiredSig,
			wantErr: false,
		},
		{
			name: "BLOCK_V2",
			args: args{
				request: mock.GetMockSignRequest("BLOCK_V2"),
			},
			want:    desiredSig,
			wantErr: false,
		},
		{
			name: "RANDAO_REVEAL",
			args: args{
				request: mock.GetMockSignRequest("RANDAO_REVEAL"),
			},
			want:    desiredSig,
			wantErr: false,
		},
		{
			name: "SYNC_COMMITTEE_CONTRIBUTION_AND_PROOF",
			args: args{
				request: mock.GetMockSignRequest("SYNC_COMMITTEE_CONTRIBUTION_AND_PROOF"),
			},
			want:    desiredSig,
			wantErr: false,
		},
		{
			name: "SYNC_COMMITTEE_MESSAGE",
			args: args{
				request: mock.GetMockSignRequest("SYNC_COMMITTEE_MESSAGE"),
			},
			want:    desiredSig,
			wantErr: false,
		},
		{
			name: "SYNC_COMMITTEE_SELECTION_PROOF",
			args: args{
				request: mock.GetMockSignRequest("SYNC_COMMITTEE_SELECTION_PROOF"),
			},
			want:    desiredSig,
			wantErr: false,
		},
		{
			name: "VOLUNTARY_EXIT",
			args: args{
				request: mock.GetMockSignRequest("VOLUNTARY_EXIT"),
			},
			want:    desiredSig,
			wantErr: false,
		},
		{
			name: "VALIDATOR_REGISTRATION",
			args: args{
				request: mock.GetMockSignRequest("VALIDATOR_REGISTRATION"),
			},
			want:    desiredSig,
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := km.Sign(ctx, tt.args.request)
			if (err != nil) != tt.wantErr {
				t.Errorf("name:%s error = %v, wantErr %v", tt.name, err, tt.wantErr)
				return
			}
			require.DeepEqual(t, got, tt.want)
		})
	}

}

func TestKeymanager_FetchValidatingPublicKeys_HappyPath_WithKeyList(t *testing.T) {
	ctx := context.Background()
	decodedKey, err := hexutil.Decode("0xa2b5aaad9c6efefe7bb9b1243a043404f3362937cfb6b31833929833173f476630ea2cfeb0d9ddf15f97ca8685948820")
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	keys := [][48]byte{
		bytesutil.ToBytes48(decodedKey),
	}
	root, err := hexutil.Decode("0x270d43e74ce340de4bca2b1936beca0f4f5408d9e78aec4850920baf659d5b69")
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	config := &SetupConfig{
		BaseEndpoint:          "http://example.com",
		GenesisValidatorsRoot: root,
		ProvidedPublicKeys:    keys,
	}
	km, err := NewKeymanager(ctx, config)
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	resp, err := km.FetchValidatingPublicKeys(ctx)
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	assert.NotNil(t, resp)
	assert.Nil(t, err)
	assert.EqualValues(t, resp, keys)
}

func TestKeymanager_FetchValidatingPublicKeys_HappyPath_WithExternalURL(t *testing.T) {
	ctx := context.Background()
	client := &MockClient{
		PublicKeys: []string{"0xa2b5aaad9c6efefe7bb9b1243a043404f3362937cfb6b31833929833173f476630ea2cfeb0d9ddf15f97ca8685948820"},
	}
	decodedKey, err := hexutil.Decode("0xa2b5aaad9c6efefe7bb9b1243a043404f3362937cfb6b31833929833173f476630ea2cfeb0d9ddf15f97ca8685948820")
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	keys := [][48]byte{
		bytesutil.ToBytes48(decodedKey),
	}
	root, err := hexutil.Decode("0x270d43e74ce340de4bca2b1936beca0f4f5408d9e78aec4850920baf659d5b69")
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	config := &SetupConfig{
		BaseEndpoint:          "http://example.com",
		GenesisValidatorsRoot: root,
		PublicKeysURL:         "http://example2.com/api/v1/eth2/publicKeys",
	}
	km, err := NewKeymanager(ctx, config)
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	km.client = client
	resp, err := km.FetchValidatingPublicKeys(ctx)
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	assert.NotNil(t, resp)
	assert.Nil(t, err)
	assert.EqualValues(t, resp, keys)
}

func TestKeymanager_FetchValidatingPublicKeys_WithExternalURL_ThrowsError(t *testing.T) {
	ctx := context.Background()
	client := &MockClient{
		PublicKeys:      []string{"0xa2b5aaad9c6efefe7bb9b1243a043404f3362937cfb6b31833929833173f476630ea2cfeb0d9ddf15f97ca8685948820"},
		isThrowingError: true,
	}
	root, err := hexutil.Decode("0x270d43e74ce340de4bca2b1936beca0f4f5408d9e78aec4850920baf659d5b69")
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	config := &SetupConfig{
		BaseEndpoint:          "http://example.com",
		GenesisValidatorsRoot: root,
		PublicKeysURL:         "http://example2.com/api/v1/eth2/publicKeys",
	}
	km, err := NewKeymanager(ctx, config)
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	km.client = client
	resp, err := km.FetchValidatingPublicKeys(ctx)
	assert.NotNil(t, err)
	assert.Nil(t, resp)
	assert.Equal(t, "could not get public keys from remote server url: http://example2.com/api/v1/eth2/publicKeys: mock error", fmt.Sprintf("%v", err))
}

func TestKeymanager_AddPublicKeys(t *testing.T) {
	ctx := context.Background()
	root, err := hexutil.Decode("0x270d43e74ce340de4bca2b1936beca0f4f5408d9e78aec4850920baf659d5b69")
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	config := &SetupConfig{
		BaseEndpoint:          "http://example.com",
		GenesisValidatorsRoot: root,
	}
	km, err := NewKeymanager(ctx, config)
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	publicKeys := []string{"0xa2b5aaad9c6efefe7bb9b1243a043404f3362937cfb6b31833929833173f476630ea2cfeb0d9ddf15f97ca8685948820"}
	statuses := km.AddPublicKeys(publicKeys)
	for _, status := range statuses {
		require.Equal(t, keymanager.StatusImported, status.Status)
	}
	statuses = km.AddPublicKeys(publicKeys)
	for _, status := range statuses {
		require.Equal(t, keymanager.StatusDuplicate, status.Status)
	}
}

func TestKeymanager_DeletePublicKeys(t *testing.T) {
	ctx := context.Background()
	root, err := hexutil.Decode("0x270d43e74ce340de4bca2b1936beca0f4f5408d9e78aec4850920baf659d5b69")
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	config := &SetupConfig{
		BaseEndpoint:          "http://example.com",
		GenesisValidatorsRoot: root,
	}
	km, err := NewKeymanager(ctx, config)
	if err != nil {
		fmt.Printf("error: %v", err)
	}
	publicKeys := []string{"0xa2b5aaad9c6efefe7bb9b1243a043404f3362937cfb6b31833929833173f476630ea2cfeb0d9ddf15f97ca8685948820"}
	statuses := km.AddPublicKeys(publicKeys)
	for _, status := range statuses {
		require.Equal(t, keymanager.StatusImported, status.Status)
	}

	s := km.DeletePublicKeys(publicKeys)
	for _, status := range s {
		require.Equal(t, keymanager.StatusDeleted, status.Status)
	}

	s = km.DeletePublicKeys(publicKeys)
	for _, status := range s {
		require.Equal(t, keymanager.StatusNotFound, status.Status)
	}
}
