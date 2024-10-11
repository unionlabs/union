package testpb

import (
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/dynamicpb"
	"google.golang.org/protobuf/types/known/anypb"
)

func TestNewField(t *testing.T) {

	t.Run("matching values", func(t *testing.T) {
		fds := (&A{}).slowProtoReflect().Descriptor().Fields()

		dyn := dynamicpb.NewMessage((&A{}).slowProtoReflect().Descriptor())

		for i := 0; i < fds.Len(); i++ {
			fd := fds.Get(i)

			dynV := dyn.NewField(fd)
			v := (&A{}).ProtoReflect().NewField(fd)

			switch {
			case fd.IsMap():
				// cast to map
				require.NotPanics(t, func() {
					v.Map()
					dynV.Map()
				})
				// validity
				require.Equal(t, dynV.IsValid(), v.IsValid())
			case fd.IsList():
				// cast to list
				require.NotPanics(t, func() {
					v.List()
					dynV.List()
				})
				// validity
				require.Equal(t, dynV.IsValid(), v.IsValid())
			case !fd.HasPresence():
				require.Equal(t, dynV, v)
			case fd.Kind() == protoreflect.MessageKind:
				require.NotPanics(t, func() {
					v.Message()
					dynV.Message()

					require.Equal(t, dynV.IsValid(), v.IsValid())
				})
			}
		}
	})

	t.Run("invalid fd", func(t *testing.T) {
		invalidFd := (&anypb.Any{}).ProtoReflect().Descriptor().Fields().ByName("value")

		require.Panics(t, func() {
			dynamicpb.NewMessage((&A{}).slowProtoReflect().Descriptor()).NewField(invalidFd)
		})

		require.Panics(t, func() {
			(&A{}).ProtoReflect().NewField(invalidFd)
		})

	})

	t.Run("extensions panic", func(t *testing.T) {
		// TODO(fdymylja)
	})
}
