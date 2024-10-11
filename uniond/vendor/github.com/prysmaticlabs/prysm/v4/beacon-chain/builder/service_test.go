package builder

import (
	"context"
	"testing"
	"time"

	buildertesting "github.com/prysmaticlabs/prysm/v4/api/client/builder/testing"
	blockchainTesting "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	dbtesting "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func Test_NewServiceWithBuilder(t *testing.T) {
	s, err := NewService(context.Background(), WithBuilderClient(&buildertesting.MockClient{}))
	require.NoError(t, err)
	assert.Equal(t, true, s.Configured())
}

func Test_NewServiceWithoutBuilder(t *testing.T) {
	s, err := NewService(context.Background())
	require.NoError(t, err)
	assert.Equal(t, false, s.Configured())
}

func Test_RegisterValidator(t *testing.T) {
	ctx := context.Background()
	db := dbtesting.SetupDB(t)
	headFetcher := &blockchainTesting.ChainService{}
	builder := buildertesting.NewClient()
	s, err := NewService(ctx, WithDatabase(db), WithHeadFetcher(headFetcher), WithBuilderClient(&builder))
	require.NoError(t, err)
	pubkey := bytesutil.ToBytes48([]byte("pubkey"))
	var feeRecipient [20]byte
	require.NoError(t, s.RegisterValidator(ctx, []*eth.SignedValidatorRegistrationV1{{Message: &eth.ValidatorRegistrationV1{Pubkey: pubkey[:], FeeRecipient: feeRecipient[:]}}}))
	assert.Equal(t, true, builder.RegisteredVals[pubkey])
}

func Test_RegisterValidator_WithCache(t *testing.T) {
	ctx := context.Background()
	headFetcher := &blockchainTesting.ChainService{}
	builder := buildertesting.NewClient()
	s, err := NewService(ctx, WithRegistrationCache(), WithHeadFetcher(headFetcher), WithBuilderClient(&builder))
	require.NoError(t, err)
	pubkey := bytesutil.ToBytes48([]byte("pubkey"))
	var feeRecipient [20]byte
	reg := &eth.ValidatorRegistrationV1{Pubkey: pubkey[:], Timestamp: uint64(time.Now().UTC().Unix()), FeeRecipient: feeRecipient[:]}
	require.NoError(t, s.RegisterValidator(ctx, []*eth.SignedValidatorRegistrationV1{{Message: reg}}))
	registration, err := s.registrationCache.RegistrationByIndex(0)
	require.NoError(t, err)
	require.DeepEqual(t, reg, registration)
}

func Test_BuilderMethodsWithouClient(t *testing.T) {
	s, err := NewService(context.Background())
	require.NoError(t, err)
	assert.Equal(t, false, s.Configured())

	_, err = s.GetHeader(context.Background(), 0, [32]byte{}, [48]byte{})
	assert.ErrorContains(t, ErrNoBuilder.Error(), err)

	_, _, err = s.SubmitBlindedBlock(context.Background(), nil)
	assert.ErrorContains(t, ErrNoBuilder.Error(), err)

	err = s.RegisterValidator(context.Background(), nil)
	assert.ErrorContains(t, ErrNoBuilder.Error(), err)
}
