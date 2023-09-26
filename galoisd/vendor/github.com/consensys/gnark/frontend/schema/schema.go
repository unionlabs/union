/*
Copyright © 2022 ConsenSys Software Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package schema

import (
	"fmt"
	"io"
	"reflect"
	"strconv"
	"strings"
)

// Schema represents the structure of a gnark circuit (/ witness)
type Schema struct {
	Fields   []Field
	NbPublic int
	NbSecret int
}

// New builds a schema.Schema walking through the provided interface (a circuit structure).
//
// schema.Walk performs better and should be used when possible.
func New(circuit interface{}, tLeaf reflect.Type) (*Schema, error) {
	// note circuit is of type interface{} instead of frontend.Circuit to avoid import cycle
	// same for tLeaf it is in practice always frontend.Variable

	var nbPublic, nbSecret int
	fields, err := parse(nil, circuit, tLeaf, "", "", "", Unset, &nbPublic, &nbSecret)
	if err != nil {
		return nil, err
	}

	return &Schema{Fields: fields, NbPublic: nbPublic, NbSecret: nbSecret}, nil
}

// Instantiate builds a concrete type using reflect matching the provided schema
//
// It replaces leafs by provided type, such that one can do:
//
//	struct { A []frontend.Variable} -> Schema -> struct {A [12]fr.Element}
//
// Default behavior is to add "json:,omitempty" to the generated struct
func (s Schema) Instantiate(leafType reflect.Type, omitEmptyTag ...bool) interface{} {
	omitEmpty := true
	if len(omitEmptyTag) == 1 {
		omitEmpty = omitEmptyTag[0]
	}

	// first, let's replace the Field by reflect.StructField
	is := toStructField(s.Fields, leafType, omitEmpty)

	// now create the corresponding type
	typ := reflect.StructOf(is)

	// instantiate the type
	v := reflect.New(typ).Elem()

	// return interface
	return v.Addr().Interface()
}

// WriteSequence writes the expected sequence order of the witness on provided writer
// witness elements are identified by their tag name, or if unset, struct & field name
//
// The expected sequence matches the binary encoding protocol [public | secret]
func (s Schema) WriteSequence(w io.Writer) error {
	var public, secret []string

	var a int
	instance := s.Instantiate(reflect.TypeOf(a), false)

	collectHandler := func(f LeafInfo, _ reflect.Value) error {
		if f.Visibility == Public {
			public = append(public, f.FullName())
		} else if f.Visibility == Secret {
			secret = append(secret, f.FullName())
		}
		return nil
	}
	if _, err := Walk(instance, reflect.TypeOf(a), collectHandler); err != nil {
		return err
	}

	if _, err := io.WriteString(w, "public:\n"); err != nil {
		return err
	}
	for _, p := range public {
		if _, err := io.WriteString(w, p); err != nil {
			return err
		}
		if _, err := w.Write([]byte{'\n'}); err != nil {
			return err
		}
	}

	if _, err := io.WriteString(w, "secret:\n"); err != nil {
		return err
	}
	for _, s := range secret {
		if _, err := io.WriteString(w, s); err != nil {
			return err
		}
		if _, err := w.Write([]byte{'\n'}); err != nil {
			return err
		}
	}

	return nil
}

// toStructField recurse through Field and builds corresponding reflect.StructField
func toStructField(fields []Field, leafType reflect.Type, omitEmpty bool) []reflect.StructField {
	r := make([]reflect.StructField, len(fields))

	for i, f := range fields {
		r[i] = reflect.StructField{
			Name: f.Name,
			Tag:  structTag(f.NameTag, f.Visibility, omitEmpty),
		}
		switch f.Type {
		case Leaf:
			r[i].Type = leafType
		case Array:
			r[i].Type = arrayElementType(f.ArraySize, f.SubFields, leafType, omitEmpty)
		case Struct:
			r[i].Type = reflect.StructOf(toStructField(f.SubFields, leafType, omitEmpty))
		}
	}

	return r
}

func arrayElementType(n int, fields []Field, leafType reflect.Type, omitEmpty bool) reflect.Type {
	// we know parent is an array.
	// we check first element of fields
	// if it's a struct or a leaf, we're done.
	// if it's another array, we recurse

	if len(fields) == 0 {
		// no subfields, we reached an array of leaves
		return reflect.ArrayOf(n, leafType)
	}

	switch fields[0].Type {
	case Struct:
		return reflect.ArrayOf(n, reflect.StructOf(toStructField(fields[0].SubFields, leafType, omitEmpty)))
	case Array:
		return reflect.ArrayOf(n, arrayElementType(fields[0].ArraySize, fields[0].SubFields, leafType, omitEmpty))
	}
	panic("invalid array type")
}

func structTag(baseNameTag string, visibility Visibility, omitEmpty bool) reflect.StructTag {
	sOmitEmpty := ""
	if omitEmpty {
		sOmitEmpty = ",omitempty"
	}
	if visibility == Unset {
		if baseNameTag != "" {
			return reflect.StructTag(fmt.Sprintf("gnark:\"%s\" json:\"%s%s\"", baseNameTag, baseNameTag, sOmitEmpty))
		}
		return ""
	}
	if baseNameTag == "" {
		if !omitEmpty {
			return reflect.StructTag(fmt.Sprintf("gnark:\",%s\"", visibility.String()))
		}
		return reflect.StructTag(fmt.Sprintf("gnark:\",%s\" json:\",omitempty\"", visibility.String()))
	}
	return reflect.StructTag(fmt.Sprintf("gnark:\"%s,%s\" json:\"%s%s\"", baseNameTag, visibility.String(), baseNameTag, sOmitEmpty))
}

// parentFullName: the name of parent with its ancestors separated by "_"
// parentGoName: the name of parent (Go struct definition)
// parentTagName: may be empty, set if a struct tag with name is set
func parse(r []Field, input interface{}, target reflect.Type, parentFullName, parentGoName, parentTagName string, parentVisibility Visibility, nbPublic, nbSecret *int) ([]Field, error) {
	tValue := reflect.ValueOf(input)

	// get pointed value if needed
	if tValue.Kind() == reflect.Ptr {
		tValue = tValue.Elem()
	}

	// stop condition
	if tValue.Type() == target {
		f := Field{
			Name:       parentGoName,
			NameTag:    parentTagName,
			FullName:   parentFullName,
			Visibility: parentVisibility,
			Type:       Leaf,
			SubFields:  nil,
			ArraySize:  1,
		}
		if f.Visibility == Unset {
			f.Visibility = Secret
		}
		if f.Visibility == Secret {
			(*nbSecret) += f.ArraySize
		} else if f.Visibility == Public {
			(*nbPublic) += f.ArraySize
		}
		return append(r, f), nil
	}

	// struct
	if tValue.Kind() == reflect.Struct {
		var subFields []Field

		// get visible fields
		fields := reflect.VisibleFields(tValue.Type())

		for _, f := range fields {
			// check if the gnark tag is set
			tag, ok := f.Tag.Lookup(string(tagKey))
			if ok && tag == string(TagOptOmit) {
				continue // skipping "-"
			}

			// default visibility is Unset
			visibility := Unset

			// variable name is field name, unless overridden by gnark tag value
			name := f.Name
			var nameTag string

			if ok && tag != "" {
				// gnark tag is set
				var opts tagOptions
				nameTag, opts = parseTag(tag)
				if !isValidTag(nameTag) {
					nameTag = ""
				}
				opts = tagOptions(strings.TrimSpace(string(opts)))
				switch {
				case opts.contains(TagOptSecret):
					visibility = Secret
				case opts.contains(TagOptPublic):
					visibility = Public
				case opts == "" && parentFullName == "":
					// our promise is to set visibility to secret for empty-tagged elements.
					visibility = Secret
				case opts == "":
					// even though we have the promise, then in tests we have
					// assumed that sub-elements without any tags assume parents
					// visibility (see below). For compatibility, make the same
					// assumption.
					visibility = parentVisibility
				case opts.contains(TagOptInherit) && parentFullName != "":
					// we have been asked explicitly to inherit the visibility
					visibility = parentVisibility
				case opts.contains(TagOptInherit):
					// but we can not inherit the visibility for top-level
					// elements. Return an error.
					return r, fmt.Errorf("can not inherit visibility for top-level element %s", getFullName(parentGoName, name, nameTag))
				default:
					return r, fmt.Errorf("invalid gnark struct tag option on %s. must be \"public\", \"secret\" or \"-\"", getFullName(parentGoName, name, nameTag))
				}
			}

			if ((parentVisibility == Public) && (visibility == Secret)) ||
				((parentVisibility == Secret) && (visibility == Public)) {
				return r, fmt.Errorf("conflicting visibility. %s (%s) has a parent with different visibility attribute", getFullName(parentGoName, name, nameTag), visibility.String())
			}

			// inherit parent visibility
			if visibility == Unset {
				visibility = parentVisibility
			}

			fValue := tValue.FieldByIndex(f.Index)

			if fValue.CanAddr() && fValue.Addr().CanInterface() {
				value := fValue.Addr().Interface()
				if ih, hasInitHook := value.(InitHook); hasInitHook {
					ih.GnarkInitHook()
				}
				var err error
				subFields, err = parse(subFields, value, target, getFullName(parentFullName, name, nameTag), name, nameTag, visibility, nbPublic, nbSecret)
				if err != nil {
					return r, err
				}
			}
		}

		if parentGoName == "" {
			// root
			return subFields, nil
		}
		// we just add it to our current fields
		// if parentVisibility == Unset {
		// 	parentVisibility = Secret // default visibility to Secret
		// }
		if len(subFields) == 0 {
			// nothing to add in the schema
			return r, nil
		}
		return append(r, Field{
			Name:       parentGoName,
			NameTag:    parentTagName,
			Type:       Struct,
			SubFields:  subFields,
			Visibility: parentVisibility, // == Secret,
		}), nil

	}

	if tValue.Kind() == reflect.Slice || tValue.Kind() == reflect.Array {
		if tValue.Len() == 0 {
			if reflect.SliceOf(target) == tValue.Type() {
				fmt.Printf("ignoring uninitialized slice: %s %s\n", parentGoName, reflect.SliceOf(target).String())
			}
			return r, nil
		}

		// []frontend.Variable
		// [n]frontend.Variable
		// [] / [n] of something else.
		if reflect.SliceOf(target) == tValue.Type() || reflect.ArrayOf(tValue.Len(), target) == tValue.Type() {
			// if parentVisibility == Unset {
			// 	parentVisibility = Secret // default visibility to Secret
			// }

			for j := 0; j < tValue.Len(); j++ {
				val := tValue.Index(j)
				if val.CanAddr() && val.Addr().CanInterface() {
					fqn := getFullName(parentFullName, strconv.Itoa(j), "")
					if _, err := parse(nil, val.Addr().Interface(), target, fqn, fqn, parentTagName, parentVisibility, nbPublic, nbSecret); err != nil {
						return nil, err
					}
				}
			}

			return append(r, Field{
				Name:       parentGoName,
				NameTag:    parentTagName,
				Type:       Array,
				Visibility: parentVisibility,
				ArraySize:  tValue.Len(),
			}), nil
		}

		// we have a slice / array of things that may contain variables
		var subFields []Field
		var err error
		for j := 0; j < tValue.Len(); j++ {
			val := tValue.Index(j)
			if val.CanAddr() && val.Addr().CanInterface() {
				fqn := getFullName(parentFullName, strconv.Itoa(j), "")
				ival := val.Addr().Interface()
				if ih, hasInitHook := ival.(InitHook); hasInitHook {
					ih.GnarkInitHook()
				}
				subFields, err = parse(subFields, ival, target, fqn, fqn, parentTagName, parentVisibility, nbPublic, nbSecret)
				if err != nil {
					return nil, err
				}
			}
		}
		if len(subFields) == 0 {
			// nothing to add
			return r, nil
		}
		return append(r, Field{
			Name:       parentGoName,
			NameTag:    parentTagName,
			Type:       Array,
			SubFields:  subFields[:1], // TODO @gbotrel we should ensure that elements are not heterogeneous?
			Visibility: parentVisibility,
			ArraySize:  tValue.Len(),
		}), nil

	}

	return r, nil
}

// specify parentName, name and tag
// returns fully qualified name
func getFullName(parentFullName, name, tagName string) string {
	n := name
	if tagName != "" {
		n = tagName
	}
	if parentFullName == "" {
		return n
	}
	return parentFullName + "_" + n
}
