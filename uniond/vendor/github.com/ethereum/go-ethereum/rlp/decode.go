// Copyright 2014 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package rlp

import (
	"bufio"
	"bytes"
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"math/big"
	"reflect"
	"strings"
	"sync"

	"github.com/ethereum/go-ethereum/rlp/internal/rlpstruct"
	"github.com/holiman/uint256"
)

//lint:ignore ST1012 EOL is not an error.

// EOL is returned when the end of the current list
// has been reached during streaming.
var EOL = errors.New("rlp: end of list")

var (
	ErrExpectedString   = errors.New("rlp: expected String or Byte")
	ErrExpectedList     = errors.New("rlp: expected List")
	ErrCanonInt         = errors.New("rlp: non-canonical integer format")
	ErrCanonSize        = errors.New("rlp: non-canonical size information")
	ErrElemTooLarge     = errors.New("rlp: element is larger than containing list")
	ErrValueTooLarge    = errors.New("rlp: value size exceeds available input length")
	ErrMoreThanOneValue = errors.New("rlp: input contains more than one value")

	// internal errors
	errNotInList     = errors.New("rlp: call of ListEnd outside of any list")
	errNotAtEOL      = errors.New("rlp: call of ListEnd not positioned at EOL")
	errUintOverflow  = errors.New("rlp: uint overflow")
	errNoPointer     = errors.New("rlp: interface given to Decode must be a pointer")
	errDecodeIntoNil = errors.New("rlp: pointer given to Decode must not be nil")
	errUint256Large  = errors.New("rlp: value too large for uint256")

	streamPool = sync.Pool{
		New: func() interface{} { return new(Stream) },
	}
)

// Decoder is implemented by types that require custom RLP decoding rules or need to decode
// into private fields.
//
// The DecodeRLP method should read one value from the given Stream. It is not forbidden to
// read less or more, but it might be confusing.
type Decoder interface {
	DecodeRLP(*Stream) error
}

// Decode parses RLP-encoded data from r and stores the result in the value pointed to by
// val. Please see package-level documentation for the decoding rules. Val must be a
// non-nil pointer.
//
// If r does not implement ByteReader, Decode will do its own buffering.
//
// Note that Decode does not set an input limit for all readers and may be vulnerable to
// panics cause by huge value sizes. If you need an input limit, use
//
//	NewStream(r, limit).Decode(val)
func Decode(r io.Reader, val interface{}) error {
	stream := streamPool.Get().(*Stream)
	defer streamPool.Put(stream)

	stream.Reset(r, 0)
	return stream.Decode(val)
}

// DecodeBytes parses RLP data from b into val. Please see package-level documentation for
// the decoding rules. The input must contain exactly one value and no trailing data.
func DecodeBytes(b []byte, val interface{}) error {
	r := (*sliceReader)(&b)

	stream := streamPool.Get().(*Stream)
	defer streamPool.Put(stream)

	stream.Reset(r, uint64(len(b)))
	if err := stream.Decode(val); err != nil {
		return err
	}
	if len(b) > 0 {
		return ErrMoreThanOneValue
	}
	return nil
}

type decodeError struct {
	msg string
	typ reflect.Type
	ctx []string
}

func (err *decodeError) Error() string {
	ctx := ""
	if len(err.ctx) > 0 {
		ctx = ", decoding into "
		for i := len(err.ctx) - 1; i >= 0; i-- {
			ctx += err.ctx[i]
		}
	}
	return fmt.Sprintf("rlp: %s for %v%s", err.msg, err.typ, ctx)
}

func wrapStreamError(err error, typ reflect.Type) error {
	switch err {
	case ErrCanonInt:
		return &decodeError{msg: "non-canonical integer (leading zero bytes)", typ: typ}
	case ErrCanonSize:
		return &decodeError{msg: "non-canonical size information", typ: typ}
	case ErrExpectedList:
		return &decodeError{msg: "expected input list", typ: typ}
	case ErrExpectedString:
		return &decodeError{msg: "expected input string or byte", typ: typ}
	case errUintOverflow:
		return &decodeError{msg: "input string too long", typ: typ}
	case errNotAtEOL:
		return &decodeError{msg: "input list has too many elements", typ: typ}
	}
	return err
}

func addErrorContext(err error, ctx string) error {
	if decErr, ok := err.(*decodeError); ok {
		decErr.ctx = append(decErr.ctx, ctx)
	}
	return err
}

