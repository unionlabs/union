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
	"bytes"
	"io/ioutil"
	"runtime"
	"sort"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestEmpty(t *testing.T) {
	f := Empty()
	require.NotNil(t, f)

	// Should only have the default section
	assert.Len(t, f.Sections(), 1)

	// Default section should not contain any key
	assert.Len(t, f.Section("").Keys(), 0)
}

func TestFile_NewSection(t *testing.T) {
	f := Empty()
	require.NotNil(t, f)

	sec, err := f.NewSection("author")
	require.NoError(t, err)
	require.NotNil(t, sec)
	assert.Equal(t, "author", sec.Name())

	assert.Equal(t, []string{DefaultSection, "author"}, f.SectionStrings())

	t.Run("with duplicated name", func(t *testing.T) {
		sec, err := f.NewSection("author")
		require.NoError(t, err)
		require.NotNil(t, sec)

		// Does nothing if section already exists
		assert.Equal(t, []string{DefaultSection, "author"}, f.SectionStrings())
	})

	t.Run("with empty string", func(t *testing.T) {
		_, err := f.NewSection("")
		require.Error(t, err)
	})
}

func TestFile_NonUniqueSection(t *testing.T) {
	t.Run("read and write non-unique sections", func(t *testing.T) {
		f, err := LoadSources(LoadOptions{
			AllowNonUniqueSections: true,
		}, []byte(`[Interface]
Address = 192.168.2.1
PrivateKey = <server's privatekey>
ListenPort = 51820

[Peer]
PublicKey = <client's publickey>
AllowedIPs = 192.168.2.2/32

[Peer]
PublicKey = <client2's publickey>
AllowedIPs = 192.168.2.3/32`))
		require.NoError(t, err)
		require.NotNil(t, f)

		sec, err := f.NewSection("Peer")
		require.NoError(t, err)
		require.NotNil(t, f)

		_, _ = sec.NewKey("PublicKey", "<client3's publickey>")
		_, _ = sec.NewKey("AllowedIPs", "192.168.2.4/32")

		var buf bytes.Buffer
		_, err = f.WriteTo(&buf)
		require.NoError(t, err)
		str := buf.String()
		assert.Equal(t, `[Interface]
Address    = 192.168.2.1
PrivateKey = <server's privatekey>
ListenPort = 51820

[Peer]
PublicKey  = <client's publickey>
AllowedIPs = 192.168.2.2/32

[Peer]
PublicKey  = <client2's publickey>
AllowedIPs = 192.168.2.3/32

[Peer]
PublicKey  = <client3's publickey>
AllowedIPs = 192.168.2.4/32
`, str)
	})

	t.Run("delete non-unique section", func(t *testing.T) {
		f, err := LoadSources(LoadOptions{
			AllowNonUniqueSections: true,
		}, []byte(`[Interface]
Address    = 192.168.2.1
PrivateKey = <server's privatekey>
ListenPort = 51820

[Peer]
PublicKey  = <client's publickey>
AllowedIPs = 192.168.2.2/32

[Peer]
PublicKey  = <client2's publickey>
AllowedIPs = 192.168.2.3/32

[Peer]
PublicKey  = <client3's publickey>
AllowedIPs = 192.168.2.4/32

`))
		require.NoError(t, err)
		require.NotNil(t, f)

		err = f.DeleteSectionWithIndex("Peer", 1)
		require.NoError(t, err)

		var buf bytes.Buffer
		_, err = f.WriteTo(&buf)
		require.NoError(t, err)
		str := buf.String()
		assert.Equal(t, `[Interface]
Address    = 192.168.2.1
PrivateKey = <server's privatekey>
ListenPort = 51820

[Peer]
PublicKey  = <client's publickey>
AllowedIPs = 192.168.2.2/32

[Peer]
PublicKey  = <client3's publickey>
AllowedIPs = 192.168.2.4/32
`, str)
	})

	t.Run("delete all sections", func(t *testing.T) {
		f := Empty(LoadOptions{
			AllowNonUniqueSections: true,
		})
		require.NotNil(t, f)

		_ = f.NewSections("Interface", "Peer", "Peer")
		assert.Equal(t, []string{DefaultSection, "Interface", "Peer", "Peer"}, f.SectionStrings())
		f.DeleteSection("Peer")
		assert.Equal(t, []string{DefaultSection, "Interface"}, f.SectionStrings())
	})
}

