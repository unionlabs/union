package integration_test

import (
	"context"
	"testing"

	"github.com/grpc-ecosystem/grpc-gateway/examples/internal/clients/abe"
	"github.com/grpc-ecosystem/grpc-gateway/examples/internal/clients/echo"
	"github.com/grpc-ecosystem/grpc-gateway/examples/internal/clients/unannotatedecho"
)

func TestEchoClient(t *testing.T) {
	if testing.Short() {
		t.Skip()
		return
	}

	cfg := echo.NewConfiguration()
	cfg.BasePath = "http://localhost:8088"

	cl := echo.NewAPIClient(cfg)
	resp, _, err := cl.EchoServiceApi.EchoServiceEcho(context.Background(), "foo")
	if err != nil {
		t.Errorf(`cl.EchoServiceApi.Echo("foo") failed with %v; want success`, err)
	}
	if got, want := resp.Id, "foo"; got != want {
		t.Errorf("resp.Id = %q; want %q", got, want)
	}
}

func TestEchoBodyClient(t *testing.T) {
	if testing.Short() {
		t.Skip()
		return
	}

	cfg := echo.NewConfiguration()
	cfg.BasePath = "http://localhost:8088"

	cl := echo.NewAPIClient(cfg)
	req := echo.ExamplepbSimpleMessage{Id: "foo"}
	resp, _, err := cl.EchoServiceApi.EchoServiceEchoBody(context.Background(), req)
	if err != nil {
		t.Errorf("cl.EchoBody(%#v) failed with %v; want success", req, err)
	}
	if got, want := resp.Id, "foo"; got != want {
		t.Errorf("resp.Id = %q; want %q", got, want)
	}
}

func TestAbitOfEverythingClient(t *testing.T) {
	if testing.Short() {
		t.Skip()
		return
	}

	cfg := abe.NewConfiguration()
	cfg.BasePath = "http://localhost:8088"

	cl := abe.NewAPIClient(cfg)

	testABEClientCreate(t, cl)
}

func testABEClientCreate(t *testing.T, cl *abe.APIClient) {
	enumZero := abe.ZERO_ExamplepbNumericEnum
	enumPath := abe.ABC_PathenumPathEnum
	messagePath := abe.JKL_MessagePathEnumNestedPathEnum

	want := &abe.ExamplepbABitOfEverything{
		FloatValue:               1.5,
		DoubleValue:              2.5,
		Int64Value:               "4294967296",
		Uint64Value:              "9223372036854775807",
		Int32Value:               -2147483648,
		Fixed64Value:             "9223372036854775807",
		Fixed32Value:             4294967295,
		BoolValue:                true,
		StringValue:              "strprefix/foo",
		Uint32Value:              4294967295,
		Sfixed32Value:            2147483647,
		Sfixed64Value:            "-4611686018427387904",
		Sint32Value:              2147483647,
		Sint64Value:              "4611686018427387903",
		NonConventionalNameValue: "camelCase",
		EnumValue:                &enumZero,
		PathEnumValue:            &enumPath,
		NestedPathEnumValue:      &messagePath,
		EnumValueAnnotation:      &enumZero,
	}
	resp, _, err := cl.ABitOfEverythingServiceApi.ABitOfEverythingServiceCreate(
		context.Background(),
		want.FloatValue,
		want.DoubleValue,
		want.Int64Value,
		want.Uint64Value,
		want.Int32Value,
		want.Fixed64Value,
		want.Fixed32Value,
		want.BoolValue,
		want.StringValue,
		want.Uint32Value,
		want.Sfixed32Value,
		want.Sfixed64Value,
		want.Sint32Value,
		want.Sint64Value,
		want.NonConventionalNameValue,
		want.EnumValue.String(),
		want.PathEnumValue.String(),
		want.NestedPathEnumValue.String(),
		want.EnumValueAnnotation.String(),
	)
	if err != nil {
		t.Errorf("cl.Create(%#v) failed with %v; want success", want, err)
	}
	if resp.Uuid == "" {
		t.Errorf("resp.Uuid is empty; want not empty")
	}
	resp.Uuid = ""

	if resp.FloatValue != want.FloatValue {
		t.Error("float")
	}
	if resp.DoubleValue != want.DoubleValue {
		t.Error("double")
	}
	if resp.Int64Value != want.Int64Value {
		t.Error("double")
	}
	if resp.Uint64Value != want.Uint64Value {
		t.Error("double")
	}
	if resp.Int32Value != want.Int32Value {
		t.Error("double")
	}
	if resp.Fixed32Value != want.Fixed32Value {
		t.Error("bool")
	}
	if resp.Fixed64Value != want.Fixed64Value {
		t.Error("bool")
	}
	if resp.BoolValue != want.BoolValue {
		t.Error("bool")
	}
	if resp.StringValue != want.StringValue {
		t.Error("bool")
	}
	if resp.Uint32Value != want.Uint32Value {
		t.Error("bool")
	}
	if resp.Sfixed32Value != want.Sfixed32Value {
		t.Error("bool")
	}
	if resp.Sfixed64Value != want.Sfixed64Value {
		t.Error("bool")
	}
	if resp.Sint32Value != want.Sint32Value {
		t.Error("bool")
	}
	if resp.Sint64Value != want.Sint64Value {
		t.Error("enum")
	}
	if resp.NonConventionalNameValue != want.NonConventionalNameValue {
		t.Error("enum")
	}
	if resp.EnumValue.String() != want.EnumValue.String() {
		t.Error("enum")
	}
	if resp.PathEnumValue.String() != want.PathEnumValue.String() {
		t.Error("path enum")
	}
	if resp.NestedPathEnumValue.String() != want.NestedPathEnumValue.String() {
		t.Error("nested path enum")
	}
	if resp.NestedPathEnumValue.String() != want.NestedPathEnumValue.String() {
		t.Error("nested path enum")
	}
}

func TestUnannotatedEchoClient(t *testing.T) {
	if testing.Short() {
		t.Skip()
		return
	}

	cfg := unannotatedecho.NewConfiguration()
	cfg.BasePath = "http://localhost:8088"

	cl := unannotatedecho.NewAPIClient(cfg)

	resp, _, err := cl.UnannotatedEchoServiceApi.UnannotatedEchoServiceEcho(context.Background(), "foo")
	if err != nil {
		t.Errorf(`cl.Echo("foo") failed with %v; want success`, err)
	}
	if got, want := resp.Id, "foo"; got != want {
		t.Errorf("resp.Id = %q; want %q", got, want)
	}
}

func TestUnannotatedEchoBodyClient(t *testing.T) {
	if testing.Short() {
		t.Skip()
		return
	}

	cfg := unannotatedecho.NewConfiguration()
	cfg.BasePath = "http://localhost:8088"

	cl := unannotatedecho.NewAPIClient(cfg)

	req := unannotatedecho.ExamplepbUnannotatedSimpleMessage{Id: "foo"}
	resp, _, err := cl.UnannotatedEchoServiceApi.UnannotatedEchoServiceEchoBody(context.Background(), req)
	if err != nil {
		t.Errorf("cl.EchoBody(%#v) failed with %v; want success", req, err)
	}
	if got, want := resp.Id, "foo"; got != want {
		t.Errorf("resp.Id = %q; want %q", got, want)
	}
}
