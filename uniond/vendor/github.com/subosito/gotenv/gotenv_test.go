package gotenv_test

import (
	"bufio"
	"errors"
	"io"
	"os"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/subosito/gotenv"
)

var formats = []struct {
	in     string
	out    gotenv.Env
	preset bool
}{
	// parses unquoted values
	{`FOO=bar`, gotenv.Env{"FOO": "bar"}, false},

	// parses values with spaces around equal sign
	{`FOO =bar`, gotenv.Env{"FOO": "bar"}, false},
	{`FOO= bar`, gotenv.Env{"FOO": "bar"}, false},

	// parses values with leading spaces
	{`  FOO=bar`, gotenv.Env{"FOO": "bar"}, false},

	// parses values with following spaces
	{`FOO=bar  `, gotenv.Env{"FOO": "bar"}, false},

	// parses double quoted values
	{`FOO="bar"`, gotenv.Env{"FOO": "bar"}, false},

	// parses double quoted values with following spaces
	{`FOO="bar"  `, gotenv.Env{"FOO": "bar"}, false},

	// parses single quoted values
	{`FOO='bar'`, gotenv.Env{"FOO": "bar"}, false},

	// parses single quoted values with following spaces
	{`FOO='bar'  `, gotenv.Env{"FOO": "bar"}, false},

	// parses escaped double quotes
	{`FOO="escaped\"bar"`, gotenv.Env{"FOO": `escaped"bar`}, false},

	// parses empty values
	{`FOO=`, gotenv.Env{"FOO": ""}, false},

	// expands variables found in values
	{"FOO=test\nBAR=$FOO", gotenv.Env{"FOO": "test", "BAR": "test"}, false},

	// parses variables wrapped in brackets
	{"FOO=test\nBAR=${FOO}bar", gotenv.Env{"FOO": "test", "BAR": "testbar"}, false},

	// reads variables from ENV when expanding if not found in local env
	{`BAR=$FOO`, gotenv.Env{"BAR": "test"}, true},

	// expands undefined variables to an empty string
	{`BAR=$FOO`, gotenv.Env{"BAR": ""}, false},

	// expands variables in quoted strings
	{"FOO=test\nBAR=\"quote $FOO\"", gotenv.Env{"FOO": "test", "BAR": "quote test"}, false},

	// does not expand variables in single quoted strings
	{"BAR='quote $FOO'", gotenv.Env{"BAR": "quote $FOO"}, false},

	// does not expand escaped variables
	{`FOO="foo\$BAR"`, gotenv.Env{"FOO": "foo$BAR"}, false},
	{`FOO="foo\${BAR}"`, gotenv.Env{"FOO": "foo${BAR}"}, false},
	{"FOO=test\nBAR=\"foo\\${FOO} ${FOO}\"", gotenv.Env{"FOO": "test", "BAR": "foo${FOO} test"}, false},

	// parses yaml style options
	{"OPTION_A: 1", gotenv.Env{"OPTION_A": "1"}, false},

	// parses export keyword
	{"export OPTION_A=2", gotenv.Env{"OPTION_A": "2"}, false},

	// allows export line if you want to do it that way
	{"OPTION_A=2\nexport OPTION_A", gotenv.Env{"OPTION_A": "2"}, false},

	// expands newlines in quoted strings
	{`FOO="bar\nbaz"`, gotenv.Env{"FOO": "bar\nbaz"}, false},

	// parses variables with "." in the name
	{`FOO.BAR=foobar`, gotenv.Env{"FOO.BAR": "foobar"}, false},

	// strips unquoted values
	{`foo=bar `, gotenv.Env{"foo": "bar"}, false}, // not 'bar '

	// ignores empty lines
	{"\n \t  \nfoo=bar\n \nfizz=buzz", gotenv.Env{"foo": "bar", "fizz": "buzz"}, false},

	// ignores inline comments
	{"foo=bar # this is foo", gotenv.Env{"foo": "bar"}, false},

	// allows # in quoted value
	{`foo="bar#baz" # comment`, gotenv.Env{"foo": "bar#baz"}, false},

	// ignores comment lines
	{"\n\n\n # HERE GOES FOO \nfoo=bar", gotenv.Env{"foo": "bar"}, false},

	// parses # in quoted values
	{`foo="ba#r"`, gotenv.Env{"foo": "ba#r"}, false},
	{"foo='ba#r'", gotenv.Env{"foo": "ba#r"}, false},

	// parses # in quoted values with following spaces
	{`foo="ba#r"  `, gotenv.Env{"foo": "ba#r"}, false},
	{`foo='ba#r'  `, gotenv.Env{"foo": "ba#r"}, false},

	// supports carriage return
	{"FOO=bar\rbaz=fbb", gotenv.Env{"FOO": "bar", "baz": "fbb"}, false},

	// supports carriage return combine with new line
	{"FOO=bar\r\nbaz=fbb", gotenv.Env{"FOO": "bar", "baz": "fbb"}, false},

	// expands carriage return in quoted strings
	{`FOO="bar\rbaz"`, gotenv.Env{"FOO": "bar\rbaz"}, false},

	// escape $ properly when no alphabets/numbers/_  are followed by it
	{`FOO="bar\\$ \\$\\$"`, gotenv.Env{"FOO": "bar$ $$"}, false},

	// ignore $ when it is not escaped and no variable is followed by it
	{`FOO="bar $ "`, gotenv.Env{"FOO": "bar $ "}, false},

	// parses unquoted values with spaces after separator
	{`FOO= bar`, gotenv.Env{"FOO": "bar"}, false},

	// allows # in quoted value with spaces after separator
	{`foo= "bar#baz" # comment`, gotenv.Env{"foo": "bar#baz"}, false},

	// allows = in double quoted values with newlines (typically base64 padding)
	{`foo="---\na==\n---"`, gotenv.Env{"foo": "---\na==\n---"}, false},
}

