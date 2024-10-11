// Copyright © 2014 Steve Francia <spf@spf13.com>.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file.

package viper

import (
	"bytes"
	"encoding/json"
	"io"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"reflect"
	"runtime"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/fsnotify/fsnotify"
	"github.com/mitchellh/mapstructure"
	"github.com/spf13/afero"
	"github.com/spf13/cast"
	"github.com/spf13/pflag"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/spf13/viper/internal/features"
	"github.com/spf13/viper/internal/testutil"
)

// var yamlExample = []byte(`Hacker: true
// name: steve
// hobbies:
//     - skateboarding
//     - snowboarding
//     - go
// clothing:
//     jacket: leather
//     trousers: denim
//     pants:
//         size: large
// age: 35
// eyes : brown
// beard: true
// `)

var yamlExampleWithExtras = []byte(`Existing: true
Bogus: true
`)

type testUnmarshalExtra struct {
	Existing bool
}

var tomlExample = []byte(`
title = "TOML Example"

[owner]
organization = "MongoDB"
Bio = "MongoDB Chief Developer Advocate & Hacker at Large"
dob = 1979-05-27T07:32:00Z # First class dates? Why not?`)

var dotenvExample = []byte(`
TITLE_DOTENV="DotEnv Example"
TYPE_DOTENV=donut
NAME_DOTENV=Cake`)

var jsonExample = []byte(`{
"id": "0001",
"type": "donut",
"name": "Cake",
"ppu": 0.55,
"batters": {
        "batter": [
                { "type": "Regular" },
                { "type": "Chocolate" },
                { "type": "Blueberry" },
                { "type": "Devil's Food" }
            ]
    }
}`)

var hclExample = []byte(`
id = "0001"
type = "donut"
name = "Cake"
ppu = 0.55
foos {
	foo {
		key = 1
	}
	foo {
		key = 2
	}
	foo {
		key = 3
	}
	foo {
		key = 4
	}
}`)

var propertiesExample = []byte(`
p_id: 0001
p_type: donut
p_name: Cake
p_ppu: 0.55
p_batters.batter.type: Regular
`)

var remoteExample = []byte(`{
"id":"0002",
"type":"cronut",
"newkey":"remote"
}`)

var iniExample = []byte(`; Package name
NAME        = ini
; Package version
VERSION     = v1
; Package import path
IMPORT_PATH = gopkg.in/%(NAME)s.%(VERSION)s

# Information about package author
# Bio can be written in multiple lines.
[author]
NAME   = Unknown  ; Succeeding comment
E-MAIL = fake@localhost
GITHUB = https://github.com/%(NAME)s
BIO    = """Gopher.
Coding addict.
Good man.
"""  # Succeeding comment`)

func initConfigs(v *Viper) {
	var r io.Reader
	v.SetConfigType("yaml")
	r = bytes.NewReader(yamlExample)
	v.unmarshalReader(r, v.config)

	v.SetConfigType("json")
	r = bytes.NewReader(jsonExample)
	v.unmarshalReader(r, v.config)

	v.SetConfigType("hcl")
	r = bytes.NewReader(hclExample)
	v.unmarshalReader(r, v.config)

	v.SetConfigType("properties")
	r = bytes.NewReader(propertiesExample)
	v.unmarshalReader(r, v.config)

	v.SetConfigType("toml")
	r = bytes.NewReader(tomlExample)
	v.unmarshalReader(r, v.config)

	v.SetConfigType("env")
	r = bytes.NewReader(dotenvExample)
	v.unmarshalReader(r, v.config)

	v.SetConfigType("json")
	remote := bytes.NewReader(remoteExample)
	v.unmarshalReader(remote, v.kvstore)

	v.SetConfigType("ini")
	r = bytes.NewReader(iniExample)
	v.unmarshalReader(r, v.config)
}

func initConfig(typ, config string, v *Viper) {
	v.SetConfigType(typ)
	r := strings.NewReader(config)

	if err := v.unmarshalReader(r, v.config); err != nil {
		panic(err)
	}
}

// initDirs makes directories for testing.
func initDirs(t *testing.T) (string, string) {
	var (
		testDirs = []string{`a a`, `b`, `C_`}
		config   = `improbable`
	)

	if runtime.GOOS != "windows" {
		testDirs = append(testDirs, `d\d`)
	}

	root := t.TempDir()

	for _, dir := range testDirs {
		innerDir := filepath.Join(root, dir)
		err := os.Mkdir(innerDir, 0o750)
		require.NoError(t, err)

		err = os.WriteFile(
			filepath.Join(innerDir, config+".toml"),
			[]byte(`key = "value is `+dir+`"`+"\n"),
			0o640)
		require.NoError(t, err)
	}

	return root, config
}

// stubs for PFlag Values.
type stringValue string

func newStringValue(val string, p *string) *stringValue {
	*p = val
	return (*stringValue)(p)
}

func (s *stringValue) Set(val string) error {
	*s = stringValue(val)
	return nil
}

func (s *stringValue) Type() string {
	return "string"
}

func (s *stringValue) String() string {
	return string(*s)
}

func TestGetConfigFile(t *testing.T) {
	t.Run("config file set", func(t *testing.T) {
		fs := afero.NewMemMapFs()

		err := fs.Mkdir(testutil.AbsFilePath(t, "/etc/viper"), 0o777)
		require.NoError(t, err)

		_, err = fs.Create(testutil.AbsFilePath(t, "/etc/viper/config.yaml"))
		require.NoError(t, err)

		v := New()

		v.SetFs(fs)
		v.AddConfigPath("/etc/viper")
		v.SetConfigFile(testutil.AbsFilePath(t, "/etc/viper/config.yaml"))

		filename, err := v.getConfigFile()
		assert.Equal(t, testutil.AbsFilePath(t, "/etc/viper/config.yaml"), filename)
		assert.NoError(t, err)
	})

	t.Run("find file", func(t *testing.T) {
		fs := afero.NewMemMapFs()

		err := fs.Mkdir(testutil.AbsFilePath(t, "/etc/viper"), 0o777)
		require.NoError(t, err)

		_, err = fs.Create(testutil.AbsFilePath(t, "/etc/viper/config.yaml"))
		require.NoError(t, err)

		v := New()

		v.SetFs(fs)
		v.AddConfigPath("/etc/viper")

		filename, err := v.getConfigFile()
		assert.Equal(t, testutil.AbsFilePath(t, "/etc/viper/config.yaml"), filename)
		assert.NoError(t, err)
	})

	t.Run("find files only", func(t *testing.T) {
		fs := afero.NewMemMapFs()

		err := fs.Mkdir(testutil.AbsFilePath(t, "/etc/config"), 0o777)
		require.NoError(t, err)

		_, err = fs.Create(testutil.AbsFilePath(t, "/etc/config/config.yaml"))
		require.NoError(t, err)

		v := New()

		v.SetFs(fs)
		v.AddConfigPath("/etc")
		v.AddConfigPath("/etc/config")

		filename, err := v.getConfigFile()
		assert.Equal(t, testutil.AbsFilePath(t, "/etc/config/config.yaml"), filename)
		assert.NoError(t, err)
	})

	t.Run("precedence", func(t *testing.T) {
		fs := afero.NewMemMapFs()

		err := fs.Mkdir(testutil.AbsFilePath(t, "/home/viper"), 0o777)
		require.NoError(t, err)

		_, err = fs.Create(testutil.AbsFilePath(t, "/home/viper/config.zml"))
		require.NoError(t, err)

		err = fs.Mkdir(testutil.AbsFilePath(t, "/etc/viper"), 0o777)
		require.NoError(t, err)

		_, err = fs.Create(testutil.AbsFilePath(t, "/etc/viper/config.bml"))
		require.NoError(t, err)

		err = fs.Mkdir(testutil.AbsFilePath(t, "/var/viper"), 0o777)
		require.NoError(t, err)

		_, err = fs.Create(testutil.AbsFilePath(t, "/var/viper/config.yaml"))
		require.NoError(t, err)

		v := New()

		v.SetFs(fs)
		v.AddConfigPath("/home/viper")
		v.AddConfigPath("/etc/viper")
		v.AddConfigPath("/var/viper")

		filename, err := v.getConfigFile()
		assert.Equal(t, testutil.AbsFilePath(t, "/var/viper/config.yaml"), filename)
		assert.NoError(t, err)
	})

	t.Run("without extension", func(t *testing.T) {
		fs := afero.NewMemMapFs()

		err := fs.Mkdir(testutil.AbsFilePath(t, "/etc/viper"), 0o777)
		require.NoError(t, err)

		_, err = fs.Create(testutil.AbsFilePath(t, "/etc/viper/.dotfilenoext"))
		require.NoError(t, err)

		v := New()

		v.SetFs(fs)
		v.AddConfigPath("/etc/viper")
		v.SetConfigName(".dotfilenoext")
		v.SetConfigType("yaml")

		filename, err := v.getConfigFile()
		assert.Equal(t, testutil.AbsFilePath(t, "/etc/viper/.dotfilenoext"), filename)
		assert.NoError(t, err)
	})

	t.Run("without extension and config type", func(t *testing.T) {
		fs := afero.NewMemMapFs()

		err := fs.Mkdir(testutil.AbsFilePath(t, "/etc/viper"), 0o777)
		require.NoError(t, err)

		_, err = fs.Create(testutil.AbsFilePath(t, "/etc/viper/.dotfilenoext"))
		require.NoError(t, err)

		v := New()

		v.SetFs(fs)
		v.AddConfigPath("/etc/viper")
		v.SetConfigName(".dotfilenoext")

		_, err = v.getConfigFile()
		// unless config type is set, files without extension
		// are not considered
		assert.Error(t, err)
	})
}

