package state_native_test

import (
	"testing"

	statenative "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestReadOnlyValidator_ReturnsErrorOnNil(t *testing.T) {
	if _, err := statenative.NewValidator(nil); err != statenative.ErrNilWrappedValidator {
		t.Errorf("Wrong error returned. Got %v, wanted %v", err, statenative.ErrNilWrappedValidator)
	}
}

func TestReadOnlyValidator_EffectiveBalance(t *testing.T) {
	bal := uint64(234)
	v, err := statenative.NewValidator(&ethpb.Validator{EffectiveBalance: bal})
	require.NoError(t, err)
	assert.Equal(t, bal, v.EffectiveBalance())
}

func TestReadOnlyValidator_ActivationEligibilityEpoch(t *testing.T) {
	epoch := primitives.Epoch(234)
	v, err := statenative.NewValidator(&ethpb.Validator{ActivationEligibilityEpoch: epoch})
	require.NoError(t, err)
	assert.Equal(t, epoch, v.ActivationEligibilityEpoch())
}

func TestReadOnlyValidator_ActivationEpoch(t *testing.T) {
	epoch := primitives.Epoch(234)
	v, err := statenative.NewValidator(&ethpb.Validator{ActivationEpoch: epoch})
	require.NoError(t, err)
	assert.Equal(t, epoch, v.ActivationEpoch())
}

func TestReadOnlyValidator_WithdrawableEpoch(t *testing.T) {
	epoch := primitives.Epoch(234)
	v, err := statenative.NewValidator(&ethpb.Validator{WithdrawableEpoch: epoch})
	require.NoError(t, err)
	assert.Equal(t, epoch, v.WithdrawableEpoch())
}

func TestReadOnlyValidator_ExitEpoch(t *testing.T) {
	epoch := primitives.Epoch(234)
	v, err := statenative.NewValidator(&ethpb.Validator{ExitEpoch: epoch})
	require.NoError(t, err)
	assert.Equal(t, epoch, v.ExitEpoch())
}

func TestReadOnlyValidator_PublicKey(t *testing.T) {
	key := [fieldparams.BLSPubkeyLength]byte{0xFA, 0xCC}
	v, err := statenative.NewValidator(&ethpb.Validator{PublicKey: key[:]})
	require.NoError(t, err)
	assert.Equal(t, key, v.PublicKey())
}

func TestReadOnlyValidator_WithdrawalCredentials(t *testing.T) {
	creds := []byte{0xFA, 0xCC}
	v, err := statenative.NewValidator(&ethpb.Validator{WithdrawalCredentials: creds})
	require.NoError(t, err)
	assert.DeepEqual(t, creds, v.WithdrawalCredentials())
}

func TestReadOnlyValidator_Slashed(t *testing.T) {
	v, err := statenative.NewValidator(&ethpb.Validator{Slashed: true})
	require.NoError(t, err)
	assert.Equal(t, true, v.Slashed())
}
