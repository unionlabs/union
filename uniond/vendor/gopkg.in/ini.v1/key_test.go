// Copyright 2014 Unknwon
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
	"bytes"
	"fmt"
	"runtime"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestKey_AddShadow(t *testing.T) {
	t.Run("add shadow to a key", func(t *testing.T) {
		f, err := ShadowLoad([]byte(`
[notes]
-: note1`))
		require.NoError(t, err)
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)

		assert.NoError(t, k.AddShadow("ini.v1"))
		assert.Equal(t, []string{"ini", "ini.v1"}, k.ValueWithShadows())

		t.Run("add shadow to boolean key", func(t *testing.T) {
			k, err := f.Section("").NewBooleanKey("published")
			require.NoError(t, err)
			require.NotNil(t, k)
			assert.Error(t, k.AddShadow("beta"))
		})

		t.Run("add shadow to auto-increment key", func(t *testing.T) {
			assert.Error(t, f.Section("notes").Key("#1").AddShadow("beta"))
		})

		t.Run("deduplicate an existing value", func(t *testing.T) {
			k := f.Section("").Key("NAME")
			assert.NoError(t, k.AddShadow("ini"))
			assert.Equal(t, []string{"ini", "ini.v1"}, k.ValueWithShadows())
		})

		t.Run("ignore empty shadow values", func(t *testing.T) {
			k := f.Section("").Key("empty")
			assert.NoError(t, k.AddShadow(""))
			assert.NoError(t, k.AddShadow("ini"))
			assert.Equal(t, []string{"ini"}, k.ValueWithShadows())
		})
	})

	t.Run("allow duplicate shadowed values", func(t *testing.T) {
		f := Empty(LoadOptions{
			AllowShadows:               true,
			AllowDuplicateShadowValues: true,
		})
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)

		assert.NoError(t, k.AddShadow("ini.v1"))
		assert.NoError(t, k.AddShadow("ini"))
		assert.NoError(t, k.AddShadow("ini"))
		assert.Equal(t, []string{"ini", "ini.v1", "ini", "ini"}, k.ValueWithShadows())
	})

	t.Run("shadow is not allowed", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)

		assert.Error(t, k.AddShadow("ini.v1"))
	})
}

// Helpers for slice tests.
func float64sEqual(t *testing.T, values []float64, expected ...float64) {
	t.Helper()

	assert.Len(t, values, len(expected))
	for i, v := range expected {
		assert.Equal(t, v, values[i])
	}
}

func intsEqual(t *testing.T, values []int, expected ...int) {
	t.Helper()

	assert.Len(t, values, len(expected))
	for i, v := range expected {
		assert.Equal(t, v, values[i])
	}
}

func int64sEqual(t *testing.T, values []int64, expected ...int64) {
	t.Helper()

	assert.Len(t, values, len(expected))
	for i, v := range expected {
		assert.Equal(t, v, values[i])
	}
}

func uintsEqual(t *testing.T, values []uint, expected ...uint) {
	t.Helper()

	assert.Len(t, values, len(expected))
	for i, v := range expected {
		assert.Equal(t, v, values[i])
	}
}

func uint64sEqual(t *testing.T, values []uint64, expected ...uint64) {
	t.Helper()

	assert.Len(t, values, len(expected))
	for i, v := range expected {
		assert.Equal(t, v, values[i])
	}
}

func boolsEqual(t *testing.T, values []bool, expected ...bool) {
	t.Helper()

	assert.Len(t, values, len(expected))
	for i, v := range expected {
		assert.Equal(t, v, values[i])
	}
}

func timesEqual(t *testing.T, values []time.Time, expected ...time.Time) {
	t.Helper()

	assert.Len(t, values, len(expected))
	for i, v := range expected {
		assert.Equal(t, v.String(), values[i].String())
	}
}

