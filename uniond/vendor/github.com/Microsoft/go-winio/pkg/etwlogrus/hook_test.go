//go:build windows
// +build windows

package etwlogrus

import (
	"testing"

	"github.com/sirupsen/logrus"
)

func fireEvent(name string, value interface{}) {
	logrus.WithField("Field", value).Info(name)
}

// The purpose of this test is to log lots of different field types, to test the
// logic that converts them to ETW. Because we don't have a way to
// programatically validate the ETW events, this test has two main purposes: (1)
// validate nothing causes a panic while logging (2) allow manual validation that
// the data is logged correctly (through a tool like WPA).
func TestFieldLogging(t *testing.T) {
	// Sample WPRP to collect this provider is included in HookTest.wprp.
	//
	// Start collection:
	// wpr -start HookTest.wprp -filemode
	//
	// Stop collection:
	// wpr -stop HookTest.etl
	h, err := NewHook("HookTest")
	if err != nil {
		t.Fatal(err)
	}
	logrus.AddHook(h)

	fireEvent("Bool", true)
	fireEvent("BoolSlice", []bool{true, false, true})
	fireEvent("EmptyBoolSlice", []bool{})
	fireEvent("String", "teststring")
	fireEvent("StringSlice", []string{"sstr1", "sstr2", "sstr3"})
	fireEvent("EmptyStringSlice", []string{})
	fireEvent("Int", int(1))
	fireEvent("IntSlice", []int{2, 3, 4})
	fireEvent("EmptyIntSlice", []int{})
	fireEvent("Int8", int8(5))
	fireEvent("Int8Slice", []int8{6, 7, 8})
	fireEvent("EmptyInt8Slice", []int8{})
	fireEvent("Int16", int16(9))
	fireEvent("Int16Slice", []int16{10, 11, 12})
	fireEvent("EmptyInt16Slice", []int16{})
	fireEvent("Int32", int32(13))
	fireEvent("Int32Slice", []int32{14, 15, 16})
	fireEvent("EmptyInt32Slice", []int32{})
	fireEvent("Int64", int64(17))
	fireEvent("Int64Slice", []int64{18, 19, 20})
	fireEvent("EmptyInt64Slice", []int64{})
	fireEvent("Uint", uint(21))
	fireEvent("UintSlice", []uint{22, 23, 24})
	fireEvent("EmptyUintSlice", []uint{})
	fireEvent("Uint8", uint8(25))
	fireEvent("Uint8Slice", []uint8{26, 27, 28})
	fireEvent("EmptyUint8Slice", []uint8{})
	fireEvent("Uint16", uint16(29))
	fireEvent("Uint16Slice", []uint16{30, 31, 32})
	fireEvent("EmptyUint16Slice", []uint16{})
	fireEvent("Uint32", uint32(33))
	fireEvent("Uint32Slice", []uint32{34, 35, 36})
	fireEvent("EmptyUint32Slice", []uint32{})
	fireEvent("Uint64", uint64(37))
	fireEvent("Uint64Slice", []uint64{38, 39, 40})
	fireEvent("EmptyUint64Slice", []uint64{})
	fireEvent("Uintptr", uintptr(41))
	fireEvent("UintptrSlice", []uintptr{42, 43, 44})
	fireEvent("EmptyUintptrSlice", []uintptr{})
	fireEvent("Float32", float32(45.46))
	fireEvent("Float32Slice", []float32{47.48, 49.50, 51.52})
	fireEvent("EmptyFloat32Slice", []float32{})
	fireEvent("Float64", float64(53.54))
	fireEvent("Float64Slice", []float64{55.56, 57.58, 59.60})
	fireEvent("EmptyFloat64Slice", []float64{})

	type struct1 struct {
		A    float32
		priv int
		B    []uint
	}
	type struct2 struct {
		A int
		B int
	}
	type struct3 struct {
		struct2
		A    int
		B    string
		priv string
		C    struct1
		D    uint16
	}
	// Unexported fields, and fields in embedded structs, should not log.
	fireEvent("Struct", struct3{struct2{-1, -2}, 1, "2s", "-3s", struct1{3.4, -4, []uint{5, 6, 7}}, 8})
}