func TestReadInConfig(t *testing.T) {
	t.Run("config file set", func(t *testing.T) {
		fs := afero.NewMemMapFs()

		err := fs.Mkdir("/etc/viper", 0o777)
		require.NoError(t, err)

		file, err := fs.Create(testutil.AbsFilePath(t, "/etc/viper/config.yaml"))
		require.NoError(t, err)

		_, err = file.WriteString(`key: value`)
		require.NoError(t, err)

		file.Close()

		v := New()

		v.SetFs(fs)
		v.SetConfigFile(testutil.AbsFilePath(t, "/etc/viper/config.yaml"))

		err = v.ReadInConfig()
		require.NoError(t, err)

		assert.Equal(t, "value", v.Get("key"))
	})

	t.Run("find file", func(t *testing.T) {
		fs := afero.NewMemMapFs()

		err := fs.Mkdir(testutil.AbsFilePath(t, "/etc/viper"), 0o777)
		require.NoError(t, err)

		file, err := fs.Create(testutil.AbsFilePath(t, "/etc/viper/config.yaml"))
		require.NoError(t, err)

		_, err = file.WriteString(`key: value`)
		require.NoError(t, err)

		file.Close()

		v := New()

		v.SetFs(fs)
		v.AddConfigPath("/etc/viper")

		err = v.ReadInConfig()
		require.NoError(t, err)

		assert.Equal(t, "value", v.Get("key"))
	})
}

func TestDefault(t *testing.T) {
	v := New()
	v.SetDefault("age", 45)
	assert.Equal(t, 45, v.Get("age"))

	v.SetDefault("clothing.jacket", "slacks")
	assert.Equal(t, "slacks", v.Get("clothing.jacket"))

	v.SetConfigType("yaml")
	err := v.ReadConfig(bytes.NewBuffer(yamlExample))

	assert.NoError(t, err)
	assert.Equal(t, "leather", v.Get("clothing.jacket"))
}

func TestUnmarshaling(t *testing.T) {
	v := New()
	v.SetConfigType("yaml")
	r := bytes.NewReader(yamlExample)

	v.unmarshalReader(r, v.config)
	assert.True(t, v.InConfig("name"))
	assert.True(t, v.InConfig("clothing.jacket"))
	assert.False(t, v.InConfig("state"))
	assert.False(t, v.InConfig("clothing.hat"))
	assert.Equal(t, "steve", v.Get("name"))
	assert.Equal(t, []any{"skateboarding", "snowboarding", "go"}, v.Get("hobbies"))
	assert.Equal(t, map[string]any{"jacket": "leather", "trousers": "denim", "pants": map[string]any{"size": "large"}}, v.Get("clothing"))
	assert.Equal(t, 35, v.Get("age"))
}

func TestUnmarshalExact(t *testing.T) {
	v := New()
	target := &testUnmarshalExtra{}
	v.SetConfigType("yaml")
	r := bytes.NewReader(yamlExampleWithExtras)
	v.ReadConfig(r)
	err := v.UnmarshalExact(target)
	assert.Error(t, err, "UnmarshalExact should error when populating a struct from a conf that contains unused fields")
}

func TestOverrides(t *testing.T) {
	v := New()
	v.Set("age", 40)
	assert.Equal(t, 40, v.Get("age"))
}

func TestDefaultPost(t *testing.T) {
	v := New()
	assert.NotEqual(t, "NYC", v.Get("state"))
	v.SetDefault("state", "NYC")
	assert.Equal(t, "NYC", v.Get("state"))
}

func TestAliases(t *testing.T) {
	v := New()
	v.Set("age", 40)
	v.RegisterAlias("years", "age")
	assert.Equal(t, 40, v.Get("years"))
	v.Set("years", 45)
	assert.Equal(t, 45, v.Get("age"))
}

func TestAliasInConfigFile(t *testing.T) {
	v := New()

	v.SetConfigType("yaml")

	// Read the YAML data into Viper configuration
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(yamlExample)), "Error reading YAML data")

	v.RegisterAlias("beard", "hasbeard")
	assert.Equal(t, true, v.Get("hasbeard"))

	v.Set("hasbeard", false)
	assert.Equal(t, false, v.Get("beard"))
}

func TestYML(t *testing.T) {
	v := New()
	v.SetConfigType("yaml")

	// Read the YAML data into Viper configuration
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(yamlExample)), "Error reading YAML data")

	assert.Equal(t, "steve", v.Get("name"))
}

func TestJSON(t *testing.T) {
	v := New()

	v.SetConfigType("json")
	// Read the JSON data into Viper configuration
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(jsonExample)), "Error reading JSON data")

	assert.Equal(t, "0001", v.Get("id"))
}

func TestProperties(t *testing.T) {
	v := New()

	v.SetConfigType("properties")

	// Read the properties data into Viper configuration
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(propertiesExample)), "Error reading properties data")

	assert.Equal(t, "0001", v.Get("p_id"))
}

func TestTOML(t *testing.T) {
	v := New()
	v.SetConfigType("toml")

	// Read the properties data into Viper configuration
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(tomlExample)), "Error reading toml data")

	assert.Equal(t, "TOML Example", v.Get("title"))
}

func TestDotEnv(t *testing.T) {
	v := New()
	v.SetConfigType("env")
	// Read the properties data into Viper configuration
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(dotenvExample)), "Error reading env data")

	assert.Equal(t, "DotEnv Example", v.Get("title_dotenv"))
}

func TestHCL(t *testing.T) {
	v := New()
	v.SetConfigType("hcl")
	// Read the properties data into Viper configuration
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(hclExample)), "Error reading hcl data")

	// initHcl()
	assert.Equal(t, "0001", v.Get("id"))
	assert.Equal(t, 0.55, v.Get("ppu"))
	assert.Equal(t, "donut", v.Get("type"))
	assert.Equal(t, "Cake", v.Get("name"))
	v.Set("id", "0002")
	assert.Equal(t, "0002", v.Get("id"))
	assert.NotEqual(t, "cronut", v.Get("type"))
}

func TestIni(t *testing.T) {
	// initIni()
	v := New()
	v.SetConfigType("ini")
	// Read the properties data into Viper configuration
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(iniExample)), "Error reading ini data")

	assert.Equal(t, "ini", v.Get("default.name"))
}

func TestRemotePrecedence(t *testing.T) {
	v := New()
	v.SetConfigType("json")
	// Read the properties data into Viper configuration v.config
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(jsonExample)), "Error reading json data")

	assert.Equal(t, "0001", v.Get("id"))

	// update the kvstore with the remoteExample which should overite the key in v.config
	remote := bytes.NewReader(remoteExample)
	require.NoError(t, v.unmarshalReader(remote, v.kvstore), "Error reading json data in to kvstore")

	assert.Equal(t, "0001", v.Get("id"))
	assert.NotEqual(t, "cronut", v.Get("type"))
	assert.Equal(t, "remote", v.Get("newkey"))
	v.Set("newkey", "newvalue")
	assert.NotEqual(t, "remote", v.Get("newkey"))
	assert.Equal(t, "newvalue", v.Get("newkey"))
}

func TestEnv(t *testing.T) {
	v := New()
	v.SetConfigType("json")
	// Read the properties data into Viper configuration v.config
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(jsonExample)), "Error reading json data")

	v.BindEnv("id")
	v.BindEnv("f", "FOOD", "OLD_FOOD")

	t.Setenv("ID", "13")
	t.Setenv("FOOD", "apple")
	t.Setenv("OLD_FOOD", "banana")
	t.Setenv("NAME", "crunk")

	assert.Equal(t, "13", v.Get("id"))
	assert.Equal(t, "apple", v.Get("f"))
	assert.Equal(t, "Cake", v.Get("name"))

	v.AutomaticEnv()

	assert.Equal(t, "crunk", v.Get("name"))
}

func TestMultipleEnv(t *testing.T) {
	v := New()
	v.SetConfigType("json")
	// Read the properties data into Viper configuration v.config
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(jsonExample)), "Error reading json data")

	v.BindEnv("f", "FOOD", "OLD_FOOD")

	t.Setenv("OLD_FOOD", "banana")

	assert.Equal(t, "banana", v.Get("f"))
}

func TestEmptyEnv(t *testing.T) {
	v := New()
	v.SetConfigType("json")
	// Read the properties data into Viper configuration v.config
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(jsonExample)), "Error reading json data")

	v.BindEnv("type") // Empty environment variable
	v.BindEnv("name") // Bound, but not set environment variable

	t.Setenv("TYPE", "")

	assert.Equal(t, "donut", v.Get("type"))
	assert.Equal(t, "Cake", v.Get("name"))
}

func TestEmptyEnv_Allowed(t *testing.T) {
	v := New()
	v.SetConfigType("json")
	// Read the properties data into Viper configuration v.config
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(jsonExample)), "Error reading json data")

	v.AllowEmptyEnv(true)

	v.BindEnv("type") // Empty environment variable
	v.BindEnv("name") // Bound, but not set environment variable

	t.Setenv("TYPE", "")

	assert.Equal(t, "", v.Get("type"))
	assert.Equal(t, "Cake", v.Get("name"))
}

func TestEnvPrefix(t *testing.T) {
	v := New()
	v.SetConfigType("json")
	// Read the properties data into Viper configuration v.config
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(jsonExample)), "Error reading json data")

	v.SetEnvPrefix("foo") // will be uppercased automatically
	v.BindEnv("id")
	v.BindEnv("f", "FOOD") // not using prefix

	t.Setenv("FOO_ID", "13")
	t.Setenv("FOOD", "apple")
	t.Setenv("FOO_NAME", "crunk")

	assert.Equal(t, "13", v.Get("id"))
	assert.Equal(t, "apple", v.Get("f"))
	assert.Equal(t, "Cake", v.Get("name"))

	v.AutomaticEnv()

	assert.Equal(t, "crunk", v.Get("name"))
}

func TestAutoEnv(t *testing.T) {
	v := New()

	v.AutomaticEnv()

	t.Setenv("FOO_BAR", "13")

	assert.Equal(t, "13", v.Get("foo_bar"))
}