func TestKey_Helpers(t *testing.T) {
	t.Run("getting and setting values", func(t *testing.T) {
		f, err := Load(fullConf)
		require.NoError(t, err)
		require.NotNil(t, f)

		t.Run("get string representation", func(t *testing.T) {
			sec := f.Section("")
			require.NotNil(t, sec)
			assert.Equal(t, "ini", sec.Key("NAME").Value())
			assert.Equal(t, "ini", sec.Key("NAME").String())
			assert.Equal(t, "ini", sec.Key("NAME").Validate(func(in string) string {
				return in
			}))
			assert.Equal(t, "; Package name", sec.Key("NAME").Comment)
			assert.Equal(t, "gopkg.in/ini.v1", sec.Key("IMPORT_PATH").String())

			t.Run("with ValueMapper", func(t *testing.T) {
				f.ValueMapper = func(in string) string {
					if in == "gopkg.in/%(NAME)s.%(VERSION)s" {
						return "github.com/go-ini/ini"
					}
					return in
				}
				assert.Equal(t, "github.com/go-ini/ini", sec.Key("IMPORT_PATH").String())
			})
		})

		t.Run("get values in non-default section", func(t *testing.T) {
			sec := f.Section("author")
			require.NotNil(t, sec)
			assert.Equal(t, "Unknwon", sec.Key("NAME").String())
			assert.Equal(t, "https://github.com/Unknwon", sec.Key("GITHUB").String())

			sec = f.Section("package")
			require.NotNil(t, sec)
			assert.Equal(t, "https://gopkg.in/ini.v1", sec.Key("CLONE_URL").String())
		})

		t.Run("get auto-increment key names", func(t *testing.T) {
			keys := f.Section("features").Keys()
			for i, k := range keys {
				assert.Equal(t, fmt.Sprintf("#%d", i+1), k.Name())
			}
		})

		t.Run("get parent-keys that are available to the child section", func(t *testing.T) {
			parentKeys := f.Section("package.sub").ParentKeys()
			for _, k := range parentKeys {
				assert.Equal(t, "CLONE_URL", k.Name())
			}
		})

		t.Run("get overwrite value", func(t *testing.T) {
			assert.Equal(t, "u@gogs.io", f.Section("author").Key("E-MAIL").String())
		})

		t.Run("get sections", func(t *testing.T) {
			sections := f.Sections()
			for i, name := range []string{DefaultSection, "author", "package", "package.sub", "features", "types", "array", "note", "comments", "string escapes", "advance"} {
				assert.Equal(t, name, sections[i].Name())
			}
		})

		t.Run("get parent section value", func(t *testing.T) {
			assert.Equal(t, "https://gopkg.in/ini.v1", f.Section("package.sub").Key("CLONE_URL").String())
			assert.Equal(t, "https://gopkg.in/ini.v1", f.Section("package.fake.sub").Key("CLONE_URL").String())
		})

		t.Run("get multiple line value", func(t *testing.T) {
			if runtime.GOOS == "windows" {
				t.Skip("Skipping testing on Windows")
			}

			assert.Equal(t, "Gopher.\nCoding addict.\nGood man.\n", f.Section("author").Key("BIO").String())
		})

		t.Run("get values with type", func(t *testing.T) {
			sec := f.Section("types")
			v1, err := sec.Key("BOOL").Bool()
			require.NoError(t, err)
			assert.True(t, v1)

			v1, err = sec.Key("BOOL_FALSE").Bool()
			require.NoError(t, err)
			assert.False(t, v1)

			v2, err := sec.Key("FLOAT64").Float64()
			require.NoError(t, err)
			assert.Equal(t, 1.25, v2)

			v3, err := sec.Key("INT").Int()
			require.NoError(t, err)
			assert.Equal(t, 10, v3)

			v4, err := sec.Key("INT").Int64()
			require.NoError(t, err)
			assert.Equal(t, int64(10), v4)

			v5, err := sec.Key("UINT").Uint()
			require.NoError(t, err)
			assert.Equal(t, uint(3), v5)

			v6, err := sec.Key("UINT").Uint64()
			require.NoError(t, err)
			assert.Equal(t, uint64(3), v6)

			ti, err := time.Parse(time.RFC3339, "2015-01-01T20:17:05Z")
			require.NoError(t, err)
			v7, err := sec.Key("TIME").Time()
			require.NoError(t, err)
			assert.Equal(t, ti.String(), v7.String())

			v8, err := sec.Key("HEX_NUMBER").Int()
			require.NoError(t, err)
			assert.Equal(t, 0x3000, v8)

			t.Run("must get values with type", func(t *testing.T) {
				assert.Equal(t, "str", sec.Key("STRING").MustString("404"))
				assert.True(t, sec.Key("BOOL").MustBool())
				assert.Equal(t, float64(1.25), sec.Key("FLOAT64").MustFloat64())
				assert.Equal(t, int(10), sec.Key("INT").MustInt())
				assert.Equal(t, int64(10), sec.Key("INT").MustInt64())
				assert.Equal(t, uint(3), sec.Key("UINT").MustUint())
				assert.Equal(t, uint64(3), sec.Key("UINT").MustUint64())
				assert.Equal(t, ti.String(), sec.Key("TIME").MustTime().String())
				assert.Equal(t, 0x3000, sec.Key("HEX_NUMBER").MustInt())

				dur, err := time.ParseDuration("2h45m")
				require.NoError(t, err)
				assert.Equal(t, dur.Seconds(), sec.Key("DURATION").MustDuration().Seconds())

				t.Run("must get values with default value", func(t *testing.T) {
					assert.Equal(t, "404", sec.Key("STRING_404").MustString("404"))
					assert.True(t, sec.Key("BOOL_404").MustBool(true))
					assert.Equal(t, float64(2.5), sec.Key("FLOAT64_404").MustFloat64(2.5))
					assert.Equal(t, int(15), sec.Key("INT_404").MustInt(15))
					assert.Equal(t, int64(15), sec.Key("INT64_404").MustInt64(15))
					assert.Equal(t, uint(6), sec.Key("UINT_404").MustUint(6))
					assert.Equal(t, uint64(6), sec.Key("UINT64_404").MustUint64(6))
					assert.Equal(t, 0x3001, sec.Key("HEX_NUMBER_404").MustInt(0x3001))

					ti, err := time.Parse(time.RFC3339, "2014-01-01T20:17:05Z")
					require.NoError(t, err)
					assert.Equal(t, ti.String(), sec.Key("TIME_404").MustTime(ti).String())

					assert.Equal(t, dur.Seconds(), sec.Key("DURATION_404").MustDuration(dur).Seconds())

					t.Run("must should set default as key value", func(t *testing.T) {
						assert.Equal(t, "404", sec.Key("STRING_404").String())
						assert.Equal(t, "true", sec.Key("BOOL_404").String())
						assert.Equal(t, "2.5", sec.Key("FLOAT64_404").String())
						assert.Equal(t, "15", sec.Key("INT_404").String())
						assert.Equal(t, "15", sec.Key("INT64_404").String())
						assert.Equal(t, "6", sec.Key("UINT_404").String())
						assert.Equal(t, "6", sec.Key("UINT64_404").String())
						assert.Equal(t, "2014-01-01T20:17:05Z", sec.Key("TIME_404").String())
						assert.Equal(t, "2h45m0s", sec.Key("DURATION_404").String())
						assert.Equal(t, "12289", sec.Key("HEX_NUMBER_404").String())
					})
				})
			})
		})

		t.Run("get value with candidates", func(t *testing.T) {
			sec := f.Section("types")
			assert.Equal(t, "str", sec.Key("STRING").In("", []string{"str", "arr", "types"}))
			assert.Equal(t, float64(1.25), sec.Key("FLOAT64").InFloat64(0, []float64{1.25, 2.5, 3.75}))
			assert.Equal(t, int(10), sec.Key("INT").InInt(0, []int{10, 20, 30}))
			assert.Equal(t, int64(10), sec.Key("INT").InInt64(0, []int64{10, 20, 30}))
			assert.Equal(t, uint(3), sec.Key("UINT").InUint(0, []uint{3, 6, 9}))
			assert.Equal(t, uint64(3), sec.Key("UINT").InUint64(0, []uint64{3, 6, 9}))

			zt, err := time.Parse(time.RFC3339, "0001-01-01T01:00:00Z")
			require.NoError(t, err)
			ti, err := time.Parse(time.RFC3339, "2015-01-01T20:17:05Z")
			require.NoError(t, err)
			assert.Equal(t, ti.String(), sec.Key("TIME").InTime(zt, []time.Time{ti, time.Now(), time.Now().Add(1 * time.Second)}).String())

			t.Run("get value with candidates and default value", func(t *testing.T) {
				assert.Equal(t, "str", sec.Key("STRING_404_2").In("str", []string{"str", "arr", "types"}))
				assert.Equal(t, float64(1.25), sec.Key("FLOAT64_404_2").InFloat64(1.25, []float64{1.25, 2.5, 3.75}))
				assert.Equal(t, int(10), sec.Key("INT_404_2").InInt(10, []int{10, 20, 30}))
				assert.Equal(t, int64(10), sec.Key("INT64_404_2").InInt64(10, []int64{10, 20, 30}))
				assert.Equal(t, uint(3), sec.Key("UINT_404_2").InUint(3, []uint{3, 6, 9}))
				assert.Equal(t, uint64(3), sec.Key("UINT_404_2").InUint64(3, []uint64{3, 6, 9}))
				assert.Equal(t, ti.String(), sec.Key("TIME_404_2").InTime(ti, []time.Time{time.Now(), time.Now(), time.Now().Add(1 * time.Second)}).String())
			})
		})

		t.Run("get values in range", func(t *testing.T) {
			sec := f.Section("types")
			assert.Equal(t, float64(1.25), sec.Key("FLOAT64").RangeFloat64(0, 1, 2))
			assert.Equal(t, int(10), sec.Key("INT").RangeInt(0, 10, 20))
			assert.Equal(t, int64(10), sec.Key("INT").RangeInt64(0, 10, 20))

			minT, err := time.Parse(time.RFC3339, "0001-01-01T01:00:00Z")
			require.NoError(t, err)
			midT, err := time.Parse(time.RFC3339, "2013-01-01T01:00:00Z")
			require.NoError(t, err)
			maxT, err := time.Parse(time.RFC3339, "9999-01-01T01:00:00Z")
			require.NoError(t, err)
			ti, err := time.Parse(time.RFC3339, "2015-01-01T20:17:05Z")
			require.NoError(t, err)
			assert.Equal(t, ti.String(), sec.Key("TIME").RangeTime(ti, minT, maxT).String())

			t.Run("get value in range with default value", func(t *testing.T) {
				assert.Equal(t, float64(5), sec.Key("FLOAT64").RangeFloat64(5, 0, 1))
				assert.Equal(t, 7, sec.Key("INT").RangeInt(7, 0, 5))
				assert.Equal(t, int64(7), sec.Key("INT").RangeInt64(7, 0, 5))
				assert.Equal(t, ti.String(), sec.Key("TIME").RangeTime(ti, minT, midT).String())
			})
		})

		t.Run("get values into slice", func(t *testing.T) {
			sec := f.Section("array")
			assert.Equal(t, "en,zh,de", strings.Join(sec.Key("STRINGS").Strings(","), ","))
			assert.Equal(t, 0, len(sec.Key("STRINGS_404").Strings(",")))

			vals1 := sec.Key("FLOAT64S").Float64s(",")
			float64sEqual(t, vals1, 1.1, 2.2, 3.3)

			vals2 := sec.Key("INTS").Ints(",")
			intsEqual(t, vals2, 1, 2, 3)

			vals3 := sec.Key("INTS").Int64s(",")
			int64sEqual(t, vals3, 1, 2, 3)

			vals4 := sec.Key("UINTS").Uints(",")
			uintsEqual(t, vals4, 1, 2, 3)

			vals5 := sec.Key("UINTS").Uint64s(",")
			uint64sEqual(t, vals5, 1, 2, 3)

			vals6 := sec.Key("BOOLS").Bools(",")
			boolsEqual(t, vals6, true, false, false)

			ti, err := time.Parse(time.RFC3339, "2015-01-01T20:17:05Z")
			require.NoError(t, err)
			vals7 := sec.Key("TIMES").Times(",")
			timesEqual(t, vals7, ti, ti, ti)
		})

		t.Run("test string slice escapes", func(t *testing.T) {
			sec := f.Section("string escapes")
			assert.Equal(t, []string{"value1", "value2", "value3"}, sec.Key("key1").Strings(","))
			assert.Equal(t, []string{"value1, value2"}, sec.Key("key2").Strings(","))
			assert.Equal(t, []string{`val\ue1`, "value2"}, sec.Key("key3").Strings(","))
			assert.Equal(t, []string{`value1\`, `value\\2`}, sec.Key("key4").Strings(","))
			assert.Equal(t, []string{"value1,, value2"}, sec.Key("key5").Strings(",,"))
			assert.Equal(t, []string{"aaa", "bbb and space", "ccc"}, sec.Key("key6").Strings(" "))
		})

		t.Run("get valid values into slice", func(t *testing.T) {
			sec := f.Section("array")
			vals1 := sec.Key("FLOAT64S").ValidFloat64s(",")
			float64sEqual(t, vals1, 1.1, 2.2, 3.3)

			vals2 := sec.Key("INTS").ValidInts(",")
			intsEqual(t, vals2, 1, 2, 3)

			vals3 := sec.Key("INTS").ValidInt64s(",")
			int64sEqual(t, vals3, 1, 2, 3)

			vals4 := sec.Key("UINTS").ValidUints(",")
			uintsEqual(t, vals4, 1, 2, 3)

			vals5 := sec.Key("UINTS").ValidUint64s(",")
			uint64sEqual(t, vals5, 1, 2, 3)

			vals6 := sec.Key("BOOLS").ValidBools(",")
			boolsEqual(t, vals6, true, false, false)

			ti, err := time.Parse(time.RFC3339, "2015-01-01T20:17:05Z")
			require.NoError(t, err)
			vals7 := sec.Key("TIMES").ValidTimes(",")
			timesEqual(t, vals7, ti, ti, ti)
		})

		t.Run("get values one type into slice of another type", func(t *testing.T) {
			sec := f.Section("array")
			vals1 := sec.Key("STRINGS").ValidFloat64s(",")
			assert.Empty(t, vals1)

			vals2 := sec.Key("STRINGS").ValidInts(",")
			assert.Empty(t, vals2)

			vals3 := sec.Key("STRINGS").ValidInt64s(",")
			assert.Empty(t, vals3)

			vals4 := sec.Key("STRINGS").ValidUints(",")
			assert.Empty(t, vals4)

			vals5 := sec.Key("STRINGS").ValidUint64s(",")
			assert.Empty(t, vals5)

			vals6 := sec.Key("STRINGS").ValidBools(",")
			assert.Empty(t, vals6)

			vals7 := sec.Key("STRINGS").ValidTimes(",")
			assert.Empty(t, vals7)
		})

		t.Run("get valid values into slice without errors", func(t *testing.T) {
			sec := f.Section("array")
			vals1, err := sec.Key("FLOAT64S").StrictFloat64s(",")
			require.NoError(t, err)
			float64sEqual(t, vals1, 1.1, 2.2, 3.3)

			vals2, err := sec.Key("INTS").StrictInts(",")
			require.NoError(t, err)
			intsEqual(t, vals2, 1, 2, 3)

			vals3, err := sec.Key("INTS").StrictInt64s(",")
			require.NoError(t, err)
			int64sEqual(t, vals3, 1, 2, 3)

			vals4, err := sec.Key("UINTS").StrictUints(",")
			require.NoError(t, err)
			uintsEqual(t, vals4, 1, 2, 3)

			vals5, err := sec.Key("UINTS").StrictUint64s(",")
			require.NoError(t, err)
			uint64sEqual(t, vals5, 1, 2, 3)

			vals6, err := sec.Key("BOOLS").StrictBools(",")
			require.NoError(t, err)
			boolsEqual(t, vals6, true, false, false)

			ti, err := time.Parse(time.RFC3339, "2015-01-01T20:17:05Z")
			require.NoError(t, err)
			vals7, err := sec.Key("TIMES").StrictTimes(",")
			require.NoError(t, err)
			timesEqual(t, vals7, ti, ti, ti)
		})

		t.Run("get invalid values into slice", func(t *testing.T) {
			sec := f.Section("array")
			vals1, err := sec.Key("STRINGS").StrictFloat64s(",")
			assert.Empty(t, vals1)
			assert.Error(t, err)

			vals2, err := sec.Key("STRINGS").StrictInts(",")
			assert.Empty(t, vals2)
			assert.Error(t, err)

			vals3, err := sec.Key("STRINGS").StrictInt64s(",")
			assert.Empty(t, vals3)
			assert.Error(t, err)

			vals4, err := sec.Key("STRINGS").StrictUints(",")
			assert.Empty(t, vals4)
			assert.Error(t, err)

			vals5, err := sec.Key("STRINGS").StrictUint64s(",")
			assert.Empty(t, vals5)
			assert.Error(t, err)

			vals6, err := sec.Key("STRINGS").StrictBools(",")
			assert.Empty(t, vals6)
			assert.Error(t, err)

			vals7, err := sec.Key("STRINGS").StrictTimes(",")
			assert.Empty(t, vals7)
			assert.Error(t, err)
		})
	})
}

func TestKey_ValueWithShadows(t *testing.T) {
	t.Run("", func(t *testing.T) {
		f, err := ShadowLoad([]byte(`
keyName = value1
keyName = value2
`))
		require.NoError(t, err)
		require.NotNil(t, f)

		k := f.Section("").Key("FakeKey")
		require.NotNil(t, k)
		assert.Equal(t, []string{}, k.ValueWithShadows())

		k = f.Section("").Key("keyName")
		require.NotNil(t, k)
		assert.Equal(t, []string{"value1", "value2"}, k.ValueWithShadows())
	})
}

func TestKey_StringsWithShadows(t *testing.T) {
	t.Run("get strings of shadows of a key", func(t *testing.T) {
		f, err := ShadowLoad([]byte(""))
		require.NoError(t, err)
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NUMS", "1,2")
		require.NoError(t, err)
		require.NotNil(t, k)
		k, err = f.Section("").NewKey("NUMS", "4,5,6")
		require.NoError(t, err)
		require.NotNil(t, k)

		assert.Equal(t, []string{"1", "2", "4", "5", "6"}, k.StringsWithShadows(","))
	})
}

func TestKey_SetValue(t *testing.T) {
	t.Run("set value of key", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)
		assert.Equal(t, "ini", k.Value())

		k.SetValue("ini.v1")
		assert.Equal(t, "ini.v1", k.Value())
	})
}

func TestKey_NestedValues(t *testing.T) {
	t.Run("read and write nested values", func(t *testing.T) {
		f, err := LoadSources(LoadOptions{
			AllowNestedValues: true,
		}, []byte(`
aws_access_key_id = foo
aws_secret_access_key = bar
region = us-west-2
s3 =
  max_concurrent_requests=10
  max_queue_size=1000`))
		require.NoError(t, err)
		require.NotNil(t, f)

		assert.Equal(t, []string{"max_concurrent_requests=10", "max_queue_size=1000"}, f.Section("").Key("s3").NestedValues())

		var buf bytes.Buffer
		_, err = f.WriteTo(&buf)
		require.NoError(t, err)
		assert.Equal(t, `aws_access_key_id     = foo
aws_secret_access_key = bar
region                = us-west-2
s3                    = 
  max_concurrent_requests=10
  max_queue_size=1000
`,
			buf.String(),
		)
	})
}

func TestRecursiveValues(t *testing.T) {
	t.Run("recursive values should not reflect on same key", func(t *testing.T) {
		f, err := Load([]byte(`
NAME = ini
expires = yes
[package]
NAME = %(NAME)s
expires = %(expires)s`))
		require.NoError(t, err)
		require.NotNil(t, f)

		assert.Equal(t, "ini", f.Section("package").Key("NAME").String())
		assert.Equal(t, "yes", f.Section("package").Key("expires").String())
	})

	t.Run("recursive value with no target found", func(t *testing.T) {
		f, err := Load([]byte(`
[foo]
bar = %(missing)s
`))
		require.NoError(t, err)
		require.NotNil(t, f)

		assert.Equal(t, "%(missing)s", f.Section("foo").Key("bar").String())
	})
}