var errorFormats = []struct {
	in  string
	out gotenv.Env
	err string
}{
	// allows export line if you want to do it that way and checks for unset variables
	{"OPTION_A=2\nexport OH_NO_NOT_SET", gotenv.Env{"OPTION_A": "2"}, "line `export OH_NO_NOT_SET` has an unset variable"},

	// throws an error if line format is incorrect
	{`lol$wut`, gotenv.Env{}, "line `lol$wut` doesn't match format"},
}

var fixtures = []struct {
	filename string
	results  gotenv.Env
}{
	{
		"fixtures/exported.env",
		gotenv.Env{
			"OPTION_A": "2",
			"OPTION_B": `\n`,
			"OPTION_C": `The MIT License (MIT)

Copyright (c) 2013 Alif Rachmawadi

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.`,
		},
	},
	{
		"fixtures/plain.env",
		gotenv.Env{
			"OPTION_A": "1",
			"OPTION_B": "2",
			"OPTION_C": "3",
			"OPTION_D": "4",
			"OPTION_E": "5",
		},
	},
	{
		"fixtures/quoted.env",
		gotenv.Env{
			"OPTION_A": "1",
			"OPTION_B": "2",
			"OPTION_C": "",
			"OPTION_D": `\n`,
			"OPTION_E": "1",
			"OPTION_F": "2",
			"OPTION_G": "",
			"OPTION_H": "\n",
			"OPTION_I": `some multi-line text
with "escaped quotes" and 1 variable`,
			"OPTION_J": `some$pecial$1$2!*chars=qweq""e$$\$""`,
			"OPTION_K": "\n",
			"OPTION_L": `some multi-line text
with "escaped quotes"
empty lines

and 1 variable
`,
		},
	},
	{
		"fixtures/yaml.env",
		gotenv.Env{
			"OPTION_A": "1",
			"OPTION_B": "2",
			"OPTION_C": "",
			"OPTION_D": `\n`,
		},
	},
}