var (
	decoderInterface = reflect.TypeOf(new(Decoder)).Elem()
	bigInt           = reflect.TypeOf(big.Int{})
	u256Int          = reflect.TypeOf(uint256.Int{})
)

func makeDecoder(typ reflect.Type, tags rlpstruct.Tags) (dec decoder, err error) {
	kind := typ.Kind()
	switch {
	case typ == rawValueType:
		return decodeRawValue, nil
	case typ.AssignableTo(reflect.PtrTo(bigInt)):
		return decodeBigInt, nil
	case typ.AssignableTo(bigInt):
		return decodeBigIntNoPtr, nil
	case typ == reflect.PtrTo(u256Int):
		return decodeU256, nil
	case typ == u256Int:
		return decodeU256NoPtr, nil
	case kind == reflect.Ptr:
		return makePtrDecoder(typ, tags)
	case reflect.PtrTo(typ).Implements(decoderInterface):
		return decodeDecoder, nil
	case isUint(kind):
		return decodeUint, nil
	case kind == reflect.Bool:
		return decodeBool, nil
	case kind == reflect.String:
		return decodeString, nil
	case kind == reflect.Slice || kind == reflect.Array:
		return makeListDecoder(typ, tags)
	case kind == reflect.Struct:
		return makeStructDecoder(typ)
	case kind == reflect.Interface:
		return decodeInterface, nil
	default:
		return nil, fmt.Errorf("rlp: type %v is not RLP-serializable", typ)
	}
}

func decodeRawValue(s *Stream, val reflect.Value) error {
	r, err := s.Raw()
	if err != nil {
		return err
	}
	val.SetBytes(r)
	return nil
}

func decodeUint(s *Stream, val reflect.Value) error {
	typ := val.Type()
	num, err := s.uint(typ.Bits())
	if err != nil {
		return wrapStreamError(err, val.Type())
	}
	val.SetUint(num)
	return nil
}

func decodeBool(s *Stream, val reflect.Value) error {
	b, err := s.Bool()
	if err != nil {
		return wrapStreamError(err, val.Type())
	}
	val.SetBool(b)
	return nil
}

func decodeString(s *Stream, val reflect.Value) error {
	b, err := s.Bytes()
	if err != nil {
		return wrapStreamError(err, val.Type())
	}
	val.SetString(string(b))
	return nil
}

func decodeBigIntNoPtr(s *Stream, val reflect.Value) error {
	return decodeBigInt(s, val.Addr())
}

func decodeBigInt(s *Stream, val reflect.Value) error {
	i := val.Interface().(*big.Int)
	if i == nil {
		i = new(big.Int)
		val.Set(reflect.ValueOf(i))
	}

	err := s.decodeBigInt(i)
	if err != nil {
		return wrapStreamError(err, val.Type())
	}
	return nil
}

func decodeU256NoPtr(s *Stream, val reflect.Value) error {
	return decodeU256(s, val.Addr())
}

func decodeU256(s *Stream, val reflect.Value) error {
	i := val.Interface().(*uint256.Int)
	if i == nil {
		i = new(uint256.Int)
		val.Set(reflect.ValueOf(i))
	}

	err := s.ReadUint256(i)
	if err != nil {
		return wrapStreamError(err, val.Type())
	}
	return nil
}

func makeListDecoder(typ reflect.Type, tag rlpstruct.Tags) (decoder, error) {
	etype := typ.Elem()
	if etype.Kind() == reflect.Uint8 && !reflect.PtrTo(etype).Implements(decoderInterface) {
		if typ.Kind() == reflect.Array {
			return decodeByteArray, nil
		}
		return decodeByteSlice, nil
	}
	etypeinfo := theTC.infoWhileGenerating(etype, rlpstruct.Tags{})
	if etypeinfo.decoderErr != nil {
		return nil, etypeinfo.decoderErr
	}
	var dec decoder
	switch {
	case typ.Kind() == reflect.Array:
		dec = func(s *Stream, val reflect.Value) error {
			return decodeListArray(s, val, etypeinfo.decoder)
		}
	case tag.Tail:
		// A slice with "tail" tag can occur as the last field
		// of a struct and is supposed to swallow all remaining
		// list elements. The struct decoder already called s.List,
		// proceed directly to decoding the elements.
		dec = func(s *Stream, val reflect.Value) error {
			return decodeSliceElems(s, val, etypeinfo.decoder)
		}
	default:
		dec = func(s *Stream, val reflect.Value) error {
			return decodeListSlice(s, val, etypeinfo.decoder)
		}
	}
	return dec, nil
}

