package promptui

import (
	"bytes"
	"testing"

	"github.com/manifoldco/promptui/screenbuf"
)

func TestSelectTemplateRender(t *testing.T) {
	t.Run("when using default style", func(t *testing.T) {
		values := []string{"Zero"}
		s := Select{
			Label: "Select Number",
			Items: values,
		}
		err := s.prepareTemplates()
		if err != nil {
			t.Fatalf("Unexpected error preparing templates %v", err)
		}

		result := string(render(s.Templates.label, s.Label))
		exp := "\x1b[34m?\x1b[0m Select Number: "
		if result != exp {
			t.Errorf("Expected label to eq %q, got %q", exp, result)
		}

		result = string(render(s.Templates.active, values[0]))
		exp = "\x1b[1m▸\x1b[0m \x1b[4mZero\x1b[0m"
		if result != exp {
			t.Errorf("Expected active item to eq %q, got %q", exp, result)
		}

		result = string(render(s.Templates.inactive, values[0]))
		exp = "  Zero"
		if result != exp {
			t.Errorf("Expected inactive item to eq %q, got %q", exp, result)
		}

		result = string(render(s.Templates.selected, values[0]))
		exp = "\x1b[32m\x1b[32m✔\x1b[0m \x1b[2mZero\x1b[0m"
		if result != exp {
			t.Errorf("Expected selected item to eq %q, got %q", exp, result)
		}
	})

	t.Run("when using custom style", func(t *testing.T) {
		type pepper struct {
			Name        string
			HeatUnit    int
			Peppers     int
			Description string
		}
		peppers := []pepper{
			{
				Name:        "Bell Pepper",
				HeatUnit:    0,
				Peppers:     1,
				Description: "Not very spicy!",
			},
		}

		templates := &SelectTemplates{
			Label:    "{{ . }}?",
			Active:   "\U0001F525 {{ .Name | bold }} ({{ .HeatUnit | red | italic }})",
			Inactive: "   {{ .Name | bold }} ({{ .HeatUnit | red | italic }})",
			Selected: "\U0001F525 {{ .Name | red | bold }}",
			Details: `Name: {{.Name}}
Peppers: {{.Peppers}}
Description: {{.Description}}`,
		}

		s := Select{
			Label:     "Spicy Level",
			Items:     peppers,
			Templates: templates,
		}

		err := s.prepareTemplates()
		if err != nil {
			t.Fatalf("Unexpected error preparing templates %v", err)
		}

		result := string(render(s.Templates.label, s.Label))
		exp := "Spicy Level?"
		if result != exp {
			t.Errorf("Expected label to eq %q, got %q", exp, result)
		}

		result = string(render(s.Templates.active, peppers[0]))
		exp = "🔥 \x1b[1mBell Pepper\x1b[0m (\x1b[3m\x1b[31m0\x1b[0m)"
		if result != exp {
			t.Errorf("Expected active item to eq %q, got %q", exp, result)
		}

		result = string(render(s.Templates.inactive, peppers[0]))
		exp = "   \x1b[1mBell Pepper\x1b[0m (\x1b[3m\x1b[31m0\x1b[0m)"
		if result != exp {
			t.Errorf("Expected inactive item to eq %q, got %q", exp, result)
		}

		result = string(render(s.Templates.selected, peppers[0]))
		exp = "🔥 \x1b[1m\x1b[31mBell Pepper\x1b[0m"
		if result != exp {
			t.Errorf("Expected selected item to eq %q, got %q", exp, result)
		}

		result = string(render(s.Templates.details, peppers[0]))
		exp = "Name: Bell Pepper\nPeppers: 1\nDescription: Not very spicy!"
		if result != exp {
			t.Errorf("Expected selected item to eq %q, got %q", exp, result)
		}
	})

	t.Run("when a template is invalid", func(t *testing.T) {
		templates := &SelectTemplates{
			Label: "{{ . ",
		}

		s := Select{
			Label:     "Spicy Level",
			Templates: templates,
		}

		err := s.prepareTemplates()
		if err == nil {
			t.Fatalf("Expected error got none")
		}
	})

	t.Run("when a template render fails", func(t *testing.T) {
		templates := &SelectTemplates{
			Label: "{{ .InvalidName }}",
		}

		s := Select{
			Label:     struct{ Name string }{Name: "Pepper"},
			Items:     []string{},
			Templates: templates,
		}

		err := s.prepareTemplates()
		if err != nil {
			t.Fatalf("Unexpected error preparing templates %v", err)
		}

		result := string(render(s.Templates.label, s.Label))
		exp := "{Pepper}"
		if result != exp {
			t.Errorf("Expected label to eq %q, got %q", exp, result)
		}
	})
}

func TestClearScreen(t *testing.T) {
	var buf bytes.Buffer
	sb := screenbuf.New(&buf)

	sb.WriteString("test")
	clearScreen(sb)

	got := buf.String()
	except := "\x1b[1A\x1b[2K\r"

	if except != got {
		t.Errorf("expected %q, got %q", except, got)
	}
}