func TestAutoEnvWithPrefix(t *testing.T) {
	v := New()
	v.AutomaticEnv()
	v.SetEnvPrefix("Baz")
	t.Setenv("BAZ_BAR", "13")
	assert.Equal(t, "13", v.Get("bar"))
}

func TestSetEnvKeyReplacer(t *testing.T) {
	v := New()
	v.AutomaticEnv()

	t.Setenv("REFRESH_INTERVAL", "30s")

	replacer := strings.NewReplacer("-", "_")
	v.SetEnvKeyReplacer(replacer)

	assert.Equal(t, "30s", v.Get("refresh-interval"))
}

func TestEnvKeyReplacer(t *testing.T) {
	v := NewWithOptions(EnvKeyReplacer(strings.NewReplacer("-", "_")))
	v.AutomaticEnv()
	t.Setenv("REFRESH_INTERVAL", "30s")
	assert.Equal(t, "30s", v.Get("refresh-interval"))
}

func TestEnvSubConfig(t *testing.T) {
	v := New()
	v.SetConfigType("yaml")
	// Read the properties data into Viper configuration v.config
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(yamlExample)), "Error reading json data")
	v.AutomaticEnv()
	v.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))

	t.Setenv("CLOTHING_PANTS_SIZE", "small")
	subv := v.Sub("clothing").Sub("pants")
	assert.Equal(t, "small", subv.Get("size"))

	// again with EnvPrefix
	v.SetEnvPrefix("foo") // will be uppercased automatically
	subWithPrefix := v.Sub("clothing").Sub("pants")
	t.Setenv("FOO_CLOTHING_PANTS_SIZE", "large")
	assert.Equal(t, "large", subWithPrefix.Get("size"))
}

func TestAllKeys(t *testing.T) {
	v := New()
	initConfigs(v)

	ks := []string{
		"title",
		"author.bio",
		"author.e-mail",
		"author.github",
		"author.name",
		"newkey",
		"owner.organization",
		"owner.dob",
		"owner.bio",
		"name",
		"beard",
		"ppu",
		"batters.batter",
		"hobbies",
		"clothing.jacket",
		"clothing.trousers",
		"default.import_path",
		"default.name",
		"default.version",
		"clothing.pants.size",
		"age",
		"hacker",
		"id",
		"type",
		"eyes",
		"p_id",
		"p_ppu",
		"p_batters.batter.type",
		"p_type",
		"p_name",
		"foos",
		"title_dotenv",
		"type_dotenv",
		"name_dotenv",
	}
	dob, _ := time.Parse(time.RFC3339, "1979-05-27T07:32:00Z")
	all := map[string]any{
		"owner": map[string]any{
			"organization": "MongoDB",
			"bio":          "MongoDB Chief Developer Advocate & Hacker at Large",
			"dob":          dob,
		},
		"title": "TOML Example",
		"author": map[string]any{
			"e-mail": "fake@localhost",
			"github": "https://github.com/Unknown",
			"name":   "Unknown",
			"bio":    "Gopher.\nCoding addict.\nGood man.\n",
		},
		"ppu":  0.55,
		"eyes": "brown",
		"clothing": map[string]any{
			"trousers": "denim",
			"jacket":   "leather",
			"pants":    map[string]any{"size": "large"},
		},
		"default": map[string]any{
			"import_path": "gopkg.in/ini.v1",
			"name":        "ini",
			"version":     "v1",
		},
		"id": "0001",
		"batters": map[string]any{
			"batter": []any{
				map[string]any{"type": "Regular"},
				map[string]any{"type": "Chocolate"},
				map[string]any{"type": "Blueberry"},
				map[string]any{"type": "Devil's Food"},
			},
		},
		"hacker": true,
		"beard":  true,
		"hobbies": []any{
			"skateboarding",
			"snowboarding",
			"go",
		},
		"age":    35,
		"type":   "donut",
		"newkey": "remote",
		"name":   "Cake",
		"p_id":   "0001",
		"p_ppu":  "0.55",
		"p_name": "Cake",
		"p_batters": map[string]any{
			"batter": map[string]any{"type": "Regular"},
		},
		"p_type": "donut",
		"foos": []map[string]any{
			{
				"foo": []map[string]any{
					{"key": 1},
					{"key": 2},
					{"key": 3},
					{"key": 4},
				},
			},
		},
		"title_dotenv": "DotEnv Example",
		"type_dotenv":  "donut",
		"name_dotenv":  "Cake",
	}

	assert.ElementsMatch(t, ks, v.AllKeys())
	assert.Equal(t, all, v.AllSettings())
}

func TestAllKeysWithEnv(t *testing.T) {
	v := New()

	// bind and define environment variables (including a nested one)
	v.BindEnv("id")
	v.BindEnv("foo.bar")
	v.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))

	t.Setenv("ID", "13")
	t.Setenv("FOO_BAR", "baz")

	assert.ElementsMatch(t, []string{"id", "foo.bar"}, v.AllKeys())
}

func TestAliasesOfAliases(t *testing.T) {
	v := New()
	v.Set("Title", "Checking Case")
	v.RegisterAlias("Foo", "Bar")
	v.RegisterAlias("Bar", "Title")
	assert.Equal(t, "Checking Case", v.Get("FOO"))
}

func TestRecursiveAliases(t *testing.T) {
	v := New()
	v.Set("baz", "bat")
	v.RegisterAlias("Baz", "Roo")
	v.RegisterAlias("Roo", "baz")
	assert.Equal(t, "bat", v.Get("Baz"))
}

func TestUnmarshal(t *testing.T) {
	v := New()
	v.SetDefault("port", 1313)
	v.Set("name", "Steve")
	v.Set("duration", "1s1ms")
	v.Set("modes", []int{1, 2, 3})

	type config struct {
		Port     int
		Name     string
		Duration time.Duration
		Modes    []int
	}

	var C config
	require.NoError(t, v.Unmarshal(&C), "unable to decode into struct")

	assert.Equal(
		t,
		&config{
			Name:     "Steve",
			Port:     1313,
			Duration: time.Second + time.Millisecond,
			Modes:    []int{1, 2, 3},
		},
		&C,
	)

	v.Set("port", 1234)
	require.NoError(t, v.Unmarshal(&C), "unable to decode into struct")

	assert.Equal(
		t,
		&config{
			Name:     "Steve",
			Port:     1234,
			Duration: time.Second + time.Millisecond,
			Modes:    []int{1, 2, 3},
		},
		&C,
	)
}

func TestUnmarshalWithDecoderOptions(t *testing.T) {
	v := New()
	v.Set("credentials", "{\"foo\":\"bar\"}")

	opt := DecodeHook(mapstructure.ComposeDecodeHookFunc(
		mapstructure.StringToTimeDurationHookFunc(),
		mapstructure.StringToSliceHookFunc(","),
		// Custom Decode Hook Function
		func(rf reflect.Kind, rt reflect.Kind, data any) (any, error) {
			if rf != reflect.String || rt != reflect.Map {
				return data, nil
			}
			m := map[string]string{}
			raw := data.(string)
			if raw == "" {
				return m, nil
			}
			err := json.Unmarshal([]byte(raw), &m)
			return m, err
		},
	))

	type config struct {
		Credentials map[string]string
	}

	var C config

	require.NoError(t, v.Unmarshal(&C, opt), "unable to decode into struct")

	assert.Equal(t, &config{
		Credentials: map[string]string{"foo": "bar"},
	}, &C)
}

func TestUnmarshalWithAutomaticEnv(t *testing.T) {
	if !features.BindStruct {
		t.Skip("binding struct is not enabled")
	}

	t.Setenv("PORT", "1313")
	t.Setenv("NAME", "Steve")
	t.Setenv("DURATION", "1s1ms")
	t.Setenv("MODES", "1,2,3")
	t.Setenv("SECRET", "42")
	t.Setenv("FILESYSTEM_SIZE", "4096")

	type AuthConfig struct {
		Secret string `mapstructure:"secret"`
	}

	type StorageConfig struct {
		Size int `mapstructure:"size"`
	}

	type Configuration struct {
		Port     int           `mapstructure:"port"`
		Name     string        `mapstructure:"name"`
		Duration time.Duration `mapstructure:"duration"`

		// Infer name from struct
		Modes []int

		// Squash nested struct (omit prefix)
		Authentication AuthConfig `mapstructure:",squash"`

		// Different key
		Storage StorageConfig `mapstructure:"filesystem"`

		// Omitted field
		Flag bool `mapstructure:"flag"`
	}

	v := New()
	v.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))
	v.AutomaticEnv()

	t.Run("OK", func(t *testing.T) {
		var config Configuration
		if err := v.Unmarshal(&config); err != nil {
			t.Fatalf("unable to decode into struct, %v", err)
		}

		assert.Equal(
			t,
			Configuration{
				Name:     "Steve",
				Port:     1313,
				Duration: time.Second + time.Millisecond,
				Modes:    []int{1, 2, 3},
				Authentication: AuthConfig{
					Secret: "42",
				},
				Storage: StorageConfig{
					Size: 4096,
				},
			},
			config,
		)
	})

	t.Run("Precedence", func(t *testing.T) {
		var config Configuration

		v.Set("port", 1234)
		if err := v.Unmarshal(&config); err != nil {
			t.Fatalf("unable to decode into struct, %v", err)
		}

		assert.Equal(
			t,
			Configuration{
				Name:     "Steve",
				Port:     1234,
				Duration: time.Second + time.Millisecond,
				Modes:    []int{1, 2, 3},
				Authentication: AuthConfig{
					Secret: "42",
				},
				Storage: StorageConfig{
					Size: 4096,
				},
			},
			config,
		)
	})

	t.Run("Unset", func(t *testing.T) {
		var config Configuration

		err := v.Unmarshal(&config, func(config *mapstructure.DecoderConfig) {
			config.ErrorUnset = true
		})

		assert.Error(t, err, "expected viper.Unmarshal to return error due to unset field 'FLAG'")
	})

	t.Run("Exact", func(t *testing.T) {
		var config Configuration

		v.Set("port", 1234)
		if err := v.UnmarshalExact(&config); err != nil {
			t.Fatalf("unable to decode into struct, %v", err)
		}

		assert.Equal(
			t,
			Configuration{
				Name:     "Steve",
				Port:     1234,
				Duration: time.Second + time.Millisecond,
				Modes:    []int{1, 2, 3},
				Authentication: AuthConfig{
					Secret: "42",
				},
				Storage: StorageConfig{
					Size: 4096,
				},
			},
			config,
		)
	})
}

