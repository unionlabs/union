package util

import (
	"context"
	"testing"

	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestNewBeaconState(t *testing.T) {
	st, err := NewBeaconState()
	require.NoError(t, err)
	b, err := st.MarshalSSZ()
	require.NoError(t, err)
	got := &ethpb.BeaconState{}
	require.NoError(t, got.UnmarshalSSZ(b))
	assert.DeepEqual(t, st.ToProtoUnsafe(), got)
}

func TestNewBeaconStateAltair(t *testing.T) {
	st, err := NewBeaconStateAltair()
	require.NoError(t, err)
	b, err := st.MarshalSSZ()
	require.NoError(t, err)
	got := &ethpb.BeaconStateAltair{}
	require.NoError(t, got.UnmarshalSSZ(b))
	assert.DeepEqual(t, st.ToProtoUnsafe(), got)
}

func TestNewBeaconStateBellatrix(t *testing.T) {
	st, err := NewBeaconStateBellatrix()
	require.NoError(t, err)
	b, err := st.MarshalSSZ()
	require.NoError(t, err)
	got := &ethpb.BeaconStateBellatrix{}
	require.NoError(t, got.UnmarshalSSZ(b))
	assert.DeepEqual(t, st.ToProtoUnsafe(), got)
}

func TestNewBeaconStateCapella(t *testing.T) {
	st, err := NewBeaconStateCapella()
	require.NoError(t, err)
	b, err := st.MarshalSSZ()
	require.NoError(t, err)
	got := &ethpb.BeaconStateCapella{}
	require.NoError(t, got.UnmarshalSSZ(b))
	assert.DeepEqual(t, st.ToProtoUnsafe(), got)
}

func TestNewBeaconStateDeneb(t *testing.T) {
	st, err := NewBeaconStateDeneb()
	require.NoError(t, err)
	b, err := st.MarshalSSZ()
	require.NoError(t, err)
	got := &ethpb.BeaconStateDeneb{}
	require.NoError(t, got.UnmarshalSSZ(b))
	assert.DeepEqual(t, st.ToProtoUnsafe(), got)
}

func TestNewBeaconState_HashTreeRoot(t *testing.T) {
	st, err := NewBeaconState()
	require.NoError(t, err)
	_, err = st.HashTreeRoot(context.Background())
	require.NoError(t, err)
	st, err = NewBeaconStateAltair()
	require.NoError(t, err)
	_, err = st.HashTreeRoot(context.Background())
	require.NoError(t, err)
	st, err = NewBeaconStateBellatrix()
	require.NoError(t, err)
	_, err = st.HashTreeRoot(context.Background())
	require.NoError(t, err)
	st, err = NewBeaconStateCapella()
	require.NoError(t, err)
	_, err = st.HashTreeRoot(context.Background())
	require.NoError(t, err)
}
