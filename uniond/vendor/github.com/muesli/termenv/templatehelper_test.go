package termenv

import (
	"bytes"
	"fmt"
	"io/ioutil"
	"testing"
	"text/template"
)

func TestTemplateFuncs(t *testing.T) {
	tests := []struct {
		name    string
		profile Profile
	}{
		{"ascii", Ascii},
		{"ansi", ANSI},
		{"ansi256", ANSI256},
		{"truecolor", TrueColor},
	}
	const templateFile = "./testdata/templatehelper.tpl"
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			tpl, err := template.New("templatehelper.tpl").Funcs(TemplateFuncs(test.profile)).ParseFiles(templateFile)
			if err != nil {
				t.Fatalf("unexpected error parsing template: %v", err)
			}
			var buf bytes.Buffer
			if err = tpl.Execute(&buf, nil); err != nil {
				t.Fatalf("unexpected error executing template: %v", err)
			}
			actual := buf.Bytes()
			filename := fmt.Sprintf("./testdata/templatehelper_%s.txt", test.name)
			expected, err := ioutil.ReadFile(filename)
			if err != nil {
				t.Fatalf("unexpected error reading golden file %q: %v", filename, err)
			}
			if !bytes.Equal(buf.Bytes(), expected) {
				t.Fatalf("template output does not match golden file.\n--- Expected ---\n%s\n--- Actual ---\n%s\n", string(expected), string(actual))
			}
		})
	}
}
