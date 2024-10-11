/*
 *
 * Copyright 2019 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package attributes_test

import (
	"fmt"
	"testing"

	"google.golang.org/grpc/attributes"
)

type stringVal struct {
	s string
}

func (s stringVal) Equal(o any) bool {
	os, ok := o.(stringVal)
	return ok && s.s == os.s
}

type stringerVal struct {
	s string
}

func (s stringerVal) String() string {
	return s.s
}

func ExampleAttributes() {
	type keyOne struct{}
	type keyTwo struct{}
	a := attributes.New(keyOne{}, 1).WithValue(keyTwo{}, stringVal{s: "two"})
	fmt.Println("Key one:", a.Value(keyOne{}))
	fmt.Println("Key two:", a.Value(keyTwo{}))
	// Output:
	// Key one: 1
	// Key two: {two}
}

func ExampleAttributes_WithValue() {
	type keyOne struct{}
	type keyTwo struct{}
	a := attributes.New(keyOne{}, 1)
	a = a.WithValue(keyTwo{}, stringVal{s: "two"})
	fmt.Println("Key one:", a.Value(keyOne{}))
	fmt.Println("Key two:", a.Value(keyTwo{}))
	// Output:
	// Key one: 1
	// Key two: {two}
}

func ExampleAttributes_String() {
	type key struct{}
	var typedNil *stringerVal
	a1 := attributes.New(key{}, typedNil)            // typed nil implements [fmt.Stringer]
	a2 := attributes.New(key{}, (*stringerVal)(nil)) // typed nil implements [fmt.Stringer]
	a3 := attributes.New(key{}, (*stringVal)(nil))   // typed nil not implements [fmt.Stringer]
	a4 := attributes.New(key{}, nil)                 // untyped nil
	a5 := attributes.New(key{}, 1)
	a6 := attributes.New(key{}, stringerVal{s: "two"})
	a7 := attributes.New(key{}, stringVal{s: "two"})
	a8 := attributes.New(1, true)
	fmt.Println("a1:", a1.String())
	fmt.Println("a2:", a2.String())
	fmt.Println("a3:", a3.String())
	fmt.Println("a4:", a4.String())
	fmt.Println("a5:", a5.String())
	fmt.Println("a6:", a6.String())
	fmt.Println("a7:", a7.String())
	fmt.Println("a8:", a8.String())
	// Output:
	// a1: {"<%!p(attributes_test.key={})>": "<nil>" }
	// a2: {"<%!p(attributes_test.key={})>": "<nil>" }
	// a3: {"<%!p(attributes_test.key={})>": "<0x0>" }
	// a4: {"<%!p(attributes_test.key={})>": "<%!p(<nil>)>" }
	// a5: {"<%!p(attributes_test.key={})>": "<%!p(int=1)>" }
	// a6: {"<%!p(attributes_test.key={})>": "two" }
	// a7: {"<%!p(attributes_test.key={})>": "<%!p(attributes_test.stringVal={two})>" }
	// a8: {"<%!p(int=1)>": "<%!p(bool=true)>" }
}

// Test that two attributes with the same content are Equal.
func TestEqual(t *testing.T) {
	type keyOne struct{}
	type keyTwo struct{}
	a1 := attributes.New(keyOne{}, 1).WithValue(keyTwo{}, stringVal{s: "two"})
	a2 := attributes.New(keyOne{}, 1).WithValue(keyTwo{}, stringVal{s: "two"})
	if !a1.Equal(a2) {
		t.Fatalf("%+v.Equals(%+v) = false; want true", a1, a2)
	}
	if !a2.Equal(a1) {
		t.Fatalf("%+v.Equals(%+v) = false; want true", a2, a1)
	}
}

// Test that two attributes with different content are not Equal.
func TestNotEqual(t *testing.T) {
	type keyOne struct{}
	type keyTwo struct{}
	a1 := attributes.New(keyOne{}, 1).WithValue(keyTwo{}, stringVal{s: "two"})
	a2 := attributes.New(keyOne{}, 2).WithValue(keyTwo{}, stringVal{s: "two"})
	a3 := attributes.New(keyOne{}, 1).WithValue(keyTwo{}, stringVal{s: "one"})
	if a1.Equal(a2) {
		t.Fatalf("%+v.Equals(%+v) = true; want false", a1, a2)
	}
	if a2.Equal(a1) {
		t.Fatalf("%+v.Equals(%+v) = true; want false", a2, a1)
	}
	if a3.Equal(a1) {
		t.Fatalf("%+v.Equals(%+v) = true; want false", a3, a1)
	}
}
