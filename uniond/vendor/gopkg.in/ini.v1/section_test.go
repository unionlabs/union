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
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestSection_SetBody(t *testing.T) {
	t.Run("set body of raw section", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		sec, err := f.NewRawSection("comments", `1111111111111111111000000000000000001110000
111111111111111111100000000000111000000000`)
		require.NoError(t, err)
		require.NotNil(t, sec)
		assert.Equal(t, `1111111111111111111000000000000000001110000
111111111111111111100000000000111000000000`, sec.Body())

		sec.SetBody("1111111111111111111000000000000000001110000")
		assert.Equal(t, `1111111111111111111000000000000000001110000`, sec.Body())

		t.Run("set for non-raw section", func(t *testing.T) {
			sec, err := f.NewSection("author")
			require.NoError(t, err)
			require.NotNil(t, sec)
			assert.Empty(t, sec.Body())

			sec.SetBody("1111111111111111111000000000000000001110000")
			assert.Empty(t, sec.Body())
		})
	})
}

func TestSection_NewKey(t *testing.T) {
	t.Run("create a new key", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)
		assert.Equal(t, "NAME", k.Name())
		assert.Equal(t, "ini", k.Value())

		t.Run("with duplicated name", func(t *testing.T) {
			k, err := f.Section("").NewKey("NAME", "ini.v1")
			require.NoError(t, err)
			require.NotNil(t, k)

			// Overwrite previous existed key
			assert.Equal(t, "ini.v1", k.Value())
		})

		t.Run("with empty string", func(t *testing.T) {
			_, err := f.Section("").NewKey("", "")
			require.Error(t, err)
		})
	})

	t.Run("create keys with same name and allow shadow", func(t *testing.T) {
		f, err := ShadowLoad([]byte(""))
		require.NoError(t, err)
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)
		k, err = f.Section("").NewKey("NAME", "ini.v1")
		require.NoError(t, err)
		require.NotNil(t, k)

		assert.Equal(t, []string{"ini", "ini.v1"}, k.ValueWithShadows())
	})
}

func TestSection_NewBooleanKey(t *testing.T) {
	t.Run("create a new boolean key", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		k, err := f.Section("").NewBooleanKey("start-ssh-server")
		require.NoError(t, err)
		require.NotNil(t, k)
		assert.Equal(t, "start-ssh-server", k.Name())
		assert.Equal(t, "true", k.Value())

		t.Run("with empty string", func(t *testing.T) {
			_, err := f.Section("").NewBooleanKey("")
			require.Error(t, err)
		})
	})
}

func TestSection_GetKey(t *testing.T) {
	t.Run("get a key", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)

		k, err = f.Section("").GetKey("NAME")
		require.NoError(t, err)
		require.NotNil(t, k)
		assert.Equal(t, "NAME", k.Name())
		assert.Equal(t, "ini", k.Value())

		t.Run("key not exists", func(t *testing.T) {
			_, err := f.Section("").GetKey("404")
			require.Error(t, err)
		})

		t.Run("key exists in parent section", func(t *testing.T) {
			k, err := f.Section("parent").NewKey("AGE", "18")
			require.NoError(t, err)
			require.NotNil(t, k)

			k, err = f.Section("parent.child.son").GetKey("AGE")
			require.NoError(t, err)
			require.NotNil(t, k)
			assert.Equal(t, "18", k.Value())
		})
	})
}

func TestSection_HasKey(t *testing.T) {
	t.Run("check if a key exists", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)

		assert.True(t, f.Section("").HasKey("NAME"))
		assert.True(t, f.Section("").HasKey("NAME"))
		assert.False(t, f.Section("").HasKey("404"))
		assert.False(t, f.Section("").HasKey("404"))
	})
}

func TestSection_HasValue(t *testing.T) {
	t.Run("check if contains a value in any key", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)

		assert.True(t, f.Section("").HasValue("ini"))
		assert.False(t, f.Section("").HasValue("404"))
	})
}

func TestSection_Key(t *testing.T) {
	t.Run("get a key", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)

		k = f.Section("").Key("NAME")
		require.NotNil(t, k)
		assert.Equal(t, "NAME", k.Name())
		assert.Equal(t, "ini", k.Value())

		t.Run("key not exists", func(t *testing.T) {
			k := f.Section("").Key("404")
			require.NotNil(t, k)
			assert.Equal(t, "404", k.Name())
		})

		t.Run("key exists in parent section", func(t *testing.T) {
			k, err := f.Section("parent").NewKey("AGE", "18")
			require.NoError(t, err)
			require.NotNil(t, k)

			k = f.Section("parent.child.son").Key("AGE")
			require.NotNil(t, k)
			assert.Equal(t, "18", k.Value())
		})
	})
}

func TestSection_Keys(t *testing.T) {
	t.Run("get all keys in a section", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)
		k, err = f.Section("").NewKey("VERSION", "v1")
		require.NoError(t, err)
		require.NotNil(t, k)
		k, err = f.Section("").NewKey("IMPORT_PATH", "gopkg.in/ini.v1")
		require.NoError(t, err)
		require.NotNil(t, k)

		keys := f.Section("").Keys()
		names := []string{"NAME", "VERSION", "IMPORT_PATH"}
		assert.Equal(t, len(names), len(keys))
		for i, name := range names {
			assert.Equal(t, name, keys[i].Name())
		}
	})
}

func TestSection_ParentKeys(t *testing.T) {
	t.Run("get all keys of parent sections", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		k, err := f.Section("package").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)
		k, err = f.Section("package").NewKey("VERSION", "v1")
		require.NoError(t, err)
		require.NotNil(t, k)
		k, err = f.Section("package").NewKey("IMPORT_PATH", "gopkg.in/ini.v1")
		require.NoError(t, err)
		require.NotNil(t, k)

		keys := f.Section("package.sub.sub2").ParentKeys()
		names := []string{"NAME", "VERSION", "IMPORT_PATH"}
		assert.Equal(t, len(names), len(keys))
		for i, name := range names {
			assert.Equal(t, name, keys[i].Name())
		}
	})
}

func TestSection_KeyStrings(t *testing.T) {
	t.Run("get all key names in a section", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)
		k, err = f.Section("").NewKey("VERSION", "v1")
		require.NoError(t, err)
		require.NotNil(t, k)
		k, err = f.Section("").NewKey("IMPORT_PATH", "gopkg.in/ini.v1")
		require.NoError(t, err)
		require.NotNil(t, k)

		assert.Equal(t, []string{"NAME", "VERSION", "IMPORT_PATH"}, f.Section("").KeyStrings())
	})
}

func TestSection_KeyHash(t *testing.T) {
	t.Run("get clone of key hash", func(t *testing.T) {
		f, err := Load([]byte(`
key = one
[log]
name = app
file = a.log
`), []byte(`
key = two
[log]
name = app2
file = b.log
`))
		require.NoError(t, err)
		require.NotNil(t, f)

		assert.Equal(t, "two", f.Section("").Key("key").String())

		hash := f.Section("log").KeysHash()
		relation := map[string]string{
			"name": "app2",
			"file": "b.log",
		}
		for k, v := range hash {
			assert.Equal(t, relation[k], v)
		}
	})
}

func TestSection_DeleteKey(t *testing.T) {
	t.Run("delete a key", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		k, err := f.Section("").NewKey("NAME", "ini")
		require.NoError(t, err)
		require.NotNil(t, k)

		assert.True(t, f.Section("").HasKey("NAME"))
		f.Section("").DeleteKey("NAME")
		assert.False(t, f.Section("").HasKey("NAME"))
	})
}