func TestBindPFlags(t *testing.T) {
	v := New() // create independent Viper object
	flagSet := pflag.NewFlagSet("test", pflag.ContinueOnError)

	testValues := map[string]*string{
		"host":     nil,
		"port":     nil,
		"endpoint": nil,
	}

	mutatedTestValues := map[string]string{
		"host":     "localhost",
		"port":     "6060",
		"endpoint": "/public",
	}

	for name := range testValues {
		testValues[name] = flagSet.String(name, "", "test")
	}

	err := v.BindPFlags(flagSet)
	require.NoError(t, err, "error binding flag set")

	flagSet.VisitAll(func(flag *pflag.Flag) {
		flag.Value.Set(mutatedTestValues[flag.Name])
		flag.Changed = true
	})

	for name, expected := range mutatedTestValues {
		assert.Equal(t, expected, v.Get(name))
	}
}

func TestBindPFlagsStringSlice(t *testing.T) {
	tests := []struct {
		Expected []string
		Value    string
	}{
		{[]string{}, ""},
		{[]string{"jeden"}, "jeden"},
		{[]string{"dwa", "trzy"}, "dwa,trzy"},
		{[]string{"cztery", "piec , szesc"}, "cztery,\"piec , szesc\""},
	}

	v := New() // create independent Viper object
	defaultVal := []string{"default"}
	v.SetDefault("stringslice", defaultVal)

	for _, testValue := range tests {
		flagSet := pflag.NewFlagSet("test", pflag.ContinueOnError)
		flagSet.StringSlice("stringslice", testValue.Expected, "test")

		for _, changed := range []bool{true, false} {
			flagSet.VisitAll(func(f *pflag.Flag) {
				f.Value.Set(testValue.Value)
				f.Changed = changed
			})

			err := v.BindPFlags(flagSet)
			require.NoError(t, err, "error binding flag set")

			type TestStr struct {
				StringSlice []string
			}
			val := &TestStr{}
			err = v.Unmarshal(val)
			require.NoError(t, err, "cannot unmarshal")
			if changed {
				assert.Equal(t, testValue.Expected, val.StringSlice)
				assert.Equal(t, testValue.Expected, v.Get("stringslice"))
			} else {
				assert.Equal(t, defaultVal, val.StringSlice)
			}
		}
	}
}

func TestBindPFlagsStringArray(t *testing.T) {
	tests := []struct {
		Expected []string
		Value    string
	}{
		{[]string{}, ""},
		{[]string{"jeden"}, "jeden"},
		{[]string{"dwa,trzy"}, "dwa,trzy"},
		{[]string{"cztery,\"piec , szesc\""}, "cztery,\"piec , szesc\""},
	}

	v := New() // create independent Viper object
	defaultVal := []string{"default"}
	v.SetDefault("stringarray", defaultVal)

	for _, testValue := range tests {
		flagSet := pflag.NewFlagSet("test", pflag.ContinueOnError)
		flagSet.StringArray("stringarray", testValue.Expected, "test")

		for _, changed := range []bool{true, false} {
			flagSet.VisitAll(func(f *pflag.Flag) {
				f.Value.Set(testValue.Value)
				f.Changed = changed
			})

			err := v.BindPFlags(flagSet)
			require.NoError(t, err, "error binding flag set")

			type TestStr struct {
				StringArray []string
			}
			val := &TestStr{}
			err = v.Unmarshal(val)
			require.NoError(t, err, "cannot unmarshal")
			if changed {
				assert.Equal(t, testValue.Expected, val.StringArray)
				assert.Equal(t, testValue.Expected, v.Get("stringarray"))
			} else {
				assert.Equal(t, defaultVal, val.StringArray)
			}
		}
	}
}

func TestSliceFlagsReturnCorrectType(t *testing.T) {
	flagSet := pflag.NewFlagSet("test", pflag.ContinueOnError)
	flagSet.IntSlice("int", []int{1, 2}, "")
	flagSet.StringSlice("str", []string{"3", "4"}, "")
	flagSet.DurationSlice("duration", []time.Duration{5 * time.Second}, "")

	v := New()
	v.BindPFlags(flagSet)

	all := v.AllSettings()

	assert.IsType(t, []int{}, all["int"])
	assert.IsType(t, []string{}, all["str"])
	assert.IsType(t, []time.Duration{}, all["duration"])
}

func TestBindPFlagsIntSlice(t *testing.T) {
	tests := []struct {
		Expected []int
		Value    string
	}{
		{[]int{}, ""},
		{[]int{1}, "1"},
		{[]int{2, 3}, "2,3"},
	}

	v := New() // create independent Viper object
	defaultVal := []int{0}
	v.SetDefault("intslice", defaultVal)

	for _, testValue := range tests {
		flagSet := pflag.NewFlagSet("test", pflag.ContinueOnError)
		flagSet.IntSlice("intslice", testValue.Expected, "test")

		for _, changed := range []bool{true, false} {
			flagSet.VisitAll(func(f *pflag.Flag) {
				f.Value.Set(testValue.Value)
				f.Changed = changed
			})

			err := v.BindPFlags(flagSet)
			require.NoError(t, err, "error binding flag set")

			type TestInt struct {
				IntSlice []int
			}
			val := &TestInt{}
			err = v.Unmarshal(val)
			require.NoError(t, err, "cannot unmarshal")
			if changed {
				assert.Equal(t, testValue.Expected, val.IntSlice)
				assert.Equal(t, testValue.Expected, v.Get("intslice"))
			} else {
				assert.Equal(t, defaultVal, val.IntSlice)
			}
		}
	}
}

func TestBindPFlag(t *testing.T) {
	v := New()
	testString := "testing"
	testValue := newStringValue(testString, &testString)

	flag := &pflag.Flag{
		Name:    "testflag",
		Value:   testValue,
		Changed: false,
	}

	v.BindPFlag("testvalue", flag)

	assert.Equal(t, testString, v.Get("testvalue"))

	flag.Value.Set("testing_mutate")
	flag.Changed = true // hack for pflag usage

	assert.Equal(t, "testing_mutate", v.Get("testvalue"))
}

func TestBindPFlagDetectNilFlag(t *testing.T) {
	v := New()
	result := v.BindPFlag("testvalue", nil)
	assert.Error(t, result)
}

func TestBindPFlagStringToString(t *testing.T) {
	tests := []struct {
		Expected map[string]string
		Value    string
	}{
		{map[string]string{}, ""},
		{map[string]string{"yo": "hi"}, "yo=hi"},
		{map[string]string{"yo": "hi", "oh": "hi=there"}, "yo=hi,oh=hi=there"},
		{map[string]string{"yo": ""}, "yo="},
		{map[string]string{"yo": "", "oh": "hi=there"}, "yo=,oh=hi=there"},
	}

	v := New() // create independent Viper object
	defaultVal := map[string]string{}
	v.SetDefault("stringtostring", defaultVal)

	for _, testValue := range tests {
		flagSet := pflag.NewFlagSet("test", pflag.ContinueOnError)
		flagSet.StringToString("stringtostring", testValue.Expected, "test")

		for _, changed := range []bool{true, false} {
			flagSet.VisitAll(func(f *pflag.Flag) {
				f.Value.Set(testValue.Value)
				f.Changed = changed
			})

			err := v.BindPFlags(flagSet)
			require.NoError(t, err, "error binding flag set")

			type TestMap struct {
				StringToString map[string]string
			}
			val := &TestMap{}
			err = v.Unmarshal(val)
			require.NoError(t, err, "cannot unmarshal")
			if changed {
				assert.Equal(t, testValue.Expected, val.StringToString)
			} else {
				assert.Equal(t, defaultVal, val.StringToString)
			}
		}
	}
}

func TestBindPFlagStringToInt(t *testing.T) {
	tests := []struct {
		Expected map[string]int
		Value    string
	}{
		{map[string]int{"yo": 1, "oh": 21}, "yo=1,oh=21"},
		{map[string]int{"yo": 100000000, "oh": 0}, "yo=100000000,oh=0"},
		{map[string]int{}, "yo=2,oh=21.0"},
		{map[string]int{}, "yo=,oh=20.99"},
		{map[string]int{}, "yo=,oh="},
	}

	v := New() // create independent Viper object
	defaultVal := map[string]int{}
	v.SetDefault("stringtoint", defaultVal)

	for _, testValue := range tests {
		flagSet := pflag.NewFlagSet("test", pflag.ContinueOnError)
		flagSet.StringToInt("stringtoint", testValue.Expected, "test")

		for _, changed := range []bool{true, false} {
			flagSet.VisitAll(func(f *pflag.Flag) {
				f.Value.Set(testValue.Value)
				f.Changed = changed
			})

			err := v.BindPFlags(flagSet)
			require.NoError(t, err, "error binding flag set")

			type TestMap struct {
				StringToInt map[string]int
			}
			val := &TestMap{}
			err = v.Unmarshal(val)
			require.NoError(t, err, "cannot unmarshal")
			if changed {
				assert.Equal(t, testValue.Expected, val.StringToInt)
			} else {
				assert.Equal(t, defaultVal, val.StringToInt)
			}
		}
	}
}

func TestBoundCaseSensitivity(t *testing.T) {
	v := New()
	initConfigs(v)
	assert.Equal(t, "brown", v.Get("eyes"))

	v.BindEnv("eYEs", "TURTLE_EYES")

	t.Setenv("TURTLE_EYES", "blue")

	assert.Equal(t, "blue", v.Get("eyes"))

	testString := "green"
	testValue := newStringValue(testString, &testString)

	flag := &pflag.Flag{
		Name:    "eyeballs",
		Value:   testValue,
		Changed: true,
	}

	v.BindPFlag("eYEs", flag)
	assert.Equal(t, "green", v.Get("eyes"))
}

