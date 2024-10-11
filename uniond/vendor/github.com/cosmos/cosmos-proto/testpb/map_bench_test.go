package testpb

import (
	"testing"

	"google.golang.org/protobuf/reflect/protoreflect"
)

func newFastMap() protoreflect.Map {
	msg := &A{MAP: map[string]*B{
		"1": &B{X: "a"},
	}}

	return msg.ProtoReflect().Get(msg.ProtoReflect().Descriptor().Fields().ByName("MAP")).Map()
}

func newSlowMap() protoreflect.Map {
	msg := &A{MAP: map[string]*B{
		"1": &B{X: "a"},
	}}

	return msg.slowProtoReflect().Get(msg.ProtoReflect().Descriptor().Fields().ByName("MAP")).Map()
}

func Benchmark_Map_Get_FR(b *testing.B) {
	m := newFastMap()
	key := (protoreflect.MapKey)(protoreflect.ValueOfString("1"))

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = m.Get(key)
	}
}

func Benchmark_Map_Get_SR(b *testing.B) {
	m := newSlowMap()
	key := (protoreflect.MapKey)(protoreflect.ValueOfString("1"))

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = m.Get(key)
	}
}

func Benchmark_Map_Has_FR(b *testing.B) {
	m := newFastMap()
	key := (protoreflect.MapKey)(protoreflect.ValueOfString("1"))

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = m.Has(key)
	}
}

func Benchmark_Map_Has_SR(b *testing.B) {
	m := newSlowMap()
	key := (protoreflect.MapKey)(protoreflect.ValueOfString("1"))

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = m.Has(key)
	}
}

func Benchmark_Map_Clear_FR(b *testing.B) {
	m := newFastMap()
	key := (protoreflect.MapKey)(protoreflect.ValueOfString("1"))

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		m.Clear(key)
	}
}

func Benchmark_Map_Clear_SR(b *testing.B) {
	m := newSlowMap()
	key := (protoreflect.MapKey)(protoreflect.ValueOfString("1"))

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		m.Clear(key)
	}
}

func Benchmark_Map_Set_FR(b *testing.B) {
	m := newFastMap()
	key := (protoreflect.MapKey)(protoreflect.ValueOfString("2"))
	value := protoreflect.ValueOfMessage((&B{X: "b"}).ProtoReflect())
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		m.Set(key, value)
	}
}

func Benchmark_Map_Set_SR(b *testing.B) {
	m := newSlowMap()
	key := (protoreflect.MapKey)(protoreflect.ValueOfString("2"))
	value := protoreflect.ValueOfMessage((&B{X: "b"}).ProtoReflect())

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		m.Set(key, value)
	}
}

func Benchmark_Map_Mutable_FR(b *testing.B) {
	m := newFastMap()
	key := (protoreflect.MapKey)(protoreflect.ValueOfString("2"))
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		m.Mutable(key)
	}
}

func Benchmark_Map_Mutable_SR(b *testing.B) {
	m := newSlowMap()
	key := (protoreflect.MapKey)(protoreflect.ValueOfString("2"))

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		m.Mutable(key)
	}
}

func Benchmark_Map_NewValue_FR(b *testing.B) {
	m := newFastMap()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = m.NewValue()
	}
}

func Benchmark_Map_NewValue_SR(b *testing.B) {
	m := newSlowMap()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = m.NewValue()
	}
}

func Benchmark_Map_Len_FR(b *testing.B) {
	m := newFastMap()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = m.Len()
	}
}

func Benchmark_Map_Len_SR(b *testing.B) {
	m := newSlowMap()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = m.Len()
	}
}

func Benchmark_Map_Range_FR(b *testing.B) {
	msg := &A{MAP: map[string]*B{
		"1":  {X: "a"},
		"2":  {X: "b"},
		"3":  {X: "c"},
		"4":  {X: "d"},
		"5":  {X: "e"},
		"6":  {X: "f"},
		"7":  {X: "g"},
		"8":  {X: "h"},
		"9":  {X: "i"},
		"10": {X: "j"},
	}}

	m := msg.ProtoReflect().Get(msg.ProtoReflect().Descriptor().Fields().ByName("MAP")).Map()
	for i := 0; i < b.N; i++ {
		m.Range(func(_ protoreflect.MapKey, _ protoreflect.Value) bool {
			return true
		})
	}
}

func Benchmark_Map_Range_SR(b *testing.B) {
	msg := &A{MAP: map[string]*B{
		"1":  {X: "a"},
		"2":  {X: "b"},
		"3":  {X: "c"},
		"4":  {X: "d"},
		"5":  {X: "e"},
		"6":  {X: "f"},
		"7":  {X: "g"},
		"8":  {X: "h"},
		"9":  {X: "i"},
		"10": {X: "j"},
	}}

	m := msg.slowProtoReflect().Get(msg.ProtoReflect().Descriptor().Fields().ByName("MAP")).Map()

	for i := 0; i < b.N; i++ {
		m.Range(func(_ protoreflect.MapKey, _ protoreflect.Value) bool {
			return true
		})
	}
}