func TestFile_NewRawSection(t *testing.T) {
	f := Empty()
	require.NotNil(t, f)

	sec, err := f.NewRawSection("comments", `1111111111111111111000000000000000001110000
111111111111111111100000000000111000000000`)
	require.NoError(t, err)
	require.NotNil(t, sec)
	assert.Equal(t, "comments", sec.Name())

	assert.Equal(t, []string{DefaultSection, "comments"}, f.SectionStrings())
	assert.Equal(t, `1111111111111111111000000000000000001110000
111111111111111111100000000000111000000000`, f.Section("comments").Body())

	t.Run("with duplicated name", func(t *testing.T) {
		sec, err := f.NewRawSection("comments", `1111111111111111111000000000000000001110000`)
		require.NoError(t, err)
		require.NotNil(t, sec)
		assert.Equal(t, []string{DefaultSection, "comments"}, f.SectionStrings())

		// Overwrite previous existed section
		assert.Equal(t, `1111111111111111111000000000000000001110000`, f.Section("comments").Body())
	})

	t.Run("with empty string", func(t *testing.T) {
		_, err := f.NewRawSection("", "")
		require.Error(t, err)
	})
}

func TestFile_NewSections(t *testing.T) {
	f := Empty()
	require.NotNil(t, f)

	assert.NoError(t, f.NewSections("package", "author"))
	assert.Equal(t, []string{DefaultSection, "package", "author"}, f.SectionStrings())

	t.Run("with duplicated name", func(t *testing.T) {
		assert.NoError(t, f.NewSections("author", "features"))

		// Ignore section already exists
		assert.Equal(t, []string{DefaultSection, "package", "author", "features"}, f.SectionStrings())
	})

	t.Run("with empty string", func(t *testing.T) {
		assert.Error(t, f.NewSections("", ""))
	})
}

func TestFile_GetSection(t *testing.T) {
	f, err := Load(fullConf)
	require.NoError(t, err)
	require.NotNil(t, f)

	sec, err := f.GetSection("author")
	require.NoError(t, err)
	require.NotNil(t, sec)
	assert.Equal(t, "author", sec.Name())

	t.Run("section not exists", func(t *testing.T) {
		_, err := f.GetSection("404")
		require.Error(t, err)
	})
}

func TestFile_HasSection(t *testing.T) {
	f, err := Load(fullConf)
	require.NoError(t, err)
	require.NotNil(t, f)

	sec := f.HasSection("author")
	assert.True(t, sec)

	t.Run("section not exists", func(t *testing.T) {
		nonexistent := f.HasSection("404")
		assert.False(t, nonexistent)
	})
}

func TestFile_Section(t *testing.T) {
	t.Run("get a section", func(t *testing.T) {
		f, err := Load(fullConf)
		require.NoError(t, err)
		require.NotNil(t, f)

		sec := f.Section("author")
		require.NotNil(t, sec)
		assert.Equal(t, "author", sec.Name())

		t.Run("section not exists", func(t *testing.T) {
			sec := f.Section("404")
			require.NotNil(t, sec)
			assert.Equal(t, "404", sec.Name())
		})
	})

	t.Run("get default section in lower case with insensitive load", func(t *testing.T) {
		f, err := InsensitiveLoad([]byte(`
[default]
NAME = ini
VERSION = v1`))
		require.NoError(t, err)
		require.NotNil(t, f)

		assert.Equal(t, "ini", f.Section("").Key("name").String())
		assert.Equal(t, "v1", f.Section("").Key("version").String())
	})

	t.Run("get sections after deletion", func(t *testing.T) {
		f, err := Load([]byte(`
[RANDOM]
`))
		require.NoError(t, err)
		require.NotNil(t, f)

		sectionNames := f.SectionStrings()
		sort.Strings(sectionNames)
		assert.Equal(t, []string{DefaultSection, "RANDOM"}, sectionNames)

		for _, currentSection := range sectionNames {
			f.DeleteSection(currentSection)
		}

		for sectionParam, expectedSectionName := range map[string]string{
			"":       DefaultSection,
			"RANDOM": "RANDOM",
		} {
			sec := f.Section(sectionParam)
			require.NotNil(t, sec)
			assert.Equal(t, expectedSectionName, sec.Name())
		}
	})

}

