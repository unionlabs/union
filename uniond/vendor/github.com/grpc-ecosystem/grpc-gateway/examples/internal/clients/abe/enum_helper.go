package abe

import (
	pbexamplepb "github.com/grpc-ecosystem/grpc-gateway/examples/internal/proto/examplepb"
	pbpathenum "github.com/grpc-ecosystem/grpc-gateway/examples/internal/proto/pathenum"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"
)

// String returns a string representation of "NumericEnum"
func (e ExamplepbNumericEnum) String() string {
	return pbexamplepb.NumericEnum_ONE.String()
}

// UnmarshalJSON does a no-op unmarshal to ExamplepbNumericEnum.
// It just validates that the input is sane.
func (e ExamplepbNumericEnum) UnmarshalJSON(b []byte) error {
	return unmarshalJSONEnum(b, pbexamplepb.NumericEnum_value)
}

// String returns a string representation of "MessagePathEnum"
func (e MessagePathEnumNestedPathEnum) String() string {
	return pbpathenum.MessagePathEnum_JKL.String()
}

// UnmarshalJSON does a no-op unmarshal to MessagePathEnumNestedPathEnum.
// It just validates that the input is sane.
func (e MessagePathEnumNestedPathEnum) UnmarshalJSON(b []byte) error {
	return unmarshalJSONEnum(b, pbpathenum.MessagePathEnum_NestedPathEnum_value)
}

// String returns a string representation of "PathEnum"
func (e PathenumPathEnum) String() string {
	return pbpathenum.PathEnum_DEF.String()
}

// UnmarshalJSON does a no-op unmarshal to PathenumPathEnum.
// It just validates that the input is sane.
func (e PathenumPathEnum) UnmarshalJSON(b []byte) error {
	return unmarshalJSONEnum(b, pbpathenum.PathEnum_value)
}

func unmarshalJSONEnum(b []byte, enumValMap map[string]int32) error {
	val := string(b[1 : len(b)-1])
	_, err := runtime.Enum(val, enumValMap)
	return err
}