func decodeListSlice(s *Stream, val reflect.Value, elemdec decoder) error {
	size, err := s.List()
	if err != nil {
		return wrapStreamError(err, val.Type())
	}
	if size == 0 {
		val.Set(reflect.MakeSlice(val.Type(), 0, 0))
		return s.ListEnd()
	}
	if err := decodeSliceElems(s, val, elemdec); err != nil {
		return err
	}
	return s.ListEnd()
}

func decodeSliceElems(s *Stream, val reflect.Value, elemdec decoder) error {
	i := 0
	for ; ; i++ {
		// grow slice if necessary
		if i >= val.Cap() {
			newcap := val.Cap() + val.Cap()/2
			if newcap < 4 {
				newcap = 4
			}
			newv := reflect.MakeSlice(val.Type(), val.Len(), newcap)
			reflect.Copy(newv, val)
			val.Set(newv)
		}
		if i >= val.Len() {
			val.SetLen(i + 1)
		}
		// decode into element
		if err := elemdec(s, val.Index(i)); err == EOL {
			break
		} else if err != nil {
			return addErrorContext(err, fmt.Sprint("[", i, "]"))
		}
	}
	if i < val.Len() {
		val.SetLen(i)
	}
	return nil
}

func decodeListArray(s *Stream, val reflect.Value, elemdec decoder) error {
	if _, err := s.List(); err != nil {
		return wrapStreamError(err, val.Type())
	}
	vlen := val.Len()
	i := 0
	for ; i < vlen; i++ {
		if err := elemdec(s, val.Index(i)); err == EOL {
			break
		} else if err != nil {
			return addErrorContext(err, fmt.Sprint("[", i, "]"))
		}
	}
	if i < vlen {
		return &decodeError{msg: "input list has too few elements", typ: val.Type()}
	}
	return wrapStreamError(s.ListEnd(), val.Type())
}

func decodeByteSlice(s *Stream, val reflect.Value) error {
	b, err := s.Bytes()
	if err != nil {
		return wrapStreamError(err, val.Type())
	}
	val.SetBytes(b)
	return nil
}

func decodeByteArray(s *Stream, val reflect.Value) error {
	kind, size, err := s.Kind()
	if err != nil {
		return err
	}
	slice := byteArrayBytes(val, val.Len())
	switch kind {
	case Byte:
		if len(slice) == 0 {
			return &decodeError{msg: "input string too long", typ: val.Type()}
		} else if len(slice) > 1 {
			return &decodeError{msg: "input string too short", typ: val.Type()}
		}
		slice[0] = s.byteval
		s.kind = -1
	case String:
		if uint64(len(slice)) < size {
			return &decodeError{msg: "input string too long", typ: val.Type()}
		}
		if uint64(len(slice)) > size {
			return &decodeError{msg: "input string too short", typ: val.Type()}
		}
		if err := s.readFull(slice); err != nil {
			return err
		}
		// Reject cases where single byte encoding should have been used.
		if size == 1 && slice[0] < 128 {
			return wrapStreamError(ErrCanonSize, val.Type())
		}
	case List:
		return wrapStreamError(ErrExpectedString, val.Type())
	}
	return nil
}

func makeStructDecoder(typ reflect.Type) (decoder, error) {
	fields, err := structFields(typ)
	if err != nil {
		return nil, err
	}
	for _, f := range fields {
		if f.info.decoderErr != nil {
			return nil, structFieldError{typ, f.index, f.info.decoderErr}
		}
	}
	dec := func(s *Stream, val reflect.Value) (err error) {
		if _, err := s.List(); err != nil {
			return wrapStreamError(err, typ)
		}
		for i, f := range fields {
			err := f.info.decoder(s, val.Field(f.index))
			if err == EOL {
				if f.optional {
					// The field is optional, so reaching the end of the list before
					// reaching the last field is acceptable. All remaining undecoded
					// fields are zeroed.
					zeroFields(val, fields[i:])
					break
				}
				return &decodeError{msg: "too few elements", typ: typ}
			} else if err != nil {
				return addErrorContext(err, "."+typ.Field(f.index).Name)
			}
		}
		return wrapStreamError(s.ListEnd(), typ)
	}
	return dec, nil
}

func zeroFields(structval reflect.Value, fields []field) {
	for _, f := range fields {
		fv := structval.Field(f.index)
		fv.Set(reflect.Zero(fv.Type()))
	}
}