func TestSizeInBytes(t *testing.T) {
	input := map[string]uint{
		"":               0,
		"b":              0,
		"12 bytes":       0,
		"200000000000gb": 0,
		"12 b":           12,
		"43 MB":          43 * (1 << 20),
		"10mb":           10 * (1 << 20),
		"1gb":            1 << 30,
	}

	for str, expected := range input {
		assert.Equal(t, expected, parseSizeInBytes(str), str)
	}
}

func TestFindsNestedKeys(t *testing.T) {
	v := New()
	initConfigs(v)
	dob, _ := time.Parse(time.RFC3339, "1979-05-27T07:32:00Z")

	v.Set("super", map[string]any{
		"deep": map[string]any{
			"nested": "value",
		},
	})

	expected := map[string]any{
		"super": map[string]any{
			"deep": map[string]any{
				"nested": "value",
			},
		},
		"super.deep": map[string]any{
			"nested": "value",
		},
		"super.deep.nested":  "value",
		"owner.organization": "MongoDB",
		"batters.batter": []any{
			map[string]any{
				"type": "Regular",
			},
			map[string]any{
				"type": "Chocolate",
			},
			map[string]any{
				"type": "Blueberry",
			},
			map[string]any{
				"type": "Devil's Food",
			},
		},
		"hobbies": []any{
			"skateboarding", "snowboarding", "go",
		},
		"TITLE_DOTENV": "DotEnv Example",
		"TYPE_DOTENV":  "donut",
		"NAME_DOTENV":  "Cake",
		"title":        "TOML Example",
		"newkey":       "remote",
		"batters": map[string]any{
			"batter": []any{
				map[string]any{
					"type": "Regular",
				},
				map[string]any{
					"type": "Chocolate",
				},
				map[string]any{
					"type": "Blueberry",
				},
				map[string]any{
					"type": "Devil's Food",
				},
			},
		},
		"eyes": "brown",
		"age":  35,
		"owner": map[string]any{
			"organization": "MongoDB",
			"bio":          "MongoDB Chief Developer Advocate & Hacker at Large",
			"dob":          dob,
		},
		"owner.bio": "MongoDB Chief Developer Advocate & Hacker at Large",
		"type":      "donut",
		"id":        "0001",
		"name":      "Cake",
		"hacker":    true,
		"ppu":       0.55,
		"clothing": map[string]any{
			"jacket":   "leather",
			"trousers": "denim",
			"pants": map[string]any{
				"size": "large",
			},
		},
		"clothing.jacket":     "leather",
		"clothing.pants.size": "large",
		"clothing.trousers":   "denim",
		"owner.dob":           dob,
		"beard":               true,
		"foos": []map[string]any{
			{
				"foo": []map[string]any{
					{
						"key": 1,
					},
					{
						"key": 2,
					},
					{
						"key": 3,
					},
					{
						"key": 4,
					},
				},
			},
		},
	}

	for key, expectedValue := range expected {
		assert.Equal(t, expectedValue, v.Get(key))
	}
}

func TestReadBufConfig(t *testing.T) {
	v := New()
	v.SetConfigType("yaml")
	v.ReadConfig(bytes.NewBuffer(yamlExample))
	t.Log(v.AllKeys())

	assert.True(t, v.InConfig("name"))
	assert.True(t, v.InConfig("clothing.jacket"))
	assert.False(t, v.InConfig("state"))
	assert.False(t, v.InConfig("clothing.hat"))
	assert.Equal(t, "steve", v.Get("name"))
	assert.Equal(t, []any{"skateboarding", "snowboarding", "go"}, v.Get("hobbies"))
	assert.Equal(t, map[string]any{"jacket": "leather", "trousers": "denim", "pants": map[string]any{"size": "large"}}, v.Get("clothing"))
	assert.Equal(t, 35, v.Get("age"))
}

func TestIsSet(t *testing.T) {
	v := New()
	v.SetConfigType("yaml")

	/* config and defaults */
	v.ReadConfig(bytes.NewBuffer(yamlExample))
	v.SetDefault("clothing.shoes", "sneakers")

	assert.True(t, v.IsSet("clothing"))
	assert.True(t, v.IsSet("clothing.jacket"))
	assert.False(t, v.IsSet("clothing.jackets"))
	assert.True(t, v.IsSet("clothing.shoes"))

	/* state change */
	assert.False(t, v.IsSet("helloworld"))
	v.Set("helloworld", "fubar")
	assert.True(t, v.IsSet("helloworld"))

	/* env */
	v.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))
	v.BindEnv("eyes")
	v.BindEnv("foo")
	v.BindEnv("clothing.hat")
	v.BindEnv("clothing.hats")

	t.Setenv("FOO", "bar")
	t.Setenv("CLOTHING_HAT", "bowler")

	assert.True(t, v.IsSet("eyes"))           // in the config file
	assert.True(t, v.IsSet("foo"))            // in the environment
	assert.True(t, v.IsSet("clothing.hat"))   // in the environment
	assert.False(t, v.IsSet("clothing.hats")) // not defined

	/* flags */
	flagset := pflag.NewFlagSet("testisset", pflag.ContinueOnError)
	flagset.Bool("foobaz", false, "foobaz")
	flagset.Bool("barbaz", false, "barbaz")
	foobaz, barbaz := flagset.Lookup("foobaz"), flagset.Lookup("barbaz")
	v.BindPFlag("foobaz", foobaz)
	v.BindPFlag("barbaz", barbaz)
	barbaz.Value.Set("true")
	barbaz.Changed = true // hack for pflag usage

	assert.False(t, v.IsSet("foobaz"))
	assert.True(t, v.IsSet("barbaz"))
}

func TestDirsSearch(t *testing.T) {
	root, config := initDirs(t)

	v := New()
	v.SetConfigName(config)
	v.SetDefault(`key`, `default`)

	entries, err := os.ReadDir(root)
	require.NoError(t, err)
	for _, e := range entries {
		if e.IsDir() {
			v.AddConfigPath(filepath.Join(root, e.Name()))
		}
	}

	err = v.ReadInConfig()
	require.NoError(t, err)

	assert.Equal(t, `value is `+filepath.Base(v.configPaths[0]), v.GetString(`key`))
}

func TestWrongDirsSearchNotFound(t *testing.T) {
	_, config := initDirs(t)

	v := New()
	v.SetConfigName(config)
	v.SetDefault(`key`, `default`)

	v.AddConfigPath(`whattayoutalkingbout`)
	v.AddConfigPath(`thispathaintthere`)

	err := v.ReadInConfig()
	assert.IsType(t, err, ConfigFileNotFoundError{"", ""})

	// Even though config did not load and the error might have
	// been ignored by the client, the default still loads
	assert.Equal(t, `default`, v.GetString(`key`))
}

func TestWrongDirsSearchNotFoundForMerge(t *testing.T) {
	_, config := initDirs(t)

	v := New()
	v.SetConfigName(config)
	v.SetDefault(`key`, `default`)

	v.AddConfigPath(`whattayoutalkingbout`)
	v.AddConfigPath(`thispathaintthere`)

	err := v.MergeInConfig()
	assert.Equal(t, reflect.TypeOf(ConfigFileNotFoundError{"", ""}), reflect.TypeOf(err))

	// Even though config did not load and the error might have
	// been ignored by the client, the default still loads
	assert.Equal(t, `default`, v.GetString(`key`))
}

var yamlInvalid = []byte(`hash: map
- foo
- bar
`)

func TestUnwrapParseErrors(t *testing.T) {
	v := New()
	v.SetConfigType("yaml")
	assert.ErrorAs(t, v.ReadConfig(bytes.NewBuffer(yamlInvalid)), &ConfigParseError{})
}

func TestSub(t *testing.T) {
	v := New()
	v.SetConfigType("yaml")
	v.ReadConfig(bytes.NewBuffer(yamlExample))

	subv := v.Sub("clothing")
	assert.Equal(t, v.Get("clothing.pants.size"), subv.Get("pants.size"))

	subv = v.Sub("clothing.pants")
	assert.Equal(t, v.Get("clothing.pants.size"), subv.Get("size"))

	subv = v.Sub("clothing.pants.size")
	assert.Equal(t, (*Viper)(nil), subv)

	subv = v.Sub("missing.key")
	assert.Equal(t, (*Viper)(nil), subv)

	subv = v.Sub("clothing")
	assert.Equal(t, []string{"clothing"}, subv.parents)

	subv = v.Sub("clothing").Sub("pants")
	assert.Equal(t, []string{"clothing", "pants"}, subv.parents)
}

var hclWriteExpected = []byte(`"foos" = {
  "foo" = {
    "key" = 1
  }

  "foo" = {
    "key" = 2
  }

  "foo" = {
    "key" = 3
  }

  "foo" = {
    "key" = 4
  }
}

"id" = "0001"

"name" = "Cake"

"ppu" = 0.55

"type" = "donut"`)

var jsonWriteExpected = []byte(`{
  "batters": {
    "batter": [
      {
        "type": "Regular"
      },
      {
        "type": "Chocolate"
      },
      {
        "type": "Blueberry"
      },
      {
        "type": "Devil's Food"
      }
    ]
  },
  "id": "0001",
  "name": "Cake",
  "ppu": 0.55,
  "type": "donut"
}`)

var propertiesWriteExpected = []byte(`p_id = 0001
p_type = donut
p_name = Cake
p_ppu = 0.55
p_batters.batter.type = Regular
`)

// var yamlWriteExpected = []byte(`age: 35
// beard: true
// clothing:
//     jacket: leather
//     pants:
//         size: large
//     trousers: denim
// eyes: brown
// hacker: true
// hobbies:
//     - skateboarding
//     - snowboarding
//     - go
// name: steve
// `)

