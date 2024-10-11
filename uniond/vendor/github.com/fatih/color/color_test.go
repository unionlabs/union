package color

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"testing"

	"github.com/mattn/go-colorable"
)

// Testing colors is kinda different. First we test for given colors and their
// escaped formatted results. Next we create some visual tests to be tested.
// Each visual test includes the color name to be compared.
func TestColor(t *testing.T) {
	rb := new(bytes.Buffer)
	Output = rb

	NoColor = false

	testColors := []struct {
		text string
		code Attribute
	}{
		{text: "black", code: FgBlack},
		{text: "red", code: FgRed},
		{text: "green", code: FgGreen},
		{text: "yellow", code: FgYellow},
		{text: "blue", code: FgBlue},
		{text: "magent", code: FgMagenta},
		{text: "cyan", code: FgCyan},
		{text: "white", code: FgWhite},
		{text: "hblack", code: FgHiBlack},
		{text: "hred", code: FgHiRed},
		{text: "hgreen", code: FgHiGreen},
		{text: "hyellow", code: FgHiYellow},
		{text: "hblue", code: FgHiBlue},
		{text: "hmagent", code: FgHiMagenta},
		{text: "hcyan", code: FgHiCyan},
		{text: "hwhite", code: FgHiWhite},
	}

	for _, c := range testColors {
		New(c.code).Print(c.text)

		line, _ := rb.ReadString('\n')
		scannedLine := fmt.Sprintf("%q", line)
		colored := fmt.Sprintf("\x1b[%dm%s\x1b[0m", c.code, c.text)
		escapedForm := fmt.Sprintf("%q", colored)

		fmt.Printf("%s\t: %s\n", c.text, line)

		if scannedLine != escapedForm {
			t.Errorf("Expecting %s, got '%s'\n", escapedForm, scannedLine)
		}
	}

	for _, c := range testColors {
		line := New(c.code).Sprintf("%s", c.text)
		scannedLine := fmt.Sprintf("%q", line)
		colored := fmt.Sprintf("\x1b[%dm%s\x1b[0m", c.code, c.text)
		escapedForm := fmt.Sprintf("%q", colored)

		fmt.Printf("%s\t: %s\n", c.text, line)

		if scannedLine != escapedForm {
			t.Errorf("Expecting %s, got '%s'\n", escapedForm, scannedLine)
		}
	}
}

func TestColorEquals(t *testing.T) {
	fgblack1 := New(FgBlack)
	fgblack2 := New(FgBlack)
	bgblack := New(BgBlack)
	fgbgblack := New(FgBlack, BgBlack)
	fgblackbgred := New(FgBlack, BgRed)
	fgred := New(FgRed)
	bgred := New(BgRed)

	if !fgblack1.Equals(fgblack2) {
		t.Error("Two black colors are not equal")
	}

	if fgblack1.Equals(bgblack) {
		t.Error("Fg and bg black colors are equal")
	}

	if fgblack1.Equals(fgbgblack) {
		t.Error("Fg black equals fg/bg black color")
	}

	if fgblack1.Equals(fgred) {
		t.Error("Fg black equals Fg red")
	}

	if fgblack1.Equals(bgred) {
		t.Error("Fg black equals Bg red")
	}

	if fgblack1.Equals(fgblackbgred) {
		t.Error("Fg black equals fg black bg red")
	}
}

func TestNoColor(t *testing.T) {
	rb := new(bytes.Buffer)
	Output = rb

	testColors := []struct {
		text string
		code Attribute
	}{
		{text: "black", code: FgBlack},
		{text: "red", code: FgRed},
		{text: "green", code: FgGreen},
		{text: "yellow", code: FgYellow},
		{text: "blue", code: FgBlue},
		{text: "magent", code: FgMagenta},
		{text: "cyan", code: FgCyan},
		{text: "white", code: FgWhite},
		{text: "hblack", code: FgHiBlack},
		{text: "hred", code: FgHiRed},
		{text: "hgreen", code: FgHiGreen},
		{text: "hyellow", code: FgHiYellow},
		{text: "hblue", code: FgHiBlue},
		{text: "hmagent", code: FgHiMagenta},
		{text: "hcyan", code: FgHiCyan},
		{text: "hwhite", code: FgHiWhite},
	}

	for _, c := range testColors {
		p := New(c.code)
		p.DisableColor()
		p.Print(c.text)

		line, _ := rb.ReadString('\n')
		if line != c.text {
			t.Errorf("Expecting %s, got '%s'\n", c.text, line)
		}
	}

	// global check
	NoColor = true
	t.Cleanup(func() {
		NoColor = false
	})

	for _, c := range testColors {
		p := New(c.code)
		p.Print(c.text)

		line, _ := rb.ReadString('\n')
		if line != c.text {
			t.Errorf("Expecting %s, got '%s'\n", c.text, line)
		}
	}
}