// makePtrDecoder creates a decoder that decodes into the pointer's element type.
func makePtrDecoder(typ reflect.Type, tag rlpstruct.Tags) (decoder, error) {
	etype := typ.Elem()
	etypeinfo := theTC.infoWhileGenerating(etype, rlpstruct.Tags{})
	switch {
	case etypeinfo.decoderErr != nil:
		return nil, etypeinfo.decoderErr
	case !tag.NilOK:
		return makeSimplePtrDecoder(etype, etypeinfo), nil
	default:
		return makeNilPtrDecoder(etype, etypeinfo, tag), nil
	}
}

func makeSimplePtrDecoder(etype reflect.Type, etypeinfo *typeinfo) decoder {
	return func(s *Stream, val reflect.Value) (err error) {
		newval := val
		if val.IsNil() {
			newval = reflect.New(etype)
		}
		if err = etypeinfo.decoder(s, newval.Elem()); err == nil {
			val.Set(newval)
		}
		return err
	}
}

// makeNilPtrDecoder creates a decoder that decodes empty values as nil. Non-empty
// values are decoded into a value of the element type, just like makePtrDecoder does.
//
// This decoder is used for pointer-typed struct fields with struct tag "nil".
func makeNilPtrDecoder(etype reflect.Type, etypeinfo *typeinfo, ts rlpstruct.Tags) decoder {
	typ := reflect.PtrTo(etype)
	nilPtr := reflect.Zero(typ)

	// Determine the value kind that results in nil pointer.
	nilKind := typeNilKind(etype, ts)

	return func(s *Stream, val reflect.Value) (err error) {
		kind, size, err := s.Kind()
		if err != nil {
			val.Set(nilPtr)
			return wrapStreamError(err, typ)
		}
		// Handle empty values as a nil pointer.
		if kind != Byte && size == 0 {
			if kind != nilKind {
				return &decodeError{
					msg: fmt.Sprintf("wrong kind of empty value (got %v, want %v)", kind, nilKind),
					typ: typ,
				}
			}
			// rearm s.Kind. This is important because the input
			// position must advance to the next value even though
			// we don't read anything.
			s.kind = -1
			val.Set(nilPtr)
			return nil
		}
		newval := val
		if val.IsNil() {
			newval = reflect.New(etype)
		}
		if err = etypeinfo.decoder(s, newval.Elem()); err == nil {
			val.Set(newval)
		}
		return err
	}
}

var ifsliceType = reflect.TypeOf([]interface{}{})

func decodeInterface(s *Stream, val reflect.Value) error {
	if val.Type().NumMethod() != 0 {
		return fmt.Errorf("rlp: type %v is not RLP-serializable", val.Type())
	}
	kind, _, err := s.Kind()
	if err != nil {
		return err
	}
	if kind == List {
		slice := reflect.New(ifsliceType).Elem()
		if err := decodeListSlice(s, slice, decodeInterface); err != nil {
			return err
		}
		val.Set(slice)
	} else {
		b, err := s.Bytes()
		if err != nil {
			return err
		}
		val.Set(reflect.ValueOf(b))
	}
	return nil
}

func decodeDecoder(s *Stream, val reflect.Value) error {
	return val.Addr().Interface().(Decoder).DecodeRLP(s)
}

// Kind represents the kind of value contained in an RLP stream.
type Kind int8

const (
	Byte Kind = iota
	String
	List
)

func (k Kind) String() string {
	switch k {
	case Byte:
		return "Byte"
	case String:
		return "String"
	case List:
		return "List"
	default:
		return fmt.Sprintf("Unknown(%d)", k)
	}
}

// ByteReader must be implemented by any input reader for a Stream. It
// is implemented by e.g. bufio.Reader and bytes.Reader.
type ByteReader interface {
	io.Reader
	io.ByteReader
}

// Stream can be used for piecemeal decoding of an input stream. This
// is useful if the input is very large or if the decoding rules for a
// type depend on the input structure. Stream does not keep an
// internal buffer. After decoding a value, the input reader will be
// positioned just before the type information for the next value.
//
// When decoding a list and the input position reaches the declared
// length of the list, all operations will return error EOL.
// The end of the list must be acknowledged using ListEnd to continue
// reading the enclosing list.
//
// Stream is not safe for concurrent use.
type Stream struct {
	r ByteReader

	remaining uint64   // number of bytes remaining to be read from r
	size      uint64   // size of value ahead
	kinderr   error    // error from last readKind
	stack     []uint64 // list sizes
	uintbuf   [32]byte // auxiliary buffer for integer decoding
	kind      Kind     // kind of value ahead
	byteval   byte     // value of single byte in type tag
	limited   bool     // true if input limit is in effect
}

