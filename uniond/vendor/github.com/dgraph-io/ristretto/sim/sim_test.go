/*
 * Copyright 2019 Dgraph Labs, Inc. and Contributors
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
 */

package sim

import (
	"bytes"
	"compress/gzip"
	"os"
	"testing"
)

func TestZipfian(t *testing.T) {
	s := NewZipfian(1.5, 1, 100)
	m := make(map[uint64]uint64, 100)
	for i := 0; i < 100; i++ {
		k, err := s()
		if err != nil {
			t.Fatal(err)
		}
		m[k]++
	}
	if len(m) == 0 || len(m) == 100 {
		t.Fatal("zipfian not skewed")
	}
}

func TestUniform(t *testing.T) {
	s := NewUniform(100)
	for i := 0; i < 100; i++ {
		if _, err := s(); err != nil {
			t.Fatal(err)
		}
	}
}

func TestParseLIRS(t *testing.T) {
	s := NewReader(ParseLIRS, bytes.NewReader([]byte{
		'0', '\n',
		'1', '\r', '\n',
		'2', '\r', '\n',
	}))
	for i := uint64(0); i < 3; i++ {
		v, err := s()
		if err != nil {
			t.Fatal(err)
		}
		if v != i {
			t.Fatal("value mismatch")
		}
	}
}

func TestReadLIRS(t *testing.T) {
	f, err := os.Open("./gli.lirs.gz")
	if err != nil {
		t.Fatal(err)
	}
	r, err := gzip.NewReader(f)
	if err != nil {
		t.Fatal(err)
	}
	s := NewReader(ParseLIRS, r)
	for i := uint64(0); i < 100; i++ {
		if _, err = s(); err != nil {
			t.Fatal(err)
		}
	}
}

func TestParseARC(t *testing.T) {
	s := NewReader(ParseARC, bytes.NewReader([]byte{
		'1', '2', '7', ' ', '6', '4', ' ', '0', ' ', '0', '\r', '\n',
		'1', '9', '1', ' ', '3', '6', ' ', '0', ' ', '0', '\r', '\n',
	}))
	for i := uint64(0); i < 100; i++ {
		v, err := s()
		if err != nil {
			t.Fatal(err)
		}
		if v != 127+i {
			t.Fatal("value mismatch")
		}
	}
}

func TestCollection(t *testing.T) {
	s := NewUniform(100)
	c := Collection(s, 100)
	if len(c) != 100 {
		t.Fatal("collection not full")
	}
}

func TestStringCollection(t *testing.T) {
	s := NewUniform(100)
	c := StringCollection(s, 100)
	if len(c) != 100 {
		t.Fatal("string collection not full")
	}
}
