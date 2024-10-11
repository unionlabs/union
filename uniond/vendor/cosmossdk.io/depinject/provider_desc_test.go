package depinject

import (
	"reflect"
	"testing"

	"gotest.tools/v3/assert"

	"cosmossdk.io/depinject/internal/codegen"
	"cosmossdk.io/depinject/internal/graphviz"
)

type StructIn struct {
	In
	X int
	Y float64 `optional:"true"`
}

type BadOptional struct {
	In
	X int `optional:"foo"`
}

type StructOut struct {
	Out
	X string
	Y []byte
}

func privateProvider(int, float64) (string, []byte) { return "", nil }

func PrivateInAndOut(containerConfig) *container { return nil } //revive:disable:unexported-return

func InternalInAndOut(graphviz.Attributes) *codegen.FileGen { return nil }

type SomeStruct struct{}

func (SomeStruct) privateMethod() int { return 0 }

func SimpleArgs(int, float64) (string, []byte) { return "", nil }

func SimpleArgsWithError(int, float64) (string, []byte, error) { return "", nil, nil }

func StructInAndOut(_ float32, _ StructIn, _ byte) (int16, StructOut, int32, error) {
	return int16(0), StructOut{}, int32(0), nil
}

func BadErrorPosition() (error, int) { return nil, 0 } //nolint:stylecheck // Deliberately has error as first of multiple arguments.

func BadOptionalFn(_ BadOptional) int { return 0 }

func Variadic(...float64) int { return 0 }

func TestExtractProviderDescriptor(t *testing.T) {
	var (
		intType     = reflect.TypeOf(0)
		int16Type   = reflect.TypeOf(int16(0))
		int32Type   = reflect.TypeOf(int32(0))
		float32Type = reflect.TypeOf(float32(0.0))
		float64Type = reflect.TypeOf(0.0)
		stringType  = reflect.TypeOf("")
		byteTyp     = reflect.TypeOf(byte(0))
		bytesTyp    = reflect.TypeOf([]byte{})
	)

	tests := []struct {
		name    string
		ctr     interface{}
		wantIn  []providerInput
		wantOut []providerOutput
		wantErr string
	}{
		{
			"private",
			privateProvider,
			nil,
			nil,
			"function must be exported",
		},
		{
			"private method",
			SomeStruct.privateMethod,
			nil,
			nil,
			"function must be exported",
		},
		{
			"private in and out",
			PrivateInAndOut,
			nil,
			nil,
			"type must be exported",
		},
		{
			"internal in and out",
			InternalInAndOut,
			nil,
			nil,
			"internal",
		},
		{
			"struct",
			SomeStruct{},
			nil,
			nil,
			"expected a Func type",
		},
		{
			"simple args",
			SimpleArgs,
			[]providerInput{{Type: intType}, {Type: float64Type}},
			[]providerOutput{{Type: stringType}, {Type: bytesTyp}},
			"",
		},
		{
			"simple args with error",
			SimpleArgsWithError,
			[]providerInput{{Type: intType}, {Type: float64Type}},
			[]providerOutput{{Type: stringType}, {Type: bytesTyp}},
			"",
		},
		{
			"struct in and out",
			StructInAndOut,
			[]providerInput{{Type: float32Type}, {Type: intType}, {Type: float64Type, Optional: true}, {Type: byteTyp}},
			[]providerOutput{{Type: int16Type}, {Type: stringType}, {Type: bytesTyp}, {Type: int32Type}},
			"",
		},
		{
			"error bad position",
			BadErrorPosition,
			nil,
			nil,
			"error parameter is not last parameter",
		},
		{
			"bad optional",
			BadOptionalFn,
			nil,
			nil,
			"bad optional tag",
		},
		{
			"variadic",
			Variadic,
			nil,
			nil,
			"variadic function can't be used",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := extractProviderDescriptor(tt.ctr)
			if tt.wantErr != "" {
				assert.ErrorContains(t, err, tt.wantErr)
			} else {
				assert.NilError(t, err)

				if !reflect.DeepEqual(got.Inputs, tt.wantIn) {
					t.Errorf("extractProviderDescriptor() got = %v, want %v", got.Inputs, tt.wantIn)
				}
				if !reflect.DeepEqual(got.Outputs, tt.wantOut) {
					t.Errorf("extractProviderDescriptor() got = %v, want %v", got.Outputs, tt.wantOut)
				}
			}
		})
	}
}
