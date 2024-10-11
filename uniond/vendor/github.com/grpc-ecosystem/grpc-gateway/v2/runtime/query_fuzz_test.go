//go:build go1.18
// +build go1.18

package runtime_test

import (
	"net/url"
	"testing"

	"github.com/grpc-ecosystem/grpc-gateway/v2/runtime"
	"github.com/grpc-ecosystem/grpc-gateway/v2/runtime/internal/examplepb"
	"github.com/grpc-ecosystem/grpc-gateway/v2/utilities"
)

func FuzzPopulateQueryParameters(f *testing.F) {
	f.Add("bool_value=true&bytes_value=YWJjMTIzIT8kKiYoKSctPUB-&double_value=2.5&duration_value=13h0m0s&enum_value=1&fieldmask_value=float_value%2Cdouble_value&float_value=1.5&int32_value=-2&int64_value=-1&map_value10%5Bkey%5D=1.5&map_value11%5B1.5%5D=value&map_value12%5Bkey%5D=2.5&map_value13%5B2.5%5D=value&map_value14%5Bkey%5D=true&map_value15%5Btrue%5D=value&map_value16%5Bkey%5D=2&map_value2%5Bkey%5D=-2&map_value3%5B-2%5D=value&map_value4%5Bkey%5D=-1&map_value5%5B-1%5D=value&map_value6%5Bkey%5D=3&map_value7%5B3%5D=value&map_value8%5Bkey%5D=4&map_value9%5B4%5D=value&map_value%5Bfourth%5D=&map_value%5Bkey%5D=value&map_value%5Bsecond%5D=bar&map_value%5Bthird%5D=zzz&map_value%5B~%21%40%23%24%25%5E%26%2A%28%29%5D=value&repeated_enum=1&repeated_enum=2&repeated_enum=0&repeated_message=1&repeated_message=2&repeated_message=3&repeated_value=a&repeated_value=b&repeated_value=c&string_value=str&struct_value=%7B%22a%22%3A%7B%22b%22%3A1%7D%7D&struct_value_value=%7B%22a%22%3A%7B%22b%22%3A1%7D%7D&timestamp_value=2016-12-15T12%3A23%3A32.000000049Z&uint32_value=4&uint64_value=3&wrapper_bool_value=true&wrapper_bytes_value=YWJjMTIzIT8kKiYoKSctPUB-&wrapper_double_value=2.5&wrapper_float_value=1.5&wrapper_int32_value=-2&wrapper_int64_value=-1&wrapper_string_value=str&wrapper_u_int32_value=4&wrapper_u_int64_value=3")
	f.Add("boolValue=true&bytesValue=Ynl0ZXM%3D&doubleValue=2.5&durationValue=13h0m0s&enumValue=1&fieldmaskValue=float_value%2Cdouble_value&floatValue=1.5&int32Value=-2&int64Value=-1&repeatedEnum=1&repeatedEnum=2&repeatedEnum=0&repeatedValue=a&repeatedValue=b&repeatedValue=c&stringValue=str&struct_value=%7B%7D&struct_value_value=%22%22&timestampValue=2016-12-15T12%3A23%3A32.000000049Z&uint32Value=4&uint64Value=3&wrapperBoolValue=true&wrapperBytesValue=Ynl0ZXM%3D&wrapperDoubleValue=2.5&wrapperFloatValue=1.5&wrapperInt32Value=-2&wrapperInt64Value=-1&wrapperStringValue=str&wrapperUInt32Value=4&wrapperUInt64Value=3")
	f.Add("enum_value=Z&repeated_enum=X&repeated_enum=2&repeated_enum=0&struct_value=%7B%22c%22%3A%5B1%2C2%5D%2C%22d%22%3A%5B%7B%22e%22%3A1%2C%22f%22%3A%7B%7D%7D%5D%7D&struct_value_value=%7B%7D")
	f.Add("struct_value_value=%5B%5D")
	f.Add("bool_value=true&double_value=2.5&float_value=1.5&int32_value=-2&int64_value=-1&repeated_value=a&repeated_value=b&repeated_value=c&string_value=str&uint32_value=4&uint64_value=3")
	f.Add("boolValue=true&doubleValue=2.5&floatValue=1.5&int32Value=-2&int64Value=-1&repeatedValue=a&repeatedValue=b&repeatedValue=c&stringValue=str&uint32Value=4&uint64Value=")
	f.Add("nested.nested.map_value%5Bfirst%5D=foo&nested.nested.map_value%5Bsecond%5D=bar&nested.nested.nested.repeated_value=a&nested.nested.nested.repeated_value=b&nested.nested.nested.repeated_value=c&nested.nested.nested.string_value=s&nested.nested.string_value=t&nested.string_value=u")
	f.Add("oneof_string_value=foobar")
	f.Add("nested_oneof_value_one.int64Value=-1&nested_oneof_value_one.string_value=foo")
	f.Fuzz(func(t *testing.T, query string) {
		in := &examplepb.ABitOfEverything{}
		values, err := url.ParseQuery(query)
		if err != nil {
			return
		}
		err = runtime.PopulateQueryParameters(in, values, utilities.NewDoubleArray(nil))
		if err != nil {
			return
		}
	})
}