func TestWriteConfig(t *testing.T) {
	fs := afero.NewMemMapFs()
	testCases := map[string]struct {
		configName      string
		inConfigType    string
		outConfigType   string
		fileName        string
		input           []byte
		expectedContent []byte
	}{
		"hcl with file extension": {
			configName:      "c",
			inConfigType:    "hcl",
			outConfigType:   "hcl",
			fileName:        "c.hcl",
			input:           hclExample,
			expectedContent: hclWriteExpected,
		},
		"hcl without file extension": {
			configName:      "c",
			inConfigType:    "hcl",
			outConfigType:   "hcl",
			fileName:        "c",
			input:           hclExample,
			expectedContent: hclWriteExpected,
		},
		"hcl with file extension and mismatch type": {
			configName:      "c",
			inConfigType:    "hcl",
			outConfigType:   "json",
			fileName:        "c.hcl",
			input:           hclExample,
			expectedContent: hclWriteExpected,
		},
		"json with file extension": {
			configName:      "c",
			inConfigType:    "json",
			outConfigType:   "json",
			fileName:        "c.json",
			input:           jsonExample,
			expectedContent: jsonWriteExpected,
		},
		"json without file extension": {
			configName:      "c",
			inConfigType:    "json",
			outConfigType:   "json",
			fileName:        "c",
			input:           jsonExample,
			expectedContent: jsonWriteExpected,
		},
		"json with file extension and mismatch type": {
			configName:      "c",
			inConfigType:    "json",
			outConfigType:   "hcl",
			fileName:        "c.json",
			input:           jsonExample,
			expectedContent: jsonWriteExpected,
		},
		"properties with file extension": {
			configName:      "c",
			inConfigType:    "properties",
			outConfigType:   "properties",
			fileName:        "c.properties",
			input:           propertiesExample,
			expectedContent: propertiesWriteExpected,
		},
		"properties without file extension": {
			configName:      "c",
			inConfigType:    "properties",
			outConfigType:   "properties",
			fileName:        "c",
			input:           propertiesExample,
			expectedContent: propertiesWriteExpected,
		},
		"yaml with file extension": {
			configName:      "c",
			inConfigType:    "yaml",
			outConfigType:   "yaml",
			fileName:        "c.yaml",
			input:           yamlExample,
			expectedContent: yamlWriteExpected,
		},
		"yaml without file extension": {
			configName:      "c",
			inConfigType:    "yaml",
			outConfigType:   "yaml",
			fileName:        "c",
			input:           yamlExample,
			expectedContent: yamlWriteExpected,
		},
		"yaml with file extension and mismatch type": {
			configName:      "c",
			inConfigType:    "yaml",
			outConfigType:   "json",
			fileName:        "c.yaml",
			input:           yamlExample,
			expectedContent: yamlWriteExpected,
		},
	}
	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			v := New()
			v.SetFs(fs)
			v.SetConfigName(tc.fileName)
			v.SetConfigType(tc.inConfigType)

			err := v.ReadConfig(bytes.NewBuffer(tc.input))
			require.NoError(t, err)
			v.SetConfigType(tc.outConfigType)
			err = v.WriteConfigAs(tc.fileName)
			require.NoError(t, err)
			read, err := afero.ReadFile(fs, tc.fileName)
			require.NoError(t, err)
			assert.Equal(t, tc.expectedContent, read)
		})
	}
}

func TestWriteConfigTOML(t *testing.T) {
	fs := afero.NewMemMapFs()

	testCases := map[string]struct {
		configName string
		configType string
		fileName   string
		input      []byte
	}{
		"with file extension": {
			configName: "c",
			configType: "toml",
			fileName:   "c.toml",
			input:      tomlExample,
		},
		"without file extension": {
			configName: "c",
			configType: "toml",
			fileName:   "c",
			input:      tomlExample,
		},
	}
	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			v := New()
			v.SetFs(fs)
			v.SetConfigName(tc.configName)
			v.SetConfigType(tc.configType)
			err := v.ReadConfig(bytes.NewBuffer(tc.input))
			require.NoError(t, err)
			err = v.WriteConfigAs(tc.fileName)
			require.NoError(t, err)

			// The TOML String method does not order the contents.
			// Therefore, we must read the generated file and compare the data.
			v2 := New()
			v2.SetFs(fs)
			v2.SetConfigName(tc.configName)
			v2.SetConfigType(tc.configType)
			v2.SetConfigFile(tc.fileName)
			err = v2.ReadInConfig()
			require.NoError(t, err)

			assert.Equal(t, v.GetString("title"), v2.GetString("title"))
			assert.Equal(t, v.GetString("owner.bio"), v2.GetString("owner.bio"))
			assert.Equal(t, v.GetString("owner.dob"), v2.GetString("owner.dob"))
			assert.Equal(t, v.GetString("owner.organization"), v2.GetString("owner.organization"))
		})
	}
}

func TestWriteConfigDotEnv(t *testing.T) {
	fs := afero.NewMemMapFs()
	testCases := map[string]struct {
		configName string
		configType string
		fileName   string
		input      []byte
	}{
		"with file extension": {
			configName: "c",
			configType: "env",
			fileName:   "c.env",
			input:      dotenvExample,
		},
		"without file extension": {
			configName: "c",
			configType: "env",
			fileName:   "c",
			input:      dotenvExample,
		},
	}
	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			v := New()
			v.SetFs(fs)
			v.SetConfigName(tc.configName)
			v.SetConfigType(tc.configType)
			err := v.ReadConfig(bytes.NewBuffer(tc.input))
			require.NoError(t, err)
			err = v.WriteConfigAs(tc.fileName)
			require.NoError(t, err)

			// The TOML String method does not order the contents.
			// Therefore, we must read the generated file and compare the data.
			v2 := New()
			v2.SetFs(fs)
			v2.SetConfigName(tc.configName)
			v2.SetConfigType(tc.configType)
			v2.SetConfigFile(tc.fileName)
			err = v2.ReadInConfig()
			require.NoError(t, err)

			assert.Equal(t, v.GetString("title_dotenv"), v2.GetString("title_dotenv"))
			assert.Equal(t, v.GetString("type_dotenv"), v2.GetString("type_dotenv"))
			assert.Equal(t, v.GetString("kind_dotenv"), v2.GetString("kind_dotenv"))
		})
	}
}

func TestSafeWriteConfig(t *testing.T) {
	v := New()
	fs := afero.NewMemMapFs()
	v.SetFs(fs)
	v.AddConfigPath("/test")
	v.SetConfigName("c")
	v.SetConfigType("yaml")
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(yamlExample)))
	require.NoError(t, v.SafeWriteConfig())
	read, err := afero.ReadFile(fs, testutil.AbsFilePath(t, "/test/c.yaml"))
	require.NoError(t, err)
	assert.Equal(t, yamlWriteExpected, read)
}

func TestSafeWriteConfigWithMissingConfigPath(t *testing.T) {
	v := New()
	fs := afero.NewMemMapFs()
	v.SetFs(fs)
	v.SetConfigName("c")
	v.SetConfigType("yaml")
	require.EqualError(t, v.SafeWriteConfig(), "missing configuration for 'configPath'")
}

func TestSafeWriteConfigWithExistingFile(t *testing.T) {
	v := New()
	fs := afero.NewMemMapFs()
	fs.Create(testutil.AbsFilePath(t, "/test/c.yaml"))
	v.SetFs(fs)
	v.AddConfigPath("/test")
	v.SetConfigName("c")
	v.SetConfigType("yaml")
	err := v.SafeWriteConfig()
	require.Error(t, err)
	_, ok := err.(ConfigFileAlreadyExistsError)
	assert.True(t, ok, "Expected ConfigFileAlreadyExistsError")
}

func TestSafeWriteAsConfig(t *testing.T) {
	v := New()
	fs := afero.NewMemMapFs()
	v.SetFs(fs)
	err := v.ReadConfig(bytes.NewBuffer(yamlExample))
	require.NoError(t, err)
	require.NoError(t, v.SafeWriteConfigAs("/test/c.yaml"))
	_, err = afero.ReadFile(fs, "/test/c.yaml")
	require.NoError(t, err)
}

func TestSafeWriteConfigAsWithExistingFile(t *testing.T) {
	v := New()
	fs := afero.NewMemMapFs()
	fs.Create("/test/c.yaml")
	v.SetFs(fs)
	err := v.SafeWriteConfigAs("/test/c.yaml")
	require.Error(t, err)
	_, ok := err.(ConfigFileAlreadyExistsError)
	assert.True(t, ok, "Expected ConfigFileAlreadyExistsError")
}

func TestWriteHiddenFile(t *testing.T) {
	v := New()
	fs := afero.NewMemMapFs()
	fs.Create(testutil.AbsFilePath(t, "/test/.config"))
	v.SetFs(fs)

	v.SetConfigName(".config")
	v.SetConfigType("yaml")
	v.AddConfigPath("/test")

	err := v.ReadInConfig()
	require.NoError(t, err)

	err = v.WriteConfig()
	require.NoError(t, err)
}

var yamlMergeExampleTgt = []byte(`
hello:
    pop: 37890
    largenum: 765432101234567
    num2pow63: 9223372036854775808
    universe: null
    world:
    - us
    - uk
    - fr
    - de
`)

var yamlMergeExampleSrc = []byte(`
hello:
    pop: 45000
    largenum: 7654321001234567
    universe:
    - mw
    - ad
    ints:
    - 1
    - 2
fu: bar
`)

var jsonMergeExampleTgt = []byte(`
{
	"hello": {
		"foo": null,
		"pop": 123456
	}
}
`)

var jsonMergeExampleSrc = []byte(`
{
	"hello": {
		"foo": "foo str",
		"pop": "pop str"
	}
}
`)

