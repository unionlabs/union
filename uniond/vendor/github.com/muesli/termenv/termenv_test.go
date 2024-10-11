package termenv

import (
	"bytes"
	"fmt"
	"image/color"
	"io/ioutil"
	"os"
	"strings"
	"testing"
	"text/template"
)

func TestLegacyTermEnv(t *testing.T) {
	p := ColorProfile()
	if p != TrueColor && p != Ascii {
		t.Errorf("Expected %d, got %d", TrueColor, p)
	}

	fg := ForegroundColor()
	fgseq := fg.Sequence(false)
	fgexp := "97"
	if fgseq != fgexp && fgseq != "" {
		t.Errorf("Expected %s, got %s", fgexp, fgseq)
	}

	bg := BackgroundColor()
	bgseq := bg.Sequence(true)
	bgexp := "48;2;0;0;0"
	if bgseq != bgexp && bgseq != "" {
		t.Errorf("Expected %s, got %s", bgexp, bgseq)
	}

	_ = HasDarkBackground()
}

func TestTermEnv(t *testing.T) {
	o := NewOutput(os.Stdout)
	if o.Profile != TrueColor && o.Profile != Ascii {
		t.Errorf("Expected %d, got %d", TrueColor, o.Profile)
	}

	fg := o.ForegroundColor()
	fgseq := fg.Sequence(false)
	fgexp := "97"
	if fgseq != fgexp && fgseq != "" {
		t.Errorf("Expected %s, got %s", fgexp, fgseq)
	}

	bg := o.BackgroundColor()
	bgseq := bg.Sequence(true)
	bgexp := "48;2;0;0;0"
	if bgseq != bgexp && bgseq != "" {
		t.Errorf("Expected %s, got %s", bgexp, bgseq)
	}

	_ = HasDarkBackground()
}

func TestRendering(t *testing.T) {
	out := String("foobar")
	if out.String() != "foobar" {
		t.Errorf("Unstyled strings should be returned as plain text")
	}

	out = out.Foreground(TrueColor.Color("#abcdef"))
	out = out.Background(TrueColor.Color("69"))
	out = out.Bold()
	out = out.Italic()
	out = out.Faint()
	out = out.Underline()
	out = out.Blink()

	exp := "\x1b[38;2;171;205;239;48;5;69;1;3;2;4;5mfoobar\x1b[0m"
	if out.String() != exp {
		t.Errorf("Expected %s, got %s", exp, out.String())
	}

	exp = "foobar"
	mono := String(exp)
	mono = mono.Foreground(Ascii.Color("#abcdef"))
	if mono.String() != exp {
		t.Errorf("Ascii profile should not apply color styles")
	}
}

func TestColorConversion(t *testing.T) {
	// ANSI color
	a := ANSI.Color("7")
	c := ConvertToRGB(a)

	exp := "#c0c0c0"
	if c.Hex() != exp {
		t.Errorf("Expected %s, got %s", exp, c.Hex())
	}

	// ANSI-256 color
	a256 := ANSI256.Color("91")
	c = ConvertToRGB(a256)

	exp = "#8700af"
	if c.Hex() != exp {
		t.Errorf("Expected %s, got %s", exp, c.Hex())
	}

	// hex color
	hex := "#abcdef"
	argb := TrueColor.Color(hex)
	c = ConvertToRGB(argb)

	if c.Hex() != hex {
		t.Errorf("Expected %s, got %s", exp, c.Hex())
	}
}

func TestFromColor(t *testing.T) {
	// color.Color interface
	c := TrueColor.FromColor(color.RGBA{255, 128, 0, 255})
	exp := "38;2;255;128;0"
	if c.Sequence(false) != exp {
		t.Errorf("Expected %s, got %s", exp, c.Sequence(false))
	}
}

func TestAscii(t *testing.T) {
	c := Ascii.Color("#abcdef")
	if c.Sequence(false) != "" {
		t.Errorf("Expected empty sequence, got %s", c.Sequence(false))
	}
}

func TestANSIProfile(t *testing.T) {
	p := ANSI

	c := p.Color("#e88388")
	exp := "91"
	if c.Sequence(false) != exp {
		t.Errorf("Expected %s, got %s", exp, c.Sequence(false))
	}
	if _, ok := c.(ANSIColor); !ok {
		t.Errorf("Expected type termenv.ANSIColor, got %T", c)
	}

	c = p.Color("82")
	exp = "92"
	if c.Sequence(false) != exp {
		t.Errorf("Expected %s, got %s", exp, c.Sequence(false))
	}
	if _, ok := c.(ANSIColor); !ok {
		t.Errorf("Expected type termenv.ANSIColor, got %T", c)
	}

	c = p.Color("2")
	exp = "32"
	if c.Sequence(false) != exp {
		t.Errorf("Expected %s, got %s", exp, c.Sequence(false))
	}
	if _, ok := c.(ANSIColor); !ok {
		t.Errorf("Expected type termenv.ANSIColor, got %T", c)
	}
}

