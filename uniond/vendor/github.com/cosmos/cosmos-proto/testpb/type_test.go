package testpb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestType(t *testing.T) {
	typ := (&A{}).ProtoReflect().Type()
	sTyp := (&A{}).slowProtoReflect().Type()

	require.Equal(t, sTyp.Descriptor(), typ.Descriptor())         // assert descriptor equality
	require.Equal(t, sTyp.Zero().IsValid(), typ.Zero().IsValid()) // assert invalidity
}