// NewStream creates a new decoding stream reading from r.
//
// If r implements the ByteReader interface, Stream will
// not introduce any buffering.
//
// For non-toplevel values, Stream returns ErrElemTooLarge
// for values that do not fit into the enclosing list.
//
// Stream supports an optional input limit. If a limit is set, the
// size of any toplevel value will be checked against the remaining
// input length. Stream operations that encounter a value exceeding
// the remaining input length will return ErrValueTooLarge. The limit
// can be set by passing a non-zero value for inputLimit.
//
// If r is a bytes.Reader or strings.Reader, the input limit is set to
// the length of r's underlying data unless an explicit limit is
// provided.
func NewStream(r io.Reader, inputLimit uint64) *Stream {
	s := new(Stream)
	s.Reset(r, inputLimit)
	return s
}

// NewListStream creates a new stream that pretends to be positioned
// at an encoded list of the given length.
func NewListStream(r io.Reader, len uint64) *Stream {
	s := new(Stream)
	s.Reset(r, len)
	s.kind = List
	s.size = len
	return s
}

// Bytes reads an RLP string and returns its contents as a byte slice.
// If the input does not contain an RLP string, the returned
// error will be ErrExpectedString.
func (s *Stream) Bytes() ([]byte, error) {
	kind, size, err := s.Kind()
	if err != nil {
		return nil, err
	}
	switch kind {
	case Byte:
		s.kind = -1 // rearm Kind
		return []byte{s.byteval}, nil
	case String:
		b := make([]byte, size)
		if err = s.readFull(b); err != nil {
			return nil, err
		}
		if size == 1 && b[0] < 128 {
			return nil, ErrCanonSize
		}
		return b, nil
	default:
		return nil, ErrExpectedString
	}
}

// ReadBytes decodes the next RLP value and stores the result in b.
// The value size must match len(b) exactly.
func (s *Stream) ReadBytes(b []byte) error {
	kind, size, err := s.Kind()
	if err != nil {
		return err
	}
	switch kind {
	case Byte:
		if len(b) != 1 {
			return fmt.Errorf("input value has wrong size 1, want %d", len(b))
		}
		b[0] = s.byteval
		s.kind = -1 // rearm Kind
		return nil
	case String:
		if uint64(len(b)) != size {
			return fmt.Errorf("input value has wrong size %d, want %d", size, len(b))
		}
		if err = s.readFull(b); err != nil {
			return err
		}
		if size == 1 && b[0] < 128 {
			return ErrCanonSize
		}
		return nil
	default:
		return ErrExpectedString
	}
}

// Raw reads a raw encoded value including RLP type information.
func (s *Stream) Raw() ([]byte, error) {
	kind, size, err := s.Kind()
	if err != nil {
		return nil, err
	}
	if kind == Byte {
		s.kind = -1 // rearm Kind
		return []byte{s.byteval}, nil
	}
	// The original header has already been read and is no longer
	// available. Read content and put a new header in front of it.
	start := headsize(size)
	buf := make([]byte, uint64(start)+size)
	if err := s.readFull(buf[start:]); err != nil {
		return nil, err
	}
	if kind == String {
		puthead(buf, 0x80, 0xB7, size)
	} else {
		puthead(buf, 0xC0, 0xF7, size)
	}
	return buf, nil
}

// Uint reads an RLP string of up to 8 bytes and returns its contents
// as an unsigned integer. If the input does not contain an RLP string, the
// returned error will be ErrExpectedString.
//
// Deprecated: use s.Uint64 instead.
func (s *Stream) Uint() (uint64, error) {
	return s.uint(64)
}

func (s *Stream) Uint64() (uint64, error) {
	return s.uint(64)
}

func (s *Stream) Uint32() (uint32, error) {
	i, err := s.uint(32)
	return uint32(i), err
}

func (s *Stream) Uint16() (uint16, error) {
	i, err := s.uint(16)
	return uint16(i), err
}

func (s *Stream) Uint8() (uint8, error) {
	i, err := s.uint(8)
	return uint8(i), err
}