func TestParse(t *testing.T) {
	for _, tt := range formats {
		if tt.preset {
			os.Setenv("FOO", "test")
		}

		exp := gotenv.Parse(strings.NewReader(tt.in))
		assert.Equal(t, tt.out, exp)
		os.Clearenv()
	}
}

func TestStrictParse(t *testing.T) {
	for _, tt := range errorFormats {
		env, err := gotenv.StrictParse(strings.NewReader(tt.in))
		assert.Equal(t, tt.err, err.Error())
		assert.Equal(t, tt.out, env)
	}
}

type failingReader struct {
	io.Reader
}

func (fr failingReader) Read(p []byte) (n int, err error) {
	return 0, errors.New("you shall not read")
}

func TestStrictParse_PassThroughErrors(t *testing.T) {
	_, err := gotenv.StrictParse(&failingReader{})
	assert.Error(t, err)
}

type infiniteReader struct {
	io.Reader
}

func (er infiniteReader) Read(p []byte) (n int, err error) {
	return len(p), nil
}

func TestStrictParse_NoTokenPassThroughErrors(t *testing.T) {
	_, err := gotenv.StrictParse(&infiniteReader{})
	assert.Error(t, err)
}

func TestRead(t *testing.T) {
	for _, tt := range fixtures {
		env, err := gotenv.Read(tt.filename)
		assert.Nil(t, err)

		for key, val := range tt.results {
			assert.Equal(t, val, env[key])
		}

		os.Clearenv()
	}
}

func TestLoad(t *testing.T) {
	for _, tt := range fixtures {
		err := gotenv.Load(tt.filename)
		assert.Nil(t, err)

		for key, val := range tt.results {
			assert.Equal(t, val, os.Getenv(key))
		}

		os.Clearenv()
	}
}

func TestLoad_default(t *testing.T) {
	k := "HELLO"
	v := "world"

	err := gotenv.Load()
	assert.Nil(t, err)
	assert.Equal(t, v, os.Getenv(k))
	os.Clearenv()
}

func TestLoad_overriding(t *testing.T) {
	k := "HELLO"
	v := "universe"

	os.Setenv(k, v)
	err := gotenv.Load()
	assert.Nil(t, err)
	assert.Equal(t, v, os.Getenv(k))
	os.Clearenv()
}

func TestLoad_overrideVars(t *testing.T) {
	os.Setenv("A", "fromEnv")
	err := gotenv.Load("fixtures/vars.env")
	assert.Nil(t, err)
	assert.Equal(t, "fromEnv", os.Getenv("B"))
	os.Clearenv()
}

func TestLoad_overrideVars2(t *testing.T) {
	os.Setenv("C", "fromEnv")
	err := gotenv.Load("fixtures/vars.env")
	assert.Nil(t, err)
	assert.Equal(t, "fromEnv", os.Getenv("D"))
	os.Clearenv()
}

func TestLoad_Env(t *testing.T) {
	err := gotenv.Load(".env.invalid")
	assert.NotNil(t, err)
}

func TestLoad_nonExist(t *testing.T) {
	file := ".env.not.exist"

	err := gotenv.Load(file)
	if err == nil {
		t.Errorf("gotenv.Load(`%s`) => error: `no such file or directory` != nil", file)
	}
}

func TestLoad_unicodeBOMFixture(t *testing.T) {
	file := "fixtures/utf8_bom.env"

	f, err := os.Open(file)
	assert.Nil(t, err)

	scanner := bufio.NewScanner(f)

	i := 1
	bom := string([]byte{239, 187, 191})

	for scanner.Scan() {
		if i == 1 {
			line := scanner.Text()
			assert.True(t, strings.HasPrefix(line, bom))
		}
	}
}

func TestLoad_BOM_UTF8(t *testing.T) {
	defer os.Clearenv()

	file := "fixtures/utf8_bom.env"

	if err := gotenv.Load(file); assert.Nil(t, err) {
		assert.Equal(t, "UTF-8", os.Getenv("BOM"))
	}
}