func TestMergeConfig(t *testing.T) {
	v := New()
	v.SetConfigType("yml")
	err := v.ReadConfig(bytes.NewBuffer(yamlMergeExampleTgt))
	require.NoError(t, err)

	assert.Equal(t, 37890, v.GetInt("hello.pop"))
	assert.Equal(t, int32(37890), v.GetInt32("hello.pop"))
	assert.Equal(t, int64(765432101234567), v.GetInt64("hello.largenum"))
	assert.Equal(t, uint(37890), v.GetUint("hello.pop"))
	assert.Equal(t, uint16(37890), v.GetUint16("hello.pop"))
	assert.Equal(t, uint32(37890), v.GetUint32("hello.pop"))
	assert.Equal(t, uint64(9223372036854775808), v.GetUint64("hello.num2pow63"))
	assert.Len(t, v.GetStringSlice("hello.world"), 4)
	assert.Empty(t, v.GetString("fu"))

	err = v.MergeConfig(bytes.NewBuffer(yamlMergeExampleSrc))
	require.NoError(t, err)

	assert.Equal(t, 45000, v.GetInt("hello.pop"))
	assert.Equal(t, int32(45000), v.GetInt32("hello.pop"))
	assert.Equal(t, int64(7654321001234567), v.GetInt64("hello.largenum"))
	assert.Len(t, v.GetStringSlice("hello.world"), 4)
	assert.Len(t, v.GetStringSlice("hello.universe"), 2)
	assert.Len(t, v.GetIntSlice("hello.ints"), 2)
	assert.Equal(t, "bar", v.GetString("fu"))
}

func TestMergeConfigOverrideType(t *testing.T) {
	v := New()
	v.SetConfigType("json")
	err := v.ReadConfig(bytes.NewBuffer(jsonMergeExampleTgt))
	require.NoError(t, err)

	err = v.MergeConfig(bytes.NewBuffer(jsonMergeExampleSrc))
	require.NoError(t, err)

	assert.Equal(t, "pop str", v.GetString("hello.pop"))
	assert.Equal(t, "foo str", v.GetString("hello.foo"))
}

func TestMergeConfigNoMerge(t *testing.T) {
	v := New()
	v.SetConfigType("yml")
	err := v.ReadConfig(bytes.NewBuffer(yamlMergeExampleTgt))
	require.NoError(t, err)

	assert.Equal(t, 37890, v.GetInt("hello.pop"))
	assert.Len(t, v.GetStringSlice("hello.world"), 4)
	assert.Empty(t, v.GetString("fu"))

	err = v.ReadConfig(bytes.NewBuffer(yamlMergeExampleSrc))
	require.NoError(t, err)

	assert.Equal(t, 45000, v.GetInt("hello.pop"))
	assert.Empty(t, v.GetStringSlice("hello.world"))
	assert.Len(t, v.GetStringSlice("hello.universe"), 2)
	assert.Len(t, v.GetIntSlice("hello.ints"), 2)
	assert.Equal(t, "bar", v.GetString("fu"))
}

func TestMergeConfigMap(t *testing.T) {
	v := New()
	v.SetConfigType("yml")
	err := v.ReadConfig(bytes.NewBuffer(yamlMergeExampleTgt))
	require.NoError(t, err)

	assertFn := func(i int) {
		large := v.GetInt64("hello.largenum")
		pop := v.GetInt("hello.pop")
		assert.Equal(t, int64(765432101234567), large)
		assert.Equal(t, i, pop)
	}

	assertFn(37890)

	update := map[string]any{
		"Hello": map[string]any{
			"Pop": 1234,
		},
		"World": map[any]any{
			"Rock": 345,
		},
	}

	err = v.MergeConfigMap(update)
	require.NoError(t, err)

	assert.Equal(t, 345, v.GetInt("world.rock"))

	assertFn(1234)
}

func TestUnmarshalingWithAliases(t *testing.T) {
	v := New()
	v.SetDefault("ID", 1)
	v.Set("name", "Steve")
	v.Set("lastname", "Owen")

	v.RegisterAlias("UserID", "ID")
	v.RegisterAlias("Firstname", "name")
	v.RegisterAlias("Surname", "lastname")

	type config struct {
		ID        int
		FirstName string
		Surname   string
	}

	var C config
	err := v.Unmarshal(&C)
	require.NoError(t, err, "unable to decode into struct")

	assert.Equal(t, &config{ID: 1, FirstName: "Steve", Surname: "Owen"}, &C)
}

func TestSetConfigNameClearsFileCache(t *testing.T) {
	v := New()
	v.SetConfigFile("/tmp/config.yaml")
	v.SetConfigName("default")
	f, err := v.getConfigFile()
	require.Error(t, err, "config file cache should have been cleared")
	assert.Empty(t, f)
}

func TestShadowedNestedValue(t *testing.T) {
	v := New()
	config := `name: steve
clothing:
  jacket: leather
  trousers: denim
  pants:
    size: large
`
	initConfig("yaml", config, v)

	assert.Equal(t, "steve", v.GetString("name"))

	polyester := "polyester"
	v.SetDefault("clothing.shirt", polyester)
	v.SetDefault("clothing.jacket.price", 100)

	assert.Equal(t, "leather", v.GetString("clothing.jacket"))
	assert.Nil(t, v.Get("clothing.jacket.price"))
	assert.Equal(t, polyester, v.GetString("clothing.shirt"))

	clothingSettings := v.AllSettings()["clothing"].(map[string]any)
	assert.Equal(t, "leather", clothingSettings["jacket"])
	assert.Equal(t, polyester, clothingSettings["shirt"])
}

func TestDotParameter(t *testing.T) {
	v := New()

	v.SetConfigType("json")

	// Read the YAML data into Viper configuration
	require.NoError(t, v.ReadConfig(bytes.NewBuffer(jsonExample)), "Error reading YAML data")

	// should take precedence over batters defined in jsonExample
	r := bytes.NewReader([]byte(`{ "batters.batter": [ { "type": "Small" } ] }`))
	v.unmarshalReader(r, v.config)

	actual := v.Get("batters.batter")
	expected := []any{map[string]any{"type": "Small"}}
	assert.Equal(t, expected, actual)
}

func TestCaseInsensitive(t *testing.T) {
	for _, config := range []struct {
		typ     string
		content string
	}{
		{"yaml", `
aBcD: 1
eF:
  gH: 2
  iJk: 3
  Lm:
    nO: 4
    P:
      Q: 5
      R: 6
`},
		{"json", `{
  "aBcD": 1,
  "eF": {
    "iJk": 3,
    "Lm": {
      "P": {
        "Q": 5,
        "R": 6
      },
      "nO": 4
    },
    "gH": 2
  }
}`},
		{"toml", `aBcD = 1
[eF]
gH = 2
iJk = 3
[eF.Lm]
nO = 4
[eF.Lm.P]
Q = 5
R = 6
`},
	} {
		doTestCaseInsensitive(t, config.typ, config.content)
	}
}

func TestCaseInsensitiveSet(t *testing.T) {
	v := New()
	m1 := map[string]any{
		"Foo": 32,
		"Bar": map[any]any{
			"ABc": "A",
			"cDE": "B",
		},
	}

	m2 := map[string]any{
		"Foo": 52,
		"Bar": map[any]any{
			"bCd": "A",
			"eFG": "B",
		},
	}

	v.Set("Given1", m1)
	v.Set("Number1", 42)

	v.SetDefault("Given2", m2)
	v.SetDefault("Number2", 52)

	// Verify SetDefault
	assert.Equal(t, 52, v.Get("number2"))
	assert.Equal(t, 52, v.Get("given2.foo"))
	assert.Equal(t, "A", v.Get("given2.bar.bcd"))
	_, ok := m2["Foo"]
	assert.True(t, ok)

	// Verify Set
	assert.Equal(t, 42, v.Get("number1"))
	assert.Equal(t, 32, v.Get("given1.foo"))
	assert.Equal(t, "A", v.Get("given1.bar.abc"))
	_, ok = m1["Foo"]
	assert.True(t, ok)
}

func TestParseNested(t *testing.T) {
	v := New()
	type duration struct {
		Delay time.Duration
	}

	type item struct {
		Name   string
		Delay  time.Duration
		Nested duration
	}

	config := `[[parent]]
	delay="100ms"
	[parent.nested]
	delay="200ms"
`
	initConfig("toml", config, v)

	var items []item
	err := v.UnmarshalKey("parent", &items)
	require.NoError(t, err, "unable to decode into struct")

	assert.Len(t, items, 1)
	assert.Equal(t, 100*time.Millisecond, items[0].Delay)
	assert.Equal(t, 200*time.Millisecond, items[0].Nested.Delay)
}

func doTestCaseInsensitive(t *testing.T, typ, config string) {
	v := New()
	initConfig(typ, config, v)
	v.Set("RfD", true)
	assert.Equal(t, true, v.Get("rfd"))
	assert.Equal(t, true, v.Get("rFD"))
	assert.Equal(t, 1, cast.ToInt(v.Get("abcd")))
	assert.Equal(t, 1, cast.ToInt(v.Get("Abcd")))
	assert.Equal(t, 2, cast.ToInt(v.Get("ef.gh")))
	assert.Equal(t, 3, cast.ToInt(v.Get("ef.ijk")))
	assert.Equal(t, 4, cast.ToInt(v.Get("ef.lm.no")))
	assert.Equal(t, 5, cast.ToInt(v.Get("ef.lm.p.q")))
}

func newViperWithConfigFile(t *testing.T) (*Viper, string) {
	watchDir := t.TempDir()
	configFile := path.Join(watchDir, "config.yaml")
	err := os.WriteFile(configFile, []byte("foo: bar\n"), 0o640)
	require.NoError(t, err)
	v := New()
	v.SetConfigFile(configFile)
	err = v.ReadInConfig()
	require.NoError(t, err)
	require.Equal(t, "bar", v.Get("foo"))
	return v, configFile
}