func TestANSI256Profile(t *testing.T) {
	p := ANSI256

	c := p.Color("#abcdef")
	exp := "38;5;153"
	if c.Sequence(false) != exp {
		t.Errorf("Expected %s, got %s", exp, c.Sequence(false))
	}
	if _, ok := c.(ANSI256Color); !ok {
		t.Errorf("Expected type termenv.ANSI256Color, got %T", c)
	}

	c = p.Color("139")
	exp = "38;5;139"
	if c.Sequence(false) != exp {
		t.Errorf("Expected %s, got %s", exp, c.Sequence(false))
	}
	if _, ok := c.(ANSI256Color); !ok {
		t.Errorf("Expected type termenv.ANSI256Color, got %T", c)
	}

	c = p.Color("2")
	exp = "32"
	if c.Sequence(false) != exp {
		t.Errorf("Expected %s, got %s", exp, c.Sequence(false))
	}
	if _, ok := c.(ANSIColor); !ok {
		t.Errorf("Expected type termenv.ANSIColor, got %T", c)
	}
}

func TestTrueColorProfile(t *testing.T) {
	p := TrueColor

	c := p.Color("#abcdef")
	exp := "38;2;171;205;239"
	if c.Sequence(false) != exp {
		t.Errorf("Expected %s, got %s", exp, c.Sequence(false))
	}
	if _, ok := c.(RGBColor); !ok {
		t.Errorf("Expected type termenv.HexColor, got %T", c)
	}

	c = p.Color("139")
	exp = "38;5;139"
	if c.Sequence(false) != exp {
		t.Errorf("Expected %s, got %s", exp, c.Sequence(false))
	}
	if _, ok := c.(ANSI256Color); !ok {
		t.Errorf("Expected type termenv.ANSI256Color, got %T", c)
	}

	c = p.Color("2")
	exp = "32"
	if c.Sequence(false) != exp {
		t.Errorf("Expected %s, got %s", exp, c.Sequence(false))
	}
	if _, ok := c.(ANSIColor); !ok {
		t.Errorf("Expected type termenv.ANSIColor, got %T", c)
	}
}

func TestStyles(t *testing.T) {
	s := String("foobar").Foreground(TrueColor.Color("2"))

	exp := "\x1b[32mfoobar\x1b[0m"
	if s.String() != exp {
		t.Errorf("Expected %s, got %s", exp, s.String())
	}
}

func TestTemplateHelpers(t *testing.T) {
	p := TrueColor

	exp := String("Hello World")
	basetpl := `{{ %s "Hello World" }}`
	wraptpl := `{{ %s (%s "Hello World") }}`

	tt := []struct {
		Template string
		Expected string
	}{
		{
			Template: fmt.Sprintf(basetpl, "Bold"),
			Expected: exp.Bold().String(),
		},
		{
			Template: fmt.Sprintf(basetpl, "Faint"),
			Expected: exp.Faint().String(),
		},
		{
			Template: fmt.Sprintf(basetpl, "Italic"),
			Expected: exp.Italic().String(),
		},
		{
			Template: fmt.Sprintf(basetpl, "Underline"),
			Expected: exp.Underline().String(),
		},
		{
			Template: fmt.Sprintf(basetpl, "Overline"),
			Expected: exp.Overline().String(),
		},
		{
			Template: fmt.Sprintf(basetpl, "Blink"),
			Expected: exp.Blink().String(),
		},
		{
			Template: fmt.Sprintf(basetpl, "Reverse"),
			Expected: exp.Reverse().String(),
		},
		{
			Template: fmt.Sprintf(basetpl, "CrossOut"),
			Expected: exp.CrossOut().String(),
		},
		{
			Template: fmt.Sprintf(wraptpl, "Underline", "Bold"),
			Expected: String(exp.Bold().String()).Underline().String(),
		},
		{
			Template: `{{ Color "#ff0000" "foobar" }}`,
			Expected: String("foobar").Foreground(p.Color("#ff0000")).String(),
		},
		{
			Template: `{{ Color "#ff0000" "#0000ff" "foobar" }}`,
			Expected: String("foobar").
				Foreground(p.Color("#ff0000")).
				Background(p.Color("#0000ff")).
				String(),
		},
		{
			Template: `{{ Foreground "#ff0000" "foobar" }}`,
			Expected: String("foobar").Foreground(p.Color("#ff0000")).String(),
		},
		{
			Template: `{{ Background "#ff0000" "foobar" }}`,
			Expected: String("foobar").Background(p.Color("#ff0000")).String(),
		},
	}

	for i, v := range tt {
		tpl, err := template.New(fmt.Sprintf("test_%d", i)).Funcs(TemplateFuncs(p)).Parse(v.Template)
		if err != nil {
			t.Error(err)
		}

		var buf bytes.Buffer
		err = tpl.Execute(&buf, nil)
		if err != nil {
			t.Error(err)
		}

		if buf.String() != v.Expected {
			v1 := strings.Replace(v.Expected, "\x1b", "", -1)
			v2 := strings.Replace(buf.String(), "\x1b", "", -1)
			t.Errorf("Expected %s, got %s", v1, v2)
		}
	}
}

