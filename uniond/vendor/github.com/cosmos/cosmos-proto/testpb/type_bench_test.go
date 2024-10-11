package testpb

import (
	"testing"

	"google.golang.org/protobuf/reflect/protoreflect"
)

func getFastType() (fastReflectionType protoreflect.MessageType) {
	return (&A{}).ProtoReflect().Type()
}

func getSlowType() protoreflect.MessageType {
	return (&A{}).slowProtoReflect().Type()
}

func Benchmark_MessageType_New_FR(b *testing.B) {
	typ := getFastType() // if casted to concrete type, performance goes down to: Benchmark_MessageType_New_FR-12         1000000000               0.2431 ns/op

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = typ.New()
	}
}

func Benchmark_MessageType_New_SR(b *testing.B) {
	typ := getSlowType()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = typ.New()
	}
}

func Benchmark_MessageType_Zero_FR(b *testing.B) {
	typ := getFastType()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = typ.Zero()
	}
}

func Benchmark_MessageType_Zero_SR(b *testing.B) {
	typ := getSlowType()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = typ.Zero()
	}
}

func Benchmark_MessageType_Descriptor_FR(b *testing.B) {
	typ := getFastType()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = typ.Descriptor()
	}
}

func Benchmark_MessageType_Descriptor_SR(b *testing.B) {
	typ := getSlowType()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = typ.Descriptor()
	}
}