func newViperWithSymlinkedConfigFile(t *testing.T) (*Viper, string, string) {
	watchDir := t.TempDir()
	dataDir1 := path.Join(watchDir, "data1")
	err := os.Mkdir(dataDir1, 0o777)
	require.NoError(t, err)
	realConfigFile := path.Join(dataDir1, "config.yaml")
	t.Logf("Real config file location: %s\n", realConfigFile)
	err = os.WriteFile(realConfigFile, []byte("foo: bar\n"), 0o640)
	require.NoError(t, err)
	// now, symlink the tm `data1` dir to `data` in the baseDir
	os.Symlink(dataDir1, path.Join(watchDir, "data"))
	// and link the `<watchdir>/datadir1/config.yaml` to `<watchdir>/config.yaml`
	configFile := path.Join(watchDir, "config.yaml")
	os.Symlink(path.Join(watchDir, "data", "config.yaml"), configFile)
	t.Logf("Config file location: %s\n", path.Join(watchDir, "config.yaml"))
	// init Viper
	v := New()
	v.SetConfigFile(configFile)
	err = v.ReadInConfig()
	require.NoError(t, err)
	require.Equal(t, "bar", v.Get("foo"))
	return v, watchDir, configFile
}

func TestWatchFile(t *testing.T) {
	if runtime.GOOS == "linux" {
		// TODO(bep) FIX ME
		t.Skip("Skip test on Linux ...")
	}

	t.Run("file content changed", func(t *testing.T) {
		// given a `config.yaml` file being watched
		v, configFile := newViperWithConfigFile(t)
		_, err := os.Stat(configFile)
		require.NoError(t, err)
		t.Logf("test config file: %s\n", configFile)
		wg := sync.WaitGroup{}
		wg.Add(1)
		var wgDoneOnce sync.Once // OnConfigChange is called twice on Windows
		v.OnConfigChange(func(_ fsnotify.Event) {
			t.Logf("config file changed")
			wgDoneOnce.Do(func() {
				wg.Done()
			})
		})
		v.WatchConfig()
		// when overwriting the file and waiting for the custom change notification handler to be triggered
		err = os.WriteFile(configFile, []byte("foo: baz\n"), 0o640)
		wg.Wait()
		// then the config value should have changed
		require.NoError(t, err)
		assert.Equal(t, "baz", v.Get("foo"))
	})

	t.Run("link to real file changed (à la Kubernetes)", func(t *testing.T) {
		// skip if not executed on Linux
		if runtime.GOOS != "linux" {
			t.Skipf("Skipping test as symlink replacements don't work on non-linux environment...")
		}
		v, watchDir, _ := newViperWithSymlinkedConfigFile(t)
		wg := sync.WaitGroup{}
		v.WatchConfig()
		v.OnConfigChange(func(_ fsnotify.Event) {
			t.Logf("config file changed")
			wg.Done()
		})
		wg.Add(1)
		// when link to another `config.yaml` file
		dataDir2 := path.Join(watchDir, "data2")
		err := os.Mkdir(dataDir2, 0o777)
		require.NoError(t, err)
		configFile2 := path.Join(dataDir2, "config.yaml")
		err = os.WriteFile(configFile2, []byte("foo: baz\n"), 0o640)
		require.NoError(t, err)
		// change the symlink using the `ln -sfn` command
		err = exec.Command("ln", "-sfn", dataDir2, path.Join(watchDir, "data")).Run()
		require.NoError(t, err)
		wg.Wait()
		// then
		require.NoError(t, err)
		assert.Equal(t, "baz", v.Get("foo"))
	})
}

func TestUnmarshal_DotSeparatorBackwardCompatibility(t *testing.T) {
	flags := pflag.NewFlagSet("test", pflag.ContinueOnError)
	flags.String("foo.bar", "cobra_flag", "")

	v := New()
	assert.NoError(t, v.BindPFlags(flags))

	config := &struct {
		Foo struct {
			Bar string
		}
	}{}

	assert.NoError(t, v.Unmarshal(config))
	assert.Equal(t, "cobra_flag", config.Foo.Bar)
}

// var yamlExampleWithDot = []byte(`Hacker: true
// name: steve
// hobbies:
//     - skateboarding
//     - snowboarding
//     - go
// clothing:
//     jacket: leather
//     trousers: denim
//     pants:
//         size: large
// age: 35
// eyes : brown
// beard: true
// emails:
//     steve@hacker.com:
//         created: 01/02/03
//         active: true
// `)

func TestKeyDelimiter(t *testing.T) {
	v := NewWithOptions(KeyDelimiter("::"))
	v.SetConfigType("yaml")
	r := strings.NewReader(string(yamlExampleWithDot))

	err := v.unmarshalReader(r, v.config)
	require.NoError(t, err)

	values := map[string]any{
		"image": map[string]any{
			"repository": "someImage",
			"tag":        "1.0.0",
		},
		"ingress": map[string]any{
			"annotations": map[string]any{
				"traefik.frontend.rule.type":                 "PathPrefix",
				"traefik.ingress.kubernetes.io/ssl-redirect": "true",
			},
		},
	}

	v.SetDefault("charts::values", values)

	assert.Equal(t, "leather", v.GetString("clothing::jacket"))
	assert.Equal(t, "01/02/03", v.GetString("emails::steve@hacker.com::created"))

	type config struct {
		Charts struct {
			Values map[string]any
		}
	}

	expected := config{
		Charts: struct {
			Values map[string]any
		}{
			Values: values,
		},
	}

	var actual config

	assert.NoError(t, v.Unmarshal(&actual))

	assert.Equal(t, expected, actual)
}

var yamlDeepNestedSlices = []byte(`TV:
- title: "The Expanse"
  title_i18n:
    USA: "The Expanse"
    Japan: "エクスパンス -巨獣めざめる-"
  seasons:
  - first_released: "December 14, 2015"
    episodes:
    - title: "Dulcinea"
      air_date: "December 14, 2015"
    - title: "The Big Empty"
      air_date: "December 15, 2015"
    - title: "Remember the Cant"
      air_date: "December 22, 2015"
  - first_released: "February 1, 2017"
    episodes:
    - title: "Safe"
      air_date: "February 1, 2017"
    - title: "Doors & Corners"
      air_date: "February 1, 2017"
    - title: "Static"
      air_date: "February 8, 2017"
  episodes:
    - ["Dulcinea", "The Big Empty", "Remember the Cant"]
    - ["Safe", "Doors & Corners", "Static"]
`)

func TestSliceIndexAccess(t *testing.T) {
	v := New()
	v.SetConfigType("yaml")
	r := strings.NewReader(string(yamlDeepNestedSlices))

	err := v.unmarshalReader(r, v.config)
	require.NoError(t, err)

	assert.Equal(t, "The Expanse", v.GetString("tv.0.title"))
	assert.Equal(t, "February 1, 2017", v.GetString("tv.0.seasons.1.first_released"))
	assert.Equal(t, "Static", v.GetString("tv.0.seasons.1.episodes.2.title"))
	assert.Equal(t, "December 15, 2015", v.GetString("tv.0.seasons.0.episodes.1.air_date"))

	// Test nested keys with capital letters
	assert.Equal(t, "The Expanse", v.GetString("tv.0.title_i18n.USA"))
	assert.Equal(t, "エクスパンス -巨獣めざめる-", v.GetString("tv.0.title_i18n.Japan"))

	// Test for index out of bounds
	assert.Equal(t, "", v.GetString("tv.0.seasons.2.first_released"))

	// Accessing multidimensional arrays
	assert.Equal(t, "Static", v.GetString("tv.0.episodes.1.2"))
}

func TestIsPathShadowedInFlatMap(t *testing.T) {
	v := New()

	stringMap := map[string]string{
		"foo": "value",
	}

	flagMap := map[string]FlagValue{
		"foo": pflagValue{},
	}

	path1 := []string{"foo", "bar"}
	expected1 := "foo"

	// "foo.bar" should shadowed by "foo"
	assert.Equal(t, expected1, v.isPathShadowedInFlatMap(path1, stringMap))
	assert.Equal(t, expected1, v.isPathShadowedInFlatMap(path1, flagMap))

	path2 := []string{"bar", "foo"}
	expected2 := ""

	// "bar.foo" should not shadowed by "foo"
	assert.Equal(t, expected2, v.isPathShadowedInFlatMap(path2, stringMap))
	assert.Equal(t, expected2, v.isPathShadowedInFlatMap(path2, flagMap))
}

func TestFlagShadow(t *testing.T) {
	v := New()

	v.SetDefault("foo.bar1.bar2", "default")

	flags := pflag.NewFlagSet("test", pflag.ContinueOnError)
	flags.String("foo.bar1", "shadowed", "")
	flags.VisitAll(func(flag *pflag.Flag) {
		flag.Changed = true
	})

	v.BindPFlags(flags)

	assert.Equal(t, "shadowed", v.GetString("foo.bar1"))
	// the default "foo.bar1.bar2" value should shadowed by flag "foo.bar1" value
	// and should return an empty string
	assert.Equal(t, "", v.GetString("foo.bar1.bar2"))
}

func BenchmarkGetBool(b *testing.B) {
	key := "BenchmarkGetBool"
	v = New()
	v.Set(key, true)

	for i := 0; i < b.N; i++ {
		if !v.GetBool(key) {
			b.Fatal("GetBool returned false")
		}
	}
}

func BenchmarkGet(b *testing.B) {
	key := "BenchmarkGet"
	v = New()
	v.Set(key, true)

	for i := 0; i < b.N; i++ {
		if !v.Get(key).(bool) {
			b.Fatal("Get returned false")
		}
	}
}

// BenchmarkGetBoolFromMap is the "perfect result" for the above.
func BenchmarkGetBoolFromMap(b *testing.B) {
	m := make(map[string]bool)
	key := "BenchmarkGetBool"
	m[key] = true

	for i := 0; i < b.N; i++ {
		if !m[key] {
			b.Fatal("Map value was false")
		}
	}
}

// Skip some tests on Windows that kept failing when Windows was added to the CI as a target.
func skipWindows(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("Skip test on Windows")
	}
}