func TestEnvNoColor(t *testing.T) {
	tests := []struct {
		name     string
		environ  []string
		expected bool
	}{
		{"no env", nil, false},
		{"no_color", []string{"NO_COLOR", "Y"}, true},
		{"no_color+clicolor=1", []string{"NO_COLOR", "Y", "CLICOLOR", "1"}, true},
		{"no_color+clicolor_force=1", []string{"NO_COLOR", "Y", "CLICOLOR_FORCE", "1"}, true},
		{"clicolor=0", []string{"CLICOLOR", "0"}, true},
		{"clicolor=1", []string{"CLICOLOR", "1"}, false},
		{"clicolor_force=1", []string{"CLICOLOR_FORCE", "0"}, false},
		{"clicolor_force=0", []string{"CLICOLOR_FORCE", "1"}, false},
		{"clicolor=0+clicolor_force=1", []string{"CLICOLOR", "0", "CLICOLOR_FORCE", "1"}, false},
		{"clicolor=1+clicolor_force=1", []string{"CLICOLOR", "1", "CLICOLOR_FORCE", "1"}, false},
		{"clicolor=0+clicolor_force=0", []string{"CLICOLOR", "0", "CLICOLOR_FORCE", "0"}, true},
		{"clicolor=1+clicolor_force=0", []string{"CLICOLOR", "1", "CLICOLOR_FORCE", "0"}, false},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			defer func() {
				os.Unsetenv("NO_COLOR")
				os.Unsetenv("CLICOLOR")
				os.Unsetenv("CLICOLOR_FORCE")
			}()
			for i := 0; i < len(test.environ); i += 2 {
				os.Setenv(test.environ[i], test.environ[i+1])
			}
			out := NewOutput(os.Stdout)
			actual := out.EnvNoColor()
			if test.expected != actual {
				t.Errorf("expected %t but was %t", test.expected, actual)
			}
		})
	}
}

func TestPseudoTerm(t *testing.T) {
	buf := &bytes.Buffer{}
	o := NewOutput(buf)
	if o.Profile != Ascii {
		t.Errorf("Expected %d, got %d", Ascii, o.Profile)
	}

	fg := o.ForegroundColor()
	fgseq := fg.Sequence(false)
	if fgseq != "" {
		t.Errorf("Expected empty response, got %s", fgseq)
	}

	bg := o.BackgroundColor()
	bgseq := bg.Sequence(true)
	if bgseq != "" {
		t.Errorf("Expected empty response, got %s", bgseq)
	}

	exp := "foobar"
	out := o.String(exp)
	out = out.Foreground(o.Color("#abcdef"))
	o.Write([]byte(out.String()))

	if buf.String() != exp {
		t.Errorf("Expected %s, got %s", exp, buf.String())
	}
}

func TestCache(t *testing.T) {
	o := NewOutput(os.Stdout, WithColorCache(true), WithProfile(TrueColor))

	if o.cache != true {
		t.Errorf("Expected cache to be active, got %t", o.cache)
	}
}

func TestEnableVirtualTerminalProcessing(t *testing.T) {
	// EnableVirtualTerminalProcessing should always return a non-nil
	// restoreFunc, and in tests it should never return an error.
	restoreFunc, err := EnableVirtualTerminalProcessing(NewOutput(os.Stdout))
	if restoreFunc == nil || err != nil {
		t.Fatalf("expected non-<nil>, <nil>, got %p, %v", restoreFunc, err)
	}
	// In tests, restoreFunc should never return an error.
	if err := restoreFunc(); err != nil {
		t.Fatalf("expected <nil>, got %v", err)
	}
}

func TestWithTTY(t *testing.T) {
	for _, v := range []bool{true, false} {
		o := NewOutput(ioutil.Discard, WithTTY(v))
		if o.isTTY() != v {
			t.Fatalf("expected WithTTY(%t) to set isTTY to %t", v, v)
		}
	}
}
