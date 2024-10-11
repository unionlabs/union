package aminojson

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/x/tx/signing/aminojson/internal/testpb"
)

func Test_getMessageAminoName(t *testing.T) {
	msg := &testpb.ABitOfEverything{}
	name, ok := getMessageAminoName(msg.ProtoReflect())
	require.True(t, ok)
	require.Equal(t, "ABitOfEverything", name)

	secondMsg := &testpb.Duration{}
	_, ok = getMessageAminoName(secondMsg.ProtoReflect())
	require.False(t, ok)
}

func Test_getMessageAminoNameAny(t *testing.T) {
	msg := &testpb.ABitOfEverything{}
	name := getMessageAminoNameAny(msg.ProtoReflect())
	require.Equal(t, "ABitOfEverything", name)

	secondMsg := &testpb.Duration{}
	name = getMessageAminoNameAny(secondMsg.ProtoReflect())
	require.Equal(t, "/testpb.Duration", name)
}