func (s *Stream) uint(maxbits int) (uint64, error) {
	kind, size, err := s.Kind()
	if err != nil {
		return 0, err
	}
	switch kind {
	case Byte:
		if s.byteval == 0 {
			return 0, ErrCanonInt
		}
		s.kind = -1 // rearm Kind
		return uint64(s.byteval), nil
	case String:
		if size > uint64(maxbits/8) {
			return 0, errUintOverflow
		}
		v, err := s.readUint(byte(size))
		switch {
		case err == ErrCanonSize:
			// Adjust error because we're not reading a size right now.
			return 0, ErrCanonInt
		case err != nil:
			return 0, err
		case size > 0 && v < 128:
			return 0, ErrCanonSize
		default:
			return v, nil
		}
	default:
		return 0, ErrExpectedString
	}
}

// Bool reads an RLP string of up to 1 byte and returns its contents
// as a boolean. If the input does not contain an RLP string, the
// returned error will be ErrExpectedString.
func (s *Stream) Bool() (bool, error) {
	num, err := s.uint(8)
	if err != nil {
		return false, err
	}
	switch num {
	case 0:
		return false, nil
	case 1:
		return true, nil
	default:
		return false, fmt.Errorf("rlp: invalid boolean value: %d", num)
	}
}

// List starts decoding an RLP list. If the input does not contain a
// list, the returned error will be ErrExpectedList. When the list's
// end has been reached, any Stream operation will return EOL.
func (s *Stream) List() (size uint64, err error) {
	kind, size, err := s.Kind()
	if err != nil {
		return 0, err
	}
	if kind != List {
		return 0, ErrExpectedList
	}

	// Remove size of inner list from outer list before pushing the new size
	// onto the stack. This ensures that the remaining outer list size will
	// be correct after the matching call to ListEnd.
	if inList, limit := s.listLimit(); inList {
		s.stack[len(s.stack)-1] = limit - size
	}
	s.stack = append(s.stack, size)
	s.kind = -1
	s.size = 0
	return size, nil
}

// ListEnd returns to the enclosing list.
// The input reader must be positioned at the end of a list.
func (s *Stream) ListEnd() error {
	// Ensure that no more data is remaining in the current list.
	if inList, listLimit := s.listLimit(); !inList {
		return errNotInList
	} else if listLimit > 0 {
		return errNotAtEOL
	}
	s.stack = s.stack[:len(s.stack)-1] // pop
	s.kind = -1
	s.size = 0
	return nil
}

// MoreDataInList reports whether the current list context contains
// more data to be read.
func (s *Stream) MoreDataInList() bool {
	_, listLimit := s.listLimit()
	return listLimit > 0
}

// BigInt decodes an arbitrary-size integer value.
func (s *Stream) BigInt() (*big.Int, error) {
	i := new(big.Int)
	if err := s.decodeBigInt(i); err != nil {
		return nil, err
	}
	return i, nil
}

func (s *Stream) decodeBigInt(dst *big.Int) error {
	var buffer []byte
	kind, size, err := s.Kind()
	switch {
	case err != nil:
		return err
	case kind == List:
		return ErrExpectedString
	case kind == Byte:
		buffer = s.uintbuf[:1]
		buffer[0] = s.byteval
		s.kind = -1 // re-arm Kind
	case size == 0:
		// Avoid zero-length read.
		s.kind = -1
	case size <= uint64(len(s.uintbuf)):
		// For integers smaller than s.uintbuf, allocating a buffer
		// can be avoided.
		buffer = s.uintbuf[:size]
		if err := s.readFull(buffer); err != nil {
			return err
		}
		// Reject inputs where single byte encoding should have been used.
		if size == 1 && buffer[0] < 128 {
			return ErrCanonSize
		}
	default:
		// For large integers, a temporary buffer is needed.
		buffer = make([]byte, size)
		if err := s.readFull(buffer); err != nil {
			return err
		}
	}

	// Reject leading zero bytes.
	if len(buffer) > 0 && buffer[0] == 0 {
		return ErrCanonInt
	}
	// Set the integer bytes.
	dst.SetBytes(buffer)
	return nil
}

// ReadUint256 decodes the next value as a uint256.
func (s *Stream) ReadUint256(dst *uint256.Int) error {
	var buffer []byte
	kind, size, err := s.Kind()
	switch {
	case err != nil:
		return err
	case kind == List:
		return ErrExpectedString
	case kind == Byte:
		buffer = s.uintbuf[:1]
		buffer[0] = s.byteval
		s.kind = -1 // re-arm Kind
	case size == 0:
		// Avoid zero-length read.
		s.kind = -1
	case size <= uint64(len(s.uintbuf)):
		// All possible uint256 values fit into s.uintbuf.
		buffer = s.uintbuf[:size]
		if err := s.readFull(buffer); err != nil {
			return err
		}
		// Reject inputs where single byte encoding should have been used.
		if size == 1 && buffer[0] < 128 {
			return ErrCanonSize
		}
	default:
		return errUint256Large
	}

	// Reject leading zero bytes.
	if len(buffer) > 0 && buffer[0] == 0 {
		return ErrCanonInt
	}
	// Set the integer bytes.
	dst.SetBytes(buffer)
	return nil
}

