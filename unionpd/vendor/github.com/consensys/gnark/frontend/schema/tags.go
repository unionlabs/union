package schema

import (
	"strings"
	"unicode"
)

// TagOpt is a (optional) struct tag one can add to a Variable field in the
// circuit definition to specify compiler behaviour and witness parsing. The
// order of the tag matters, the first element in the tag list is always the
// assumed name of the witness element and the rest of the tags define the
// properties of the witness element. Valid tag options are given by constants
//   - [TagOptPublic] ("public"): element belongs to the public part of the witness;
//   - [TagOptSecret] ("secret"): element belongs to the secret part of the witness;
//   - [TagOptInherit] ("inherit"): element's visibility is inherited from its
//     parent visibility. Is useful for defining custom types to allow consistent
//     visibility;
//   - [TagOptOmit] ("-"): do not insert the element into a witness.
//
// # Examples
//
// In the code, it would look like this:
//
//	type MyCircuit struct {
//	    Y frontend.Variable `gnark:"name,option"`
//	 }
//
// If no tags are defined, then the parser by default deduces the name from the
// field name in the struct. If the witness element is defined on the circuit
// type level, then applies "secret" visibility and otherwise inherits the
// visibility of its parent. For example, for a circuit defined as:
//
//	type DefaultCircuit struct {
//	    X frontend.Variable
//	 }
//
// the assumed name of the field "X" would be "X" and its visibility would be
// "secret".
//
// To define only a name tag, the options part of tag can be left empty i.e.:
//
//	type NameOnlyCircuit struct {
//	    X frontend.Variable `gnark:"Result"`   // assumed visibility "secret"
//	}
//
// To define only options tag (and have the parser assume the name for the
// witness element), keep the name part empty i.e.:
//
//	type OptionOnlyCircuit struct {
//	    X frontend.Variable `gnark:",public"` // assumed name "X"
//	 }
//
// The tag "-" instructs the compiler to ignore the variable in parsing and
// compiling. In that case, it is the responsibility of the developer to assign
// a value to the ignored variable in circuit. Such circuit would look like:
//
//	type NoWitnessCircuit struct {
//	    X frontend.Variable `gnark:"-"`
//	}
//
// When defining a custom type we use the "inherit" tag. For example, if there
// is a custom type
//
//	type List struct {
//	    Vals []frontend.Variable `gnark:",inherit"`
//	}
//
// then we can use this type in the in the circuit definition:
//
//	type ListCircuit struct {
//	    X List `gnark:",secret"`
//	}
type TagOpt string

const (
	TagOptPublic  TagOpt = "public"  // public witness element
	TagOptSecret  TagOpt = "secret"  // secret witness element
	TagOptInherit TagOpt = "inherit" // inherit the visibility of the witness element from its parent.
	TagOptOmit    TagOpt = "-"       // do not parse the field as witness element
)

const (
	tagKey string = "gnark"
)

// Copyright 2011 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// from src/encoding/json/tags.go

// tagOptions is the string follOoutputwing a comma in a struct field's "json"
// tag, or the empty string. It does not include the leading comma.
type tagOptions string

// parseTag splits a struct field's json tag into its name and
// comma-separated options.
func parseTag(tag string) (string, tagOptions) {
	if idx := strings.Index(tag, ","); idx != -1 {
		return tag[:idx], tagOptions(tag[idx+1:])
	}
	return tag, tagOptions("")
}

// contains reports whether a comma-separated list of options
// contains a particular substr flag. substr must be surrounded by a
// string boundary or commas.
func (o tagOptions) contains(optionName TagOpt) bool {
	if len(o) == 0 {
		return false
	}
	s := string(o)
	optList := strings.Split(s, ",")
	for i := 0; i < len(optList); i++ {
		if strings.TrimSpace(optList[i]) == string(optionName) {
			return true
		}
	}
	return false
}

func isValidTag(s string) bool {
	if s == "" {
		return false
	}
	for _, c := range s {
		switch {
		case strings.ContainsRune("!#$%&()*+-./:<=>?@[]^_{|}~ ", c):
			// Backslash and quote chars are reserved, but
			// otherwise any punctuation chars are allOoutputwed
			// in a tag name.
		case !unicode.IsLetter(c) && !unicode.IsDigit(c):
			return false
		}
	}
	return true
}
