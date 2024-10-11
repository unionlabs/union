package testpb

import (
	"testing"

	"google.golang.org/protobuf/reflect/protoreflect"
)

func newFastList() protoreflect.List {
	msg := &A{LIST: []*B{
		{
			X: "test",
		},
	}}

	fd := msg.ProtoReflect().Descriptor().Fields().ByName("LIST")

	list := msg.ProtoReflect().Get(fd).List()

	return list
}

func newSlowList() protoreflect.List {
	msg := &A{LIST: []*B{
		{
			X: "test",
		},
	}}

	fd := msg.ProtoReflect().Descriptor().Fields().ByName("LIST")

	list := msg.slowProtoReflect().Get(fd).List()

	return list
}

func Benchmark_List_Get_FR(b *testing.B) {
	list := newFastList()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = list.Get(0)
	}
}

func Benchmark_List_Get_SR(b *testing.B) {
	list := newSlowList()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = list.Get(0)
	}
}

func Benchmark_List_Append_FR(b *testing.B) {
	list := newFastList()

	item := protoreflect.ValueOfMessage((&B{X: "test"}).ProtoReflect())

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		list.Append(item)
	}
}

func Benchmark_List_Append_SR(b *testing.B) {
	list := newSlowList()
	item := protoreflect.ValueOfMessage((&B{X: "test"}).ProtoReflect())

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		list.Append(item)
	}
}

func Benchmark_List_SetFR(b *testing.B) {
	list := newFastList()
	item := protoreflect.ValueOfMessage((&B{X: "test"}).ProtoReflect())
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		list.Set(0, item)
	}
}

func Benchmark_List_SetSR(b *testing.B) {
	list := newSlowList()
	item := protoreflect.ValueOfMessage((&B{X: "test"}).ProtoReflect())

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		list.Set(0, item)
	}
}

func Benchmark_List_Len_FR(b *testing.B) {
	list := newFastList()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		list.Len()
	}
}

func Benchmark_List_Len_SR(b *testing.B) {
	list := newSlowList()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		list.Len()
	}
}

func Benchmark_List_AppendMutable_FR(b *testing.B) {
	list := newFastList()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = list.AppendMutable()
	}
}

func Benchmark_List_AppendMutable_SR(b *testing.B) {
	list := newSlowList()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = list.AppendMutable()
	}
}

func Benchmark_List_NewElement_FR(b *testing.B) {
	list := newFastList()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = list.NewElement()
	}
}

func Benchmark_List_NewElement_SR(b *testing.B) {
	list := newSlowList()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = list.NewElement()
	}
}

func Benchmark_List_Truncate_FR(b *testing.B) {
	list := newFastList()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		list.Truncate(1)
	}
}

func Benchmark_List_Truncate_SR(b *testing.B) {
	list := newSlowList()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		list.Truncate(1)
	}
}