// Decode decodes a value and stores the result in the value pointed
// to by val. Please see the documentation for the Decode function
// to learn about the decoding rules.
func (s *Stream) Decode(val interface{}) error {
	if val == nil {
		return errDecodeIntoNil
	}
	rval := reflect.ValueOf(val)
	rtyp := rval.Type()
	if rtyp.Kind() != reflect.Ptr {
		return errNoPointer
	}
	if rval.IsNil() {
		return errDecodeIntoNil
	}
	decoder, err := cachedDecoder(rtyp.Elem())
	if err != nil {
		return err
	}

	err = decoder(s, rval.Elem())
	if decErr, ok := err.(*decodeError); ok && len(decErr.ctx) > 0 {
		// Add decode target type to error so context has more meaning.
		decErr.ctx = append(decErr.ctx, fmt.Sprint("(", rtyp.Elem(), ")"))
	}
	return err
}

// Reset discards any information about the current decoding context
// and starts reading from r. This method is meant to facilitate reuse
// of a preallocated Stream across many decoding operations.
//
// If r does not also implement ByteReader, Stream will do its own
// buffering.
func (s *Stream) Reset(r io.Reader, inputLimit uint64) {
	if inputLimit > 0 {
		s.remaining = inputLimit
		s.limited = true
	} else {
		// Attempt to automatically discover
		// the limit when reading from a byte slice.
		switch br := r.(type) {
		case *bytes.Reader:
			s.remaining = uint64(br.Len())
			s.limited = true
		case *bytes.Buffer:
			s.remaining = uint64(br.Len())
			s.limited = true
		case *strings.Reader:
			s.remaining = uint64(br.Len())
			s.limited = true
		default:
			s.limited = false
		}
	}
	// Wrap r with a buffer if it doesn't have one.
	bufr, ok := r.(ByteReader)
	if !ok {
		bufr = bufio.NewReader(r)
	}
	s.r = bufr
	// Reset the decoding context.
	s.stack = s.stack[:0]
	s.size = 0
	s.kind = -1
	s.kinderr = nil
	s.byteval = 0
	s.uintbuf = [32]byte{}
}

// Kind returns the kind and size of the next value in the
// input stream.
//
// The returned size is the number of bytes that make up the value.
// For kind == Byte, the size is zero because the value is
// contained in the type tag.
//
// The first call to Kind will read size information from the input
// reader and leave it positioned at the start of the actual bytes of
// the value. Subsequent calls to Kind (until the value is decoded)
// will not advance the input reader and return cached information.
func (s *Stream) Kind() (kind Kind, size uint64, err error) {
	if s.kind >= 0 {
		return s.kind, s.size, s.kinderr
	}

	// Check for end of list. This needs to be done here because readKind
	// checks against the list size, and would return the wrong error.
	inList, listLimit := s.listLimit()
	if inList && listLimit == 0 {
		return 0, 0, EOL
	}
	// Read the actual size tag.
	s.kind, s.size, s.kinderr = s.readKind()
	if s.kinderr == nil {
		// Check the data size of the value ahead against input limits. This
		// is done here because many decoders require allocating an input
		// buffer matching the value size. Checking it here protects those
		// decoders from inputs declaring very large value size.
		if inList && s.size > listLimit {
			s.kinderr = ErrElemTooLarge
		} else if s.limited && s.size > s.remaining {
			s.kinderr = ErrValueTooLarge
		}
	}
	return s.kind, s.size, s.kinderr
}