func TestNoColor_Env(t *testing.T) {
	rb := new(bytes.Buffer)
	Output = rb

	testColors := []struct {
		text string
		code Attribute
	}{
		{text: "black", code: FgBlack},
		{text: "red", code: FgRed},
		{text: "green", code: FgGreen},
		{text: "yellow", code: FgYellow},
		{text: "blue", code: FgBlue},
		{text: "magent", code: FgMagenta},
		{text: "cyan", code: FgCyan},
		{text: "white", code: FgWhite},
		{text: "hblack", code: FgHiBlack},
		{text: "hred", code: FgHiRed},
		{text: "hgreen", code: FgHiGreen},
		{text: "hyellow", code: FgHiYellow},
		{text: "hblue", code: FgHiBlue},
		{text: "hmagent", code: FgHiMagenta},
		{text: "hcyan", code: FgHiCyan},
		{text: "hwhite", code: FgHiWhite},
	}

	os.Setenv("NO_COLOR", "1")
	t.Cleanup(func() {
		os.Unsetenv("NO_COLOR")
	})

	for _, c := range testColors {
		p := New(c.code)
		p.Print(c.text)

		line, _ := rb.ReadString('\n')
		if line != c.text {
			t.Errorf("Expecting %s, got '%s'\n", c.text, line)
		}
	}
}

