package testpb

import (
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/dynamicpb"
)

func TestList(t *testing.T) {
	dyn := dynamicpb.NewMessage(md_A)
	msg := (&A{}).ProtoReflect()

	dynList := dyn.Mutable(fd_A_LIST).List()
	list := msg.Mutable(fd_A_LIST).List()

	t.Run("set", func(t *testing.T) {
		// extend list without using the mutable variable
		dynList.AppendMutable()
		list.AppendMutable()

		dynElem := dynList.NewElement()
		dynElem.Message().Set(fd_B_x, protoreflect.ValueOfString("test"))

		elem := list.NewElement()
		elem.Message().Set(fd_B_x, protoreflect.ValueOfString("test"))

		dynList.Set(0, protoreflect.ValueOfMessage(dynElem.Message()))
		list.Set(0, protoreflect.ValueOfMessage(elem.Message()))

		require.True(t, proto.Equal(dyn, msg.Interface()))
	})

	t.Run("append mutable", func(t *testing.T) {
		dynElem := dynList.AppendMutable()
		dynElem.Message().Set(fd_B_x, protoreflect.ValueOfString("test"))

		elem := list.AppendMutable()
		elem.Message().Set(fd_B_x, protoreflect.ValueOfString("test"))

		require.True(t, proto.Equal(dyn, msg.Interface()))
	})

	t.Run("new element - append", func(t *testing.T) {
		dynElem := dynList.NewElement().Message()
		dynElem.Set(fd_B_x, protoreflect.ValueOfString("test"))

		elem := list.NewElement().Message()
		elem.Set(fd_B_x, protoreflect.ValueOfString("test"))

		dynList.Append(protoreflect.ValueOfMessage(dynElem))
		list.Append(protoreflect.ValueOfMessage(elem))

		require.True(t, proto.Equal(dyn, msg.Interface()))

	})

	// we reset everything in case the following tests are run separate to the others above
	dyn.Clear(fd_A_LIST)
	msg.Clear(fd_A_LIST)

	dynList = dyn.Mutable(fd_A_LIST).List()
	list = msg.Mutable(fd_A_LIST).List()

	dynElem := dynList.AppendMutable()
	dynElem.Message().Set(fd_B_x, protoreflect.ValueOfString("test"))

	elem := list.AppendMutable()
	elem.Message().Set(fd_B_x, protoreflect.ValueOfString("test"))

	t.Run("len", func(t *testing.T) {
		require.Equal(t, dynList.Len(), list.Len())
	})

	t.Run("get", func(t *testing.T) {
		dynElem := dynList.Get(0)
		elem := list.Get(0)

		require.True(t, proto.Equal(dynElem.Message().Interface(), elem.Message().Interface()))
	})

	t.Run("validity", func(t *testing.T) {
		require.Equal(t, dynList.IsValid(), list.IsValid())
	})

	t.Run("truncate", func(t *testing.T) {
		dynList.Truncate(0)
		list.Truncate(0)

		require.Equal(t, dynList.Len(), list.Len())
	})
}