func (s *Stream) readKind() (kind Kind, size uint64, err error) {
	b, err := s.readByte()
	if err != nil {
		if len(s.stack) == 0 {
			// At toplevel, Adjust the error to actual EOF. io.EOF is
			// used by callers to determine when to stop decoding.
			switch err {
			case io.ErrUnexpectedEOF:
				err = io.EOF
			case ErrValueTooLarge:
				err = io.EOF
			}
		}
		return 0, 0, err
	}
	s.byteval = 0
	switch {
	case b < 0x80:
		// For a single byte whose value is in the [0x00, 0x7F] range, that byte
		// is its own RLP encoding.
		s.byteval = b
		return Byte, 0, nil
	case b < 0xB8:
		// Otherwise, if a string is 0-55 bytes long, the RLP encoding consists
		// of a single byte with value 0x80 plus the length of the string
		// followed by the string. The range of the first byte is thus [0x80, 0xB7].
		return String, uint64(b - 0x80), nil
	case b < 0xC0:
		// If a string is more than 55 bytes long, the RLP encoding consists of a
		// single byte with value 0xB7 plus the length of the length of the
		// string in binary form, followed by the length of the string, followed
		// by the string. For example, a length-1024 string would be encoded as
		// 0xB90400 followed by the string. The range of the first byte is thus
		// [0xB8, 0xBF].
		size, err = s.readUint(b - 0xB7)
		if err == nil && size < 56 {
			err = ErrCanonSize
		}
		return String, size, err
	case b < 0xF8:
		// If the total payload of a list (i.e. the combined length of all its
		// items) is 0-55 bytes long, the RLP encoding consists of a single byte
		// with value 0xC0 plus the length of the list followed by the
		// concatenation of the RLP encodings of the items. The range of the
		// first byte is thus [0xC0, 0xF7].
		return List, uint64(b - 0xC0), nil
	default:
		// If the total payload of a list is more than 55 bytes long, the RLP
		// encoding consists of a single byte with value 0xF7 plus the length of
		// the length of the payload in binary form, followed by the length of
		// the payload, followed by the concatenation of the RLP encodings of
		// the items. The range of the first byte is thus [0xF8, 0xFF].
		size, err = s.readUint(b - 0xF7)
		if err == nil && size < 56 {
			err = ErrCanonSize
		}
		return List, size, err
	}
}

func (s *Stream) readUint(size byte) (uint64, error) {
	switch size {
	case 0:
		s.kind = -1 // rearm Kind
		return 0, nil
	case 1:
		b, err := s.readByte()
		return uint64(b), err
	default:
		buffer := s.uintbuf[:8]
		for i := range buffer {
			buffer[i] = 0
		}
		start := int(8 - size)
		if err := s.readFull(buffer[start:]); err != nil {
			return 0, err
		}
		if buffer[start] == 0 {
			// Note: readUint is also used to decode integer values.
			// The error needs to be adjusted to become ErrCanonInt in this case.
			return 0, ErrCanonSize
		}
		return binary.BigEndian.Uint64(buffer[:]), nil
	}
}

// readFull reads into buf from the underlying stream.
func (s *Stream) readFull(buf []byte) (err error) {
	if err := s.willRead(uint64(len(buf))); err != nil {
		return err
	}
	var nn, n int
	for n < len(buf) && err == nil {
		nn, err = s.r.Read(buf[n:])
		n += nn
	}
	if err == io.EOF {
		if n < len(buf) {
			err = io.ErrUnexpectedEOF
		} else {
			// Readers are allowed to give EOF even though the read succeeded.
			// In such cases, we discard the EOF, like io.ReadFull() does.
			err = nil
		}
	}
	return err
}

// readByte reads a single byte from the underlying stream.
func (s *Stream) readByte() (byte, error) {
	if err := s.willRead(1); err != nil {
		return 0, err
	}
	b, err := s.r.ReadByte()
	if err == io.EOF {
		err = io.ErrUnexpectedEOF
	}
	return b, err
}

// willRead is called before any read from the underlying stream. It checks
// n against size limits, and updates the limits if n doesn't overflow them.
func (s *Stream) willRead(n uint64) error {
	s.kind = -1 // rearm Kind

	if inList, limit := s.listLimit(); inList {
		if n > limit {
			return ErrElemTooLarge
		}
		s.stack[len(s.stack)-1] = limit - n
	}
	if s.limited {
		if n > s.remaining {
			return ErrValueTooLarge
		}
		s.remaining -= n
	}
	return nil
}

// listLimit returns the amount of data remaining in the innermost list.
func (s *Stream) listLimit() (inList bool, limit uint64) {
	if len(s.stack) == 0 {
		return false, 0
	}
	return true, s.stack[len(s.stack)-1]
}

type sliceReader []byte

func (sr *sliceReader) Read(b []byte) (int, error) {
	if len(*sr) == 0 {
		return 0, io.EOF
	}
	n := copy(b, *sr)
	*sr = (*sr)[n:]
	return n, nil
}

func (sr *sliceReader) ReadByte() (byte, error) {
	if len(*sr) == 0 {
		return 0, io.EOF
	}
	b := (*sr)[0]
	*sr = (*sr)[1:]
	return b, nil
}
