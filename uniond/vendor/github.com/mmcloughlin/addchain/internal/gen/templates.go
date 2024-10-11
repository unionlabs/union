package gen

import (
	"embed"
	"fmt"
	"path"
	"strings"
)

//go:embed templates
var templates embed.FS

// Template file extension.
const templateext = ".tmpl"

// BuiltinTemplate loads the named template. Returns an error if the template is
// unknown.
func BuiltinTemplate(name string) (string, error) {
	path := fmt.Sprintf("templates/%s%s", name, templateext)
	b, err := templates.ReadFile(path)
	if err != nil {
		return "", fmt.Errorf("unknown template %q", name)
	}
	return string(b), nil
}

// BuiltinTemplateNames returns all builtin template names.
func BuiltinTemplateNames() []string {
	entries, err := templates.ReadDir("templates")
	if err != nil {
		panic("gen: could not read embedded templates")
	}
	var names []string
	for _, entry := range entries {
		filename := entry.Name()
		if path.Ext(filename) != templateext {
			panic("gen: builtin template has wrong extension")
		}
		name := strings.TrimSuffix(filename, templateext)
		names = append(names, name)
	}
	return names
}

// IsBuiltinTemplate reports whether name is a builtin template name.
func IsBuiltinTemplate(name string) bool {
	for _, builtin := range BuiltinTemplateNames() {
		if builtin == name {
			return true
		}
	}
	return false
}
