// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"encoding/json"
	"fmt"

	"github.com/tunabay/go-bitarray"
	"gopkg.in/yaml.v3"
)

func ExampleBitArray_MarshalBinary() {
	ba := bitarray.MustParse("1111-0101 11")

	bin, _ := ba.MarshalBinary()
	fmt.Printf("%x\n", bin)

	// Output:
	// f5c3
}

func ExampleBitArray_MarshalText() {
	ba := bitarray.MustParse("1111-0101 11")

	txt, _ := ba.MarshalText()
	fmt.Printf("%s\n", txt)

	// Output:
	// 1111010111
}

func ExampleBitArray_MarshalJSON() {
	data := &struct {
		Foo  *bitarray.BitArray `json:"foox"`
		Bar  *bitarray.BitArray
		Baz  *bitarray.BitArray
		Qux  *bitarray.BitArray
		List []*bitarray.BitArray
	}{
		Foo: bitarray.MustParse("1111-0000 111"),
		Bar: bitarray.MustParse("0000-0000"),
		Baz: bitarray.MustParse(""),
		List: []*bitarray.BitArray{
			bitarray.MustParse("0001"),
			bitarray.MustParse("1000"),
		},
	}

	out, err := json.MarshalIndent(data, "", "  ")
	if err != nil {
		panic(err)
	}

	fmt.Println(string(out))

	// Output:
	// {
	//   "foox": "11110000111",
	//   "Bar": "00000000",
	//   "Baz": "",
	//   "Qux": null,
	//   "List": [
	//     "0001",
	//     "1000"
	//   ]
	// }
}

func ExampleBitArray_MarshalYAML() {
	data := &struct {
		Foo  *bitarray.BitArray `yaml:"foox"`
		Bar  *bitarray.BitArray
		Baz  *bitarray.BitArray
		Qux  *bitarray.BitArray
		List []*bitarray.BitArray
	}{
		Foo: bitarray.MustParse("1111-0000 111"),
		Bar: bitarray.MustParse("0000-0000"),
		Baz: bitarray.MustParse(""),
		List: []*bitarray.BitArray{
			bitarray.MustParse("0001"),
			bitarray.MustParse("1000"),
		},
	}

	out, err := yaml.Marshal(data)
	if err != nil {
		panic(err)
	}

	fmt.Println(string(out))

	// Output:
	// foox: "11110000111"
	// bar: "00000000"
	// baz: ""
	// qux: null
	// list:
	//     - "0001"
	//     - "1000"
}
