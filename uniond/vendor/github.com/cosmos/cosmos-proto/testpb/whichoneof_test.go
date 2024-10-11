package testpb

import (
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/types/known/structpb"
)

func TestWhichOneof(t *testing.T) {
	od := (&A{}).ProtoReflect().Descriptor().Oneofs().ByName("ONEOF")
	t.Run("unknown oneof", func(t *testing.T) {
		msg := &A{
			ONEOF: &A_ONEOF_B{ONEOF_B: &B{X: "allelujah"}},
		}

		require.Panics(t, func() {
			msg.ProtoReflect().WhichOneof((&structpb.Value{}).ProtoReflect().Descriptor().Oneofs().Get(0))
		})

	})

	t.Run("valid oneof", func(t *testing.T) {
		msg := &A{
			ONEOF: &A_ONEOF_B{ONEOF_B: &B{X: "allelujah"}},
		}

		require.Equal(t, msg.ProtoReflect().WhichOneof(od), msg.ProtoReflect().Descriptor().Fields().ByName("ONEOF_B"))
	})
}