func TestLoad_BOM_UTF16_LE(t *testing.T) {
	defer os.Clearenv()

	file := "fixtures/utf16le_bom.env"

	if err := gotenv.Load(file); assert.Nil(t, err) {
		assert.Equal(t, "UTF-16 LE", os.Getenv("BOM"))
	}
}

func TestLoad_BOM_UTF16_BE(t *testing.T) {
	defer os.Clearenv()

	file := "fixtures/utf16be_bom.env"

	if err := gotenv.Load(file); assert.Nil(t, err) {
		assert.Equal(t, "UTF-16 BE", os.Getenv("BOM"))
	}
}

func TestMust_Load(t *testing.T) {
	for _, tt := range fixtures {
		assert.NotPanics(t, func() {
			gotenv.Must(gotenv.Load, tt.filename)

			for key, val := range tt.results {
				assert.Equal(t, val, os.Getenv(key))
			}

			os.Clearenv()
		}, "Caling gotenv.Must with gotenv.Load should NOT panic")
	}
}

func TestMust_Load_default(t *testing.T) {
	assert.NotPanics(t, func() {
		gotenv.Must(gotenv.Load)

		tkey := "HELLO"
		val := "world"

		assert.Equal(t, val, os.Getenv(tkey))
		os.Clearenv()
	}, "Caling gotenv.Must with gotenv.Load without arguments should NOT panic")
}

func TestMust_Load_nonExist(t *testing.T) {
	assert.Panics(t, func() { gotenv.Must(gotenv.Load, ".env.not.exist") }, "Caling gotenv.Must with gotenv.Load and non exist file SHOULD panic")
}

func TestOverLoad_overriding(t *testing.T) {
	k := "HELLO"
	v := "universe"

	os.Setenv(k, v)
	err := gotenv.OverLoad()
	assert.Nil(t, err)
	assert.Equal(t, "world", os.Getenv(k))
	os.Clearenv()
}

func TestOverLoad_overrideVars(t *testing.T) {
	os.Setenv("A", "fromEnv")
	err := gotenv.OverLoad("fixtures/vars.env")
	assert.Nil(t, err)
	assert.Equal(t, "fromFile", os.Getenv("B"))
	os.Clearenv()
}

func TestOverLoad_overrideVars2(t *testing.T) {
	os.Setenv("C", "fromEnv")
	err := gotenv.OverLoad("fixtures/vars.env")
	assert.Nil(t, err)
	// The value for D is not "fromFile" because C is defined after the
	// definition of D.
	assert.Equal(t, "fromEnv", os.Getenv("D"), "C defined after usage")
	os.Clearenv()
}

func TestMustOverLoad_nonExist(t *testing.T) {
	assert.Panics(t, func() { gotenv.Must(gotenv.OverLoad, ".env.not.exist") }, "Caling gotenv.Must with Overgotenv.Load and non exist file SHOULD panic")
}

func TestApply(t *testing.T) {
	os.Setenv("HELLO", "world")
	r := strings.NewReader("HELLO=universe")
	err := gotenv.Apply(r)
	assert.Nil(t, err)
	assert.Equal(t, "world", os.Getenv("HELLO"))
	os.Clearenv()
}

func TestOverApply(t *testing.T) {
	os.Setenv("HELLO", "world")
	r := strings.NewReader("HELLO=universe")
	err := gotenv.OverApply(r)
	assert.Nil(t, err)
	assert.Equal(t, "universe", os.Getenv("HELLO"))
	os.Clearenv()
}

func TestMarshal(t *testing.T) {
	env := gotenv.Env{
		"FOO":    "BAR",
		"ONE":    "1",
		"QUOTED": `some "quoted" text`,
		"EMPTY":  "",
	}
	expected := `EMPTY=""
FOO="BAR"
ONE=1
QUOTED="some \"quoted\" text"`

	actual, err := gotenv.Marshal(env)
	assert.Nil(t, err)
	assert.Equal(t, expected, actual)

	out, err := gotenv.Unmarshal(expected)
	assert.Nil(t, err)
	assert.Equal(t, env, out)
}
