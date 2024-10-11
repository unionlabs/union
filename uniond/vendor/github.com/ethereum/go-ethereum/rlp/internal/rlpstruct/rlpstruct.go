// Copyright 2022 The go-ethereum Authors
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

// Package rlpstruct implements struct processing for RLP encoding/decoding.
//
// In particular, this package handles all rules around field filtering,
// struct tags and nil value determination.
package rlpstruct

import (
	"fmt"
	"reflect"
	"strings"
)

// Field represents a struct field.
type Field struct {
	Name     string
	Index    int
	Exported bool
	Type     Type
	Tag      string
}

// Type represents the attributes of a Go type.
type Type struct {
	Name      string
	Kind      reflect.Kind
	IsEncoder bool  // whether type implements rlp.Encoder
	IsDecoder bool  // whether type implements rlp.Decoder
	Elem      *Type // non-nil for Kind values of Ptr, Slice, Array
}

// DefaultNilValue determines whether a nil pointer to t encodes/decodes
// as an empty string or empty list.
func (t Type) DefaultNilValue() NilKind {
	k := t.Kind
	if isUint(k) || k == reflect.String || k == reflect.Bool || isByteArray(t) {
		return NilKindString
	}
	return NilKindList
}

// NilKind is the RLP value encoded in place of nil pointers.
type NilKind uint8

const (
	NilKindString NilKind = 0x80
	NilKindList   NilKind = 0xC0
)

// Tags represents struct tags.
type Tags struct {
	// rlp:"nil" controls whether empty input results in a nil pointer.
	// nilKind is the kind of empty value allowed for the field.
	NilKind NilKind
	NilOK   bool

	// rlp:"optional" allows for a field to be missing in the input list.
	// If this is set, all subsequent fields must also be optional.
	Optional bool

	// rlp:"tail" controls whether this field swallows additional list elements. It can
	// only be set for the last field, which must be of slice type.
	Tail bool

	// rlp:"-" ignores fields.
	Ignored bool
}

// TagError is raised for invalid struct tags.
type TagError struct {
	StructType string

	// These are set by this package.
	Field string
	Tag   string
	Err   string
}

func (e TagError) Error() string {
	field := "field " + e.Field
	if e.StructType != "" {
		field = e.StructType + "." + e.Field
	}
	return fmt.Sprintf("rlp: invalid struct tag %q for %s (%s)", e.Tag, field, e.Err)
}

// ProcessFields filters the given struct fields, returning only fields
// that should be considered for encoding/decoding.
func ProcessFields(allFields []Field) ([]Field, []Tags, error) {
	lastPublic := lastPublicField(allFields)

	// Gather all exported fields and their tags.
	var fields []Field
	var tags []Tags
	for _, field := range allFields {
		if !field.Exported {
			continue
		}
		ts, err := parseTag(field, lastPublic)
		if err != nil {
			return nil, nil, err
		}
		if ts.Ignored {
			continue
		}
		fields = append(fields, field)
		tags = append(tags, ts)
	}

	// Verify optional field consistency. If any optional field exists,
	// all fields after it must also be optional. Note: optional + tail
	// is supported.
	var anyOptional bool
	var firstOptionalName string
	for i, ts := range tags {
		name := fields[i].Name
		if ts.Optional || ts.Tail {
			if !anyOptional {
				firstOptionalName = name
			}
			anyOptional = true
		} else {
			if anyOptional {
				msg := fmt.Sprintf("must be optional because preceding field %q is optional", firstOptionalName)
				return nil, nil, TagError{Field: name, Err: msg}
			}
		}
	}
	return fields, tags, nil
}

func parseTag(field Field, lastPublic int) (Tags, error) {
	name := field.Name
	tag := reflect.StructTag(field.Tag)
	var ts Tags
	for _, t := range strings.Split(tag.Get("rlp"), ",") {
		switch t = strings.TrimSpace(t); t {
		case "":
			// empty tag is allowed for some reason
		case "-":
			ts.Ignored = true
		case "nil", "nilString", "nilList":
			ts.NilOK = true
			if field.Type.Kind != reflect.Ptr {
				return ts, TagError{Field: name, Tag: t, Err: "field is not a pointer"}
			}
			switch t {
			case "nil":
				ts.NilKind = field.Type.Elem.DefaultNilValue()
			case "nilString":
				ts.NilKind = NilKindString
			case "nilList":
				ts.NilKind = NilKindList
			}
		case "optional":
			ts.Optional = true
			if ts.Tail {
				return ts, TagError{Field: name, Tag: t, Err: `also has "tail" tag`}
			}
		case "tail":
			ts.Tail = true
			if field.Index != lastPublic {
				return ts, TagError{Field: name, Tag: t, Err: "must be on last field"}
			}
			if ts.Optional {
				return ts, TagError{Field: name, Tag: t, Err: `also has "optional" tag`}
			}
			if field.Type.Kind != reflect.Slice {
				return ts, TagError{Field: name, Tag: t, Err: "field type is not slice"}
			}
		default:
			return ts, TagError{Field: name, Tag: t, Err: "unknown tag"}
		}
	}
	return ts, nil
}

func lastPublicField(fields []Field) int {
	last := 0
	for _, f := range fields {
		if f.Exported {
			last = f.Index
		}
	}
	return last
}

func isUint(k reflect.Kind) bool {
	return k >= reflect.Uint && k <= reflect.Uintptr
}

func isByte(typ Type) bool {
	return typ.Kind == reflect.Uint8 && !typ.IsEncoder
}

func isByteArray(typ Type) bool {
	return (typ.Kind == reflect.Slice || typ.Kind == reflect.Array) && isByte(*typ.Elem)
}