func TestFile_Sections(t *testing.T) {
	f, err := Load(fullConf)
	require.NoError(t, err)
	require.NotNil(t, f)

	secs := f.Sections()
	names := []string{DefaultSection, "author", "package", "package.sub", "features", "types", "array", "note", "comments", "string escapes", "advance"}
	assert.Len(t, secs, len(names))
	for i, name := range names {
		assert.Equal(t, name, secs[i].Name())
	}
}

func TestFile_ChildSections(t *testing.T) {
	f, err := Load([]byte(`
[node]
[node.biz1]
[node.biz2]
[node.biz3]
[node.bizN]
`))
	require.NoError(t, err)
	require.NotNil(t, f)

	children := f.ChildSections("node")
	names := []string{"node.biz1", "node.biz2", "node.biz3", "node.bizN"}
	assert.Len(t, children, len(names))
	for i, name := range names {
		assert.Equal(t, name, children[i].Name())
	}
}

func TestFile_SectionStrings(t *testing.T) {
	f, err := Load(fullConf)
	require.NoError(t, err)
	require.NotNil(t, f)

	assert.Equal(t, []string{DefaultSection, "author", "package", "package.sub", "features", "types", "array", "note", "comments", "string escapes", "advance"}, f.SectionStrings())
}

func TestFile_DeleteSection(t *testing.T) {
	t.Run("delete a section", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		_ = f.NewSections("author", "package", "features")
		f.DeleteSection("features")
		f.DeleteSection("")
		assert.Equal(t, []string{"author", "package"}, f.SectionStrings())
	})

	t.Run("delete default section", func(t *testing.T) {
		f := Empty()
		require.NotNil(t, f)

		f.Section("").Key("foo").SetValue("bar")
		f.Section("section1").Key("key1").SetValue("value1")
		f.DeleteSection("")
		assert.Equal(t, []string{"section1"}, f.SectionStrings())

		var buf bytes.Buffer
		_, err := f.WriteTo(&buf)
		require.NoError(t, err)

		assert.Equal(t, `[section1]
key1 = value1
`, buf.String())
	})

	t.Run("delete a section with InsensitiveSections", func(t *testing.T) {
		f := Empty(LoadOptions{InsensitiveSections: true})
		require.NotNil(t, f)

		_ = f.NewSections("author", "package", "features")
		f.DeleteSection("FEATURES")
		f.DeleteSection("")
		assert.Equal(t, []string{"author", "package"}, f.SectionStrings())
	})
}

func TestFile_Append(t *testing.T) {
	f := Empty()
	require.NotNil(t, f)

	assert.NoError(t, f.Append(minimalConf, []byte(`
[author]
NAME = Unknwon`)))

	t.Run("with bad input", func(t *testing.T) {
		assert.Error(t, f.Append(123))
		assert.Error(t, f.Append(minimalConf, 123))
	})
}

