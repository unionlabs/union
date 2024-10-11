package test3

import (
	"github.com/cosmos/cosmos-proto/internal/fuzz"
	"github.com/google/go-cmp/cmp"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/testing/protocmp"
	"pgregory.net/rapid"
	"testing"
)

func TestMarshalUnmarshal(t *testing.T) {
	t.Run("marshal unmarshal", rapid.MakeCheck(func(t *rapid.T) {
		mType := (&TestAllTypes{}).ProtoReflect().Type()
		msg := fuzz.Message(t, mType)

		msgBytes, err := proto.MarshalOptions{Deterministic: true}.Marshal(msg.Interface())
		require.NoError(t, err)

		uMsg := mType.New()
		err = proto.UnmarshalOptions{}.Unmarshal(msgBytes, uMsg.Interface())
		require.NoError(t, err)
		cmpOpt := protocmp.Transform()
		diff := cmp.Diff(uMsg.Interface(), msg.Interface(), cmpOpt)
		require.Emptyf(t, diff, "non matching messages\n%s", diff)
	}))
}

// TestZeroValueOneofIsMarshalled tests that zero values in oneofs are marshalled
func TestZeroValueOneofIsMarshalled(t *testing.T) {
	msg1 := &TestAllTypes{OneofField: &TestAllTypes_OneofEnum{OneofEnum: TestAllTypes_FOO}}
	b, err := proto.Marshal(msg1)
	require.NoError(t, err)

	msg2 := &TestAllTypes{}
	require.NoError(t, proto.Unmarshal(b, msg2))

	require.True(t, msg2.ProtoReflect().Has(fd_TestAllTypes_oneof_enum))
}
