package tests

import "time"

//----------------------------------------
// Struct types

type EmptyStruct struct {
}

type PrimitivesStruct struct {
	Int8    int8
	Int16   int16
	Int32   int32
	Int64   int64
	Varint  int64 `binary:"varint"`
	Int     int
	Byte    byte
	Uint8   uint8
	Uint16  uint16
	Uint32  uint32
	Uint64  uint64
	Uvarint uint64 `binary:"varint"`
	Uint    uint
	String  string
	Bytes   []byte
	Time    time.Time
	Empty   EmptyStruct
}

type ShortArraysStruct struct {
	TimeAr [0]time.Time
}

type ArraysStruct struct {
	Int8Ar    [4]int8
	Int16Ar   [4]int16
	Int32Ar   [4]int32
	Int64Ar   [4]int64
	VarintAr  [4]int64 `binary:"varint"`
	IntAr     [4]int
	ByteAr    [4]byte
	Uint8Ar   [4]uint8
	Uint16Ar  [4]uint16
	Uint32Ar  [4]uint32
	Uint64Ar  [4]uint64
	UvarintAr [4]uint64 `binary:"varint"`
	UintAr    [4]uint
	StringAr  [4]string
	BytesAr   [4][]byte
	TimeAr    [4]time.Time
	EmptyAr   [4]EmptyStruct
}

type SlicesStruct struct {
	Int8Sl    []int8
	Int16Sl   []int16
	Int32Sl   []int32
	Int64Sl   []int64
	VarintSl  []int64 `binary:"varint"`
	IntSl     []int
	ByteSl    []byte
	Uint8Sl   []uint8
	Uint16Sl  []uint16
	Uint32Sl  []uint32
	Uint64Sl  []uint64
	UvarintSl []uint64 `binary:"varint"`
	UintSl    []uint
	StringSl  []string
	BytesSl   [][]byte
	TimeSl    []time.Time
	EmptySl   []EmptyStruct
}

type SliceSlicesStruct struct {
	Int8SlSl    [][]int8
	Int16SlSl   [][]int16
	Int32SlSl   [][]int32
	Int64SlSl   [][]int64
	VarintSlSl  [][]int64 `binary:"varint"`
	IntSlSl     [][]int
	ByteSlSl    [][]byte
	Uint8SlSl   [][]uint8
	Uint16SlSl  [][]uint16
	Uint32SlSl  [][]uint32
	Uint64SlSl  [][]uint64
	UvarintSlSl [][]uint64 `binary:"varint"`
	UintSlSl    [][]uint
	StringSlSl  [][]string
	BytesSlSl   [][][]byte
	TimeSlSl    [][]time.Time
	EmptySlSl   [][]EmptyStruct
}

type PointersStruct struct {
	Int8Pt    *int8
	Int16Pt   *int16
	Int32Pt   *int32
	Int64Pt   *int64
	VarintPt  *int64 `binary:"varint"`
	IntPt     *int
	BytePt    *byte
	Uint8Pt   *uint8
	Uint16Pt  *uint16
	Uint32Pt  *uint32
	Uint64Pt  *uint64
	UvarintPt *uint64 `binary:"varint"`
	UintPt    *uint
	StringPt  *string
	BytesPt   *[]byte
	TimePt    *time.Time
	EmptyPt   *EmptyStruct
}

type PointerSlicesStruct struct {
	Int8PtSl    []*int8
	Int16PtSl   []*int16
	Int32PtSl   []*int32
	Int64PtSl   []*int64
	VarintPtSl  []*int64 `binary:"varint"`
	IntPtSl     []*int
	BytePtSl    []*byte
	Uint8PtSl   []*uint8
	Uint16PtSl  []*uint16
	Uint32PtSl  []*uint32
	Uint64PtSl  []*uint64
	UvarintPtSl []*uint64 `binary:"varint"`
	UintPtSl    []*uint
	StringPtSl  []*string
	BytesPtSl   []*[]byte
	TimePtSl    []*time.Time
	EmptyPtSl   []*EmptyStruct
}

// NOTE: See registered fuzz funcs for *byte, **byte, and ***byte.
type NestedPointersStruct struct {
	Ptr1 *byte
	Ptr2 **byte
	Ptr3 ***byte
}

type ComplexSt struct {
	PrField PrimitivesStruct
	ArField ArraysStruct
	SlField SlicesStruct
	PtField PointersStruct
}

type EmbeddedSt1 struct {
	PrimitivesStruct
}

type EmbeddedSt2 struct {
	PrimitivesStruct
	ArraysStruct
	SlicesStruct
	PointersStruct
}

type EmbeddedSt3 struct {
	*PrimitivesStruct
	*ArraysStruct
	*SlicesStruct
	*PointersStruct
	*EmptyStruct
}

type EmbeddedSt4 struct {
	Foo1 int
	PrimitivesStruct
	Foo2              string
	ArraysStructField ArraysStruct
	Foo3              []byte
	SlicesStruct
	Foo4                bool
	PointersStructField PointersStruct
	Foo5                uint
}

type EmbeddedSt5 struct {
	Foo1 int
	*PrimitivesStruct
	Foo2              string
	ArraysStructField *ArraysStruct
	Foo3              []byte
	*SlicesStruct
	Foo4                bool
	PointersStructField *PointersStruct
	Foo5                uint
}

var StructTypes = []interface{}{
	(*EmptyStruct)(nil),
	(*PrimitivesStruct)(nil),
	(*ShortArraysStruct)(nil),
	(*ArraysStruct)(nil),
	(*SlicesStruct)(nil),
	(*SliceSlicesStruct)(nil),
	(*PointersStruct)(nil),
	(*PointerSlicesStruct)(nil),
	(*NestedPointersStruct)(nil),
	(*ComplexSt)(nil),
	(*EmbeddedSt1)(nil),
	(*EmbeddedSt2)(nil),
	(*EmbeddedSt3)(nil),
	(*EmbeddedSt4)(nil),
	(*EmbeddedSt5)(nil),
}

//----------------------------------------
// Type definition types

type IntDef int

type IntAr [4]int

type IntSl []int

type ByteAr [4]byte

type ByteSl []byte

type PrimitivesStructSl []PrimitivesStruct

type PrimitivesStructDef PrimitivesStruct

var DefTypes = []interface{}{
	(*IntDef)(nil),
	(*IntAr)(nil),
	(*IntSl)(nil),
	(*ByteAr)(nil),
	(*ByteSl)(nil),
	(*PrimitivesStructSl)(nil),
	(*PrimitivesStructDef)(nil),
}

//----------------------------------------
// Register/Interface test types

type Interface1 interface {
	AssertInterface1()
}

type Interface2 interface {
	AssertInterface2()
}

type Concrete1 struct{}

func (_ Concrete1) AssertInterface1() {}
func (_ Concrete1) AssertInterface2() {}

type Concrete2 struct{}

func (_ Concrete2) AssertInterface1() {}
func (_ Concrete2) AssertInterface2() {}

type Concrete3 [4]byte

func (_ Concrete3) AssertInterface1() {}

type InterfaceFieldsStruct struct {
	F1 Interface1
	F2 Interface1
}

func (_ *InterfaceFieldsStruct) AssertInterface1() {}