func Test_noColorIsSet(t *testing.T) {
	tests := []struct {
		name string
		act  func()
		want bool
	}{
		{
			name: "default",
			act:  func() {},
			want: false,
		},
		{
			name: "NO_COLOR=1",
			act:  func() { os.Setenv("NO_COLOR", "1") },
			want: true,
		},
		{
			name: "NO_COLOR=",
			act:  func() { os.Setenv("NO_COLOR", "") },
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			t.Cleanup(func() {
				os.Unsetenv("NO_COLOR")
			})
			tt.act()
			if got := noColorIsSet(); got != tt.want {
				t.Errorf("noColorIsSet() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestColorVisual(t *testing.T) {
	// First Visual Test
	Output = colorable.NewColorableStdout()

	New(FgRed).Printf("red\t")
	New(BgRed).Print("         ")
	New(FgRed, Bold).Println(" red")

	New(FgGreen).Printf("green\t")
	New(BgGreen).Print("         ")
	New(FgGreen, Bold).Println(" green")

	New(FgYellow).Printf("yellow\t")
	New(BgYellow).Print("         ")
	New(FgYellow, Bold).Println(" yellow")

	New(FgBlue).Printf("blue\t")
	New(BgBlue).Print("         ")
	New(FgBlue, Bold).Println(" blue")

	New(FgMagenta).Printf("magenta\t")
	New(BgMagenta).Print("         ")
	New(FgMagenta, Bold).Println(" magenta")

	New(FgCyan).Printf("cyan\t")
	New(BgCyan).Print("         ")
	New(FgCyan, Bold).Println(" cyan")

	New(FgWhite).Printf("white\t")
	New(BgWhite).Print("         ")
	New(FgWhite, Bold).Println(" white")
	fmt.Println("")

	// Second Visual test
	Black("black")
	Red("red")
	Green("green")
	Yellow("yellow")
	Blue("blue")
	Magenta("magenta")
	Cyan("cyan")
	White("white")
	HiBlack("hblack")
	HiRed("hred")
	HiGreen("hgreen")
	HiYellow("hyellow")
	HiBlue("hblue")
	HiMagenta("hmagenta")
	HiCyan("hcyan")
	HiWhite("hwhite")

	// Third visual test
	fmt.Println()
	Set(FgBlue)
	fmt.Println("is this blue?")
	Unset()

	Set(FgMagenta)
	fmt.Println("and this magenta?")
	Unset()

	// Fourth Visual test
	fmt.Println()
	blue := New(FgBlue).PrintlnFunc()
	blue("blue text with custom print func")

	red := New(FgRed).PrintfFunc()
	red("red text with a printf func: %d\n", 123)

	put := New(FgYellow).SprintFunc()
	warn := New(FgRed).SprintFunc()

	fmt.Fprintf(Output, "this is a %s and this is %s.\n", put("warning"), warn("error"))

	info := New(FgWhite, BgGreen).SprintFunc()
	fmt.Fprintf(Output, "this %s rocks!\n", info("package"))

	notice := New(FgBlue).FprintFunc()
	notice(os.Stderr, "just a blue notice to stderr")

	// Fifth Visual Test
	fmt.Println()

	fmt.Fprintln(Output, BlackString("black"))
	fmt.Fprintln(Output, RedString("red"))
	fmt.Fprintln(Output, GreenString("green"))
	fmt.Fprintln(Output, YellowString("yellow"))
	fmt.Fprintln(Output, BlueString("blue"))
	fmt.Fprintln(Output, MagentaString("magenta"))
	fmt.Fprintln(Output, CyanString("cyan"))
	fmt.Fprintln(Output, WhiteString("white"))
	fmt.Fprintln(Output, HiBlackString("hblack"))
	fmt.Fprintln(Output, HiRedString("hred"))
	fmt.Fprintln(Output, HiGreenString("hgreen"))
	fmt.Fprintln(Output, HiYellowString("hyellow"))
	fmt.Fprintln(Output, HiBlueString("hblue"))
	fmt.Fprintln(Output, HiMagentaString("hmagenta"))
	fmt.Fprintln(Output, HiCyanString("hcyan"))
	fmt.Fprintln(Output, HiWhiteString("hwhite"))
}

func TestNoFormat(t *testing.T) {
	fmt.Printf("%s   %%s = ", BlackString("Black"))
	Black("%s")

	fmt.Printf("%s     %%s = ", RedString("Red"))
	Red("%s")

	fmt.Printf("%s   %%s = ", GreenString("Green"))
	Green("%s")

	fmt.Printf("%s  %%s = ", YellowString("Yellow"))
	Yellow("%s")

	fmt.Printf("%s    %%s = ", BlueString("Blue"))
	Blue("%s")

	fmt.Printf("%s %%s = ", MagentaString("Magenta"))
	Magenta("%s")

	fmt.Printf("%s    %%s = ", CyanString("Cyan"))
	Cyan("%s")

	fmt.Printf("%s   %%s = ", WhiteString("White"))
	White("%s")

	fmt.Printf("%s   %%s = ", HiBlackString("HiBlack"))
	HiBlack("%s")

	fmt.Printf("%s     %%s = ", HiRedString("HiRed"))
	HiRed("%s")

	fmt.Printf("%s   %%s = ", HiGreenString("HiGreen"))
	HiGreen("%s")

	fmt.Printf("%s  %%s = ", HiYellowString("HiYellow"))
	HiYellow("%s")

	fmt.Printf("%s    %%s = ", HiBlueString("HiBlue"))
	HiBlue("%s")

	fmt.Printf("%s %%s = ", HiMagentaString("HiMagenta"))
	HiMagenta("%s")

	fmt.Printf("%s    %%s = ", HiCyanString("HiCyan"))
	HiCyan("%s")

	fmt.Printf("%s   %%s = ", HiWhiteString("HiWhite"))
	HiWhite("%s")
}

func TestNoFormatString(t *testing.T) {
	tests := []struct {
		f      func(string, ...interface{}) string
		format string
		args   []interface{}
		want   string
	}{
		{BlackString, "%s", nil, "\x1b[30m%s\x1b[0m"},
		{RedString, "%s", nil, "\x1b[31m%s\x1b[0m"},
		{GreenString, "%s", nil, "\x1b[32m%s\x1b[0m"},
		{YellowString, "%s", nil, "\x1b[33m%s\x1b[0m"},
		{BlueString, "%s", nil, "\x1b[34m%s\x1b[0m"},
		{MagentaString, "%s", nil, "\x1b[35m%s\x1b[0m"},
		{CyanString, "%s", nil, "\x1b[36m%s\x1b[0m"},
		{WhiteString, "%s", nil, "\x1b[37m%s\x1b[0m"},
		{HiBlackString, "%s", nil, "\x1b[90m%s\x1b[0m"},
		{HiRedString, "%s", nil, "\x1b[91m%s\x1b[0m"},
		{HiGreenString, "%s", nil, "\x1b[92m%s\x1b[0m"},
		{HiYellowString, "%s", nil, "\x1b[93m%s\x1b[0m"},
		{HiBlueString, "%s", nil, "\x1b[94m%s\x1b[0m"},
		{HiMagentaString, "%s", nil, "\x1b[95m%s\x1b[0m"},
		{HiCyanString, "%s", nil, "\x1b[96m%s\x1b[0m"},
		{HiWhiteString, "%s", nil, "\x1b[97m%s\x1b[0m"},
	}

	for i, test := range tests {
		s := test.f(test.format, test.args...)

		if s != test.want {
			t.Errorf("[%d] want: %q, got: %q", i, test.want, s)
		}
	}
}

func TestColor_Println_Newline(t *testing.T) {
	rb := new(bytes.Buffer)
	Output = rb

	c := New(FgRed)
	c.Println("foo")

	got := readRaw(t, rb)
	want := "\x1b[31mfoo\x1b[0m\n"

	if want != got {
		t.Errorf("Println newline error\n\nwant: %q\n got: %q", want, got)
	}
}

func TestColor_Sprintln_Newline(t *testing.T) {
	c := New(FgRed)

	got := c.Sprintln("foo")
	want := "\x1b[31mfoo\x1b[0m\n"

	if want != got {
		t.Errorf("Println newline error\n\nwant: %q\n got: %q", want, got)
	}
}

func TestColor_Fprintln_Newline(t *testing.T) {
	rb := new(bytes.Buffer)
	c := New(FgRed)
	c.Fprintln(rb, "foo")

	got := readRaw(t, rb)
	want := "\x1b[31mfoo\x1b[0m\n"

	if want != got {
		t.Errorf("Println newline error\n\nwant: %q\n got: %q", want, got)
	}
}

func readRaw(t *testing.T, r io.Reader) string {
	t.Helper()

	out, err := io.ReadAll(r)
	if err != nil {
		t.Fatal(err)
	}

	return string(out)
}

func TestIssue206_1(t *testing.T) {
	//visual test, go test -v .
	//to  see the string with escape codes, use go test -v . > c:\temp\test.txt
	var underline = New(Underline).Sprint

	var line = fmt.Sprintf("%s %s %s %s", "word1", underline("word2"), "word3", underline("word4"))

	line = CyanString(line)

	fmt.Println(line)

	var result = fmt.Sprintf("%v", line)
	const expectedResult = "\x1b[36mword1 \x1b[4mword2\x1b[24m word3 \x1b[4mword4\x1b[24m\x1b[0m"

	if !bytes.Equal([]byte(result), []byte(expectedResult)) {
		t.Errorf("Expecting %v, got '%v'\n", expectedResult, result)
	}
}

func TestIssue206_2(t *testing.T) {
	var underline = New(Underline).Sprint
	var bold = New(Bold).Sprint

	var line = fmt.Sprintf("%s %s", GreenString(underline("underlined regular green")), RedString(bold("bold red")))

	fmt.Println(line)

	var result = fmt.Sprintf("%v", line)
	const expectedResult = "\x1b[32m\x1b[4munderlined regular green\x1b[24m\x1b[0m \x1b[31m\x1b[1mbold red\x1b[22m\x1b[0m"

	if !bytes.Equal([]byte(result), []byte(expectedResult)) {
		t.Errorf("Expecting %v, got '%v'\n", expectedResult, result)
	}
}

func TestIssue218(t *testing.T) {
	// Adds a newline to the end of the last string to make sure it isn't trimmed.
	params := []interface{}{"word1", "word2", "word3", "word4\n"}

	c := New(FgCyan)
	c.Println(params...)

	var result = c.Sprintln(params...)
	fmt.Println(params...)
	fmt.Print(result)

	const expectedResult = "\x1b[36mword1 word2 word3 word4\n\x1b[0m\n"

	if !bytes.Equal([]byte(result), []byte(expectedResult)) {
		t.Errorf("Sprintln: Expecting %v (%v), got '%v (%v)'\n", expectedResult, []byte(expectedResult), result, []byte(result))
	}

	fn := c.SprintlnFunc()
	result = fn(params...)
	if !bytes.Equal([]byte(result), []byte(expectedResult)) {
		t.Errorf("SprintlnFunc: Expecting %v (%v), got '%v (%v)'\n", expectedResult, []byte(expectedResult), result, []byte(result))
	}

	var buf bytes.Buffer
	c.Fprintln(&buf, params...)
	result = buf.String()
	if !bytes.Equal([]byte(result), []byte(expectedResult)) {
		t.Errorf("Fprintln: Expecting %v (%v), got '%v (%v)'\n", expectedResult, []byte(expectedResult), result, []byte(result))
	}
}