func TestFile_WriteTo(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("Skipping testing on Windows")
	}

	t.Run("write content to somewhere", func(t *testing.T) {
		f, err := Load(fullConf)
		require.NoError(t, err)
		require.NotNil(t, f)

		f.Section("author").Comment = `Information about package author
# Bio can be written in multiple lines.`
		f.Section("author").Key("NAME").Comment = "This is author name"
		_, _ = f.Section("note").NewBooleanKey("boolean_key")
		_, _ = f.Section("note").NewKey("more", "notes")

		var buf bytes.Buffer
		_, err = f.WriteTo(&buf)
		require.NoError(t, err)

		golden := "testdata/TestFile_WriteTo.golden"
		if *update {
			require.NoError(t, ioutil.WriteFile(golden, buf.Bytes(), 0644))
		}

		expected, err := ioutil.ReadFile(golden)
		require.NoError(t, err)
		assert.Equal(t, string(expected), buf.String())
	})

	t.Run("support multiline comments", func(t *testing.T) {
		f, err := Load([]byte(`
# 
# general.domain
# 
# Domain name of XX system.
domain      = mydomain.com
`))
		require.NoError(t, err)

		f.Section("").Key("test").Comment = "Multiline\nComment"

		var buf bytes.Buffer
		_, err = f.WriteTo(&buf)
		require.NoError(t, err)

		assert.Equal(t, `# 
# general.domain
# 
# Domain name of XX system.
domain = mydomain.com
; Multiline
; Comment
test   = 
`, buf.String())

	})

	t.Run("keep leading and trailing spaces in value", func(t *testing.T) {
		f, _ := Load([]byte(`[foo]
bar1 = '  val ue1 '
bar2 = """  val ue2 """
bar3 = "  val ue3 "
`))
		require.NotNil(t, f)

		var buf bytes.Buffer
		_, err := f.WriteTo(&buf)
		require.NoError(t, err)
		assert.Equal(t, `[foo]
bar1 = "  val ue1 "
bar2 = "  val ue2 "
bar3 = "  val ue3 "
`, buf.String())
	})
}

func TestFile_SaveTo(t *testing.T) {
	f, err := Load(fullConf)
	require.NoError(t, err)
	require.NotNil(t, f)

	assert.NoError(t, f.SaveTo("testdata/conf_out.ini"))
	assert.NoError(t, f.SaveToIndent("testdata/conf_out.ini", "\t"))
}

func TestFile_WriteToWithOutputDelimiter(t *testing.T) {
	f, err := LoadSources(LoadOptions{
		KeyValueDelimiterOnWrite: "->",
	}, []byte(`[Others]
Cities = HangZhou|Boston
Visits = 1993-10-07T20:17:05Z, 1993-10-07T20:17:05Z
Years = 1993,1994
Numbers = 10010,10086
Ages = 18,19
Populations = 12345678,98765432
Coordinates = 192.168,10.11
Flags       = true,false
Note = Hello world!`))
	require.NoError(t, err)
	require.NotNil(t, f)

	var actual bytes.Buffer
	var expected = []byte(`[Others]
Cities      -> HangZhou|Boston
Visits      -> 1993-10-07T20:17:05Z, 1993-10-07T20:17:05Z
Years       -> 1993,1994
Numbers     -> 10010,10086
Ages        -> 18,19
Populations -> 12345678,98765432
Coordinates -> 192.168,10.11
Flags       -> true,false
Note        -> Hello world!
`)
	_, err = f.WriteTo(&actual)
	require.NoError(t, err)

	assert.Equal(t, expected, actual.Bytes())
}

// Inspired by https://github.com/go-ini/ini/issues/207
func TestReloadAfterShadowLoad(t *testing.T) {
	f, err := ShadowLoad([]byte(`
[slice]
v = 1
v = 2
v = 3
`))
	require.NoError(t, err)
	require.NotNil(t, f)

	assert.Equal(t, []string{"1", "2", "3"}, f.Section("slice").Key("v").ValueWithShadows())

	require.NoError(t, f.Reload())
	assert.Equal(t, []string{"1", "2", "3"}, f.Section("slice").Key("v").ValueWithShadows())
}
