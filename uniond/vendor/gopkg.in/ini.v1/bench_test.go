// Copyright 2017 Unknwon
//
// Licensed under the Apache License, Version 2.0 (the "License"): you may
// not use this file except in compliance with the License. You may obtain
// a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

package ini

import (
	"testing"
)

func newTestFile(block bool) *File {
	c, _ := Load([]byte(confData))
	c.BlockMode = block
	return c
}

func Benchmark_Key_Value(b *testing.B) {
	c := newTestFile(true)
	for i := 0; i < b.N; i++ {
		c.Section("").Key("NAME").Value()
	}
}

func Benchmark_Key_Value_NonBlock(b *testing.B) {
	c := newTestFile(false)
	for i := 0; i < b.N; i++ {
		c.Section("").Key("NAME").Value()
	}
}

func Benchmark_Key_Value_ViaSection(b *testing.B) {
	c := newTestFile(true)
	sec := c.Section("")
	for i := 0; i < b.N; i++ {
		sec.Key("NAME").Value()
	}
}

func Benchmark_Key_Value_ViaSection_NonBlock(b *testing.B) {
	c := newTestFile(false)
	sec := c.Section("")
	for i := 0; i < b.N; i++ {
		sec.Key("NAME").Value()
	}
}

func Benchmark_Key_Value_Direct(b *testing.B) {
	c := newTestFile(true)
	key := c.Section("").Key("NAME")
	for i := 0; i < b.N; i++ {
		key.Value()
	}
}

func Benchmark_Key_Value_Direct_NonBlock(b *testing.B) {
	c := newTestFile(false)
	key := c.Section("").Key("NAME")
	for i := 0; i < b.N; i++ {
		key.Value()
	}
}

func Benchmark_Key_String(b *testing.B) {
	c := newTestFile(true)
	for i := 0; i < b.N; i++ {
		_ = c.Section("").Key("NAME").String()
	}
}

func Benchmark_Key_String_NonBlock(b *testing.B) {
	c := newTestFile(false)
	for i := 0; i < b.N; i++ {
		_ = c.Section("").Key("NAME").String()
	}
}

func Benchmark_Key_String_ViaSection(b *testing.B) {
	c := newTestFile(true)
	sec := c.Section("")
	for i := 0; i < b.N; i++ {
		_ = sec.Key("NAME").String()
	}
}

func Benchmark_Key_String_ViaSection_NonBlock(b *testing.B) {
	c := newTestFile(false)
	sec := c.Section("")
	for i := 0; i < b.N; i++ {
		_ = sec.Key("NAME").String()
	}
}

func Benchmark_Key_SetValue(b *testing.B) {
	c := newTestFile(true)
	for i := 0; i < b.N; i++ {
		c.Section("").Key("NAME").SetValue("10")
	}
}

func Benchmark_Key_SetValue_VisSection(b *testing.B) {
	c := newTestFile(true)
	sec := c.Section("")
	for i := 0; i < b.N; i++ {
		sec.Key("NAME").SetValue("10")
	}
}
