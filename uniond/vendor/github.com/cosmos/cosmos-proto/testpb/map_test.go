package testpb

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/dynamicpb"
)

func TestMap(t *testing.T) {
	dyn := dynamicpb.NewMessage(md_A)
	msg := (&A{}).ProtoReflect()

	dynMap := dyn.Mutable(fd_A_MAP).Map()
	mapv := msg.Mutable(fd_A_MAP).Map()

	// we set a value in the map
	dynValue := dynMap.NewValue().Message()
	dynValue.Set(fd_B_x, protoreflect.ValueOfString("value"))

	value := mapv.NewValue().Message()
	value.Set(fd_B_x, protoreflect.ValueOfString("value"))

	dynMap.Set((protoreflect.MapKey)(protoreflect.ValueOfString("key")), protoreflect.ValueOfMessage(dynValue))
	mapv.Set((protoreflect.MapKey)(protoreflect.ValueOfString("key")), protoreflect.ValueOfMessage(value))

	// test set
	t.Run("set", func(t *testing.T) {
		dynValue := dynMap.NewValue().Message()
		dynValue.Set(fd_B_x, protoreflect.ValueOfString("something"))

		value := mapv.NewValue().Message()
		value.Set(fd_B_x, protoreflect.ValueOfString("something"))

		dynMap.Set((protoreflect.MapKey)(protoreflect.ValueOfString("test")), protoreflect.ValueOfMessage(dynValue))
		mapv.Set((protoreflect.MapKey)(protoreflect.ValueOfString("test")), protoreflect.ValueOfMessage(value))

		require.True(t, proto.Equal(dyn, msg.Interface()))

	})

	// test get
	t.Run("get", func(t *testing.T) {
		dynValue := dynMap.Get(protoreflect.MapKey(protoreflect.ValueOfString("key")))
		value := mapv.Get(protoreflect.MapKey(protoreflect.ValueOfString("key")))

		require.True(t, proto.Equal(dynValue.Message().Interface(), value.Message().Interface()))
	})

	// test len
	t.Run("len", func(t *testing.T) {
		require.Equal(t, dynMap.Len(), mapv.Len())
	})

	// test has
	t.Run("has", func(t *testing.T) {
		// case exists
		require.Equal(t, dynMap.Has(protoreflect.MapKey(protoreflect.ValueOfString("key"))), mapv.Has(protoreflect.MapKey(protoreflect.ValueOfString("key"))))
		// case not exists
		require.Equal(t, dynMap.Has(protoreflect.MapKey(protoreflect.ValueOfString("not-exist"))), mapv.Has(protoreflect.MapKey(protoreflect.ValueOfString("not-exist"))))
	})

	// test clear
	t.Run("clear", func(t *testing.T) {
		dynMap.Clear(protoreflect.MapKey(protoreflect.ValueOfString("key")))
		mapv.Clear(protoreflect.MapKey(protoreflect.ValueOfString("key")))

		require.True(t, proto.Equal(dyn, msg.Interface()))
	})

	// test range with mutable
	t.Run("range mutable", func(t *testing.T) {
		dyn.Clear(fd_A_MAP)
		msg.Clear(fd_A_MAP)

		dynMap = dyn.Mutable(fd_A_MAP).Map()
		mapv = msg.Mutable(fd_A_MAP).Map()

		insert := func(m protoreflect.Map, n int) {
			mutableMsg := m.Mutable(protoreflect.MapKey(protoreflect.ValueOfString(fmt.Sprintf("%d", n)))).Message()
			mutableMsg.Set(fd_B_x, protoreflect.ValueOfString(fmt.Sprintf("%d", n)))
		}

		nElems := 10

		for i := 0; i < nElems; i++ {
			insert(dynMap, i)
			insert(mapv, i)
		}

		dynM := make(map[string]protoreflect.Message)
		msgM := make(map[string]protoreflect.Message)

		dynMap.Range(func(key protoreflect.MapKey, value protoreflect.Value) bool {
			dynM[key.String()] = value.Message()
			return true
		})

		mapv.Range(func(key protoreflect.MapKey, value protoreflect.Value) bool {
			msgM[key.String()] = value.Message()
			return true
		})

		require.Equal(t, dynMap.Len(), nElems)
		require.Equal(t, dynMap.Len(), mapv.Len())

		for k, v := range dynM {
			vm, ok := msgM[k]
			require.True(t, ok, "map key ", k, "does not exist")

			require.True(t, proto.Equal(v.Interface(), vm.Interface()))
		}
	})

	// test validity
	t.Run("valid", func(t *testing.T) {
		require.Equal(t, dynMap.IsValid(), mapv.IsValid())
	})

}
