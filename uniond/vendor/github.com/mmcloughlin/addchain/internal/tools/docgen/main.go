package main

import (
	"bufio"
	"bytes"
	"embed"
	"errors"
	"flag"
	"fmt"
	"io/ioutil"
	"log"
	"net/url"
	"os"
	"path"
	"path/filepath"
	"regexp"
	"strings"
	"text/template"

	"github.com/mmcloughlin/addchain/internal/gen"
	"github.com/mmcloughlin/addchain/internal/results"
	"github.com/mmcloughlin/addchain/meta"
)

//go:generate bib generate -bib ../../../doc/references.bib -tmpl bibliography.tmpl -output zbibliography.go

func main() {
	log.SetPrefix("docgen: ")
	log.SetFlags(0)
	if err := mainerr(); err != nil {
		log.Fatal(err)
	}
}

var (
	typ    = flag.String("type", "", "documentation type")
	tmpl   = flag.String("tmpl", "", "explicit template file (overrides -type)")
	output = flag.String("output", "", "path to output file (default stdout)")
	tocmin = flag.Int("tocmin", 2, "table of contents minimum heading level")
	tocmax = flag.Int("tocmax", 4, "table of contents maximum heading level")
)

func mainerr() (err error) {
	flag.Parse()

	// Initialize template.
	t := template.New("doc")

	t.Funcs(template.FuncMap{
		"link":     rellink(*output),
		"include":  include,
		"snippet":  snippet,
		"anchor":   anchor,
		"bibentry": bibentry,
		"biburl":   biburl,
		"toc":      toc,
		"oplus":    symbol('\u2295'),
		"otimes":   symbol('\u2297'),
		"sum":      symbol('\u2211'),
	})

	t.Funcs(pkgfuncs("", meta.Meta.Module()))
	t.Funcs(pkgfuncs("std", ""))

	// Load template.
	s, err := load()
	if err != nil {
		return err
	}

	if _, err := t.Parse(s); err != nil {
		return err
	}

	// Prepare template data.
	data := map[string]interface{}{
		"Meta":         meta.Meta,
		"Results":      results.Results,
		"Bibliography": bibliography,

		"BuiltinTemplateNames": gen.BuiltinTemplateNames(),
		"TemplateFunctions":    gen.Functions,
	}

	// Execute.
	var buf bytes.Buffer
	if err := t.Execute(&buf, data); err != nil {
		return err
	}

	// Insert table of contents.
	body, err := generateTOC(buf.Bytes(), *tocmin, *tocmax)
	if err != nil {
		return err
	}

	// Output.
	if *output != "" {
		err = ioutil.WriteFile(*output, body, 0640)
	} else {
		_, err = os.Stdout.Write(body)
	}

	if err != nil {
		return err
	}

	return nil
}

//go:embed templates
var templates embed.FS

// load template.
func load() (string, error) {
	// Prefer explicit filename, if provided.
	if *tmpl != "" {
		b, err := ioutil.ReadFile(*tmpl)
		if err != nil {
			return "", err
		}
		return string(b), nil
	}

	// Otherwise expect a named type.
	if *typ == "" {
		return "", errors.New("missing documentation type")
	}
	path := fmt.Sprintf("templates/%s.tmpl", *typ)
	b, err := templates.ReadFile(path)
	if err != nil {
		return "", fmt.Errorf("unknown documentation type %q", *typ)
	}

	return string(b), nil
}

// link to a file in the repository.
func rellink(output string) func(string) (string, error) {
	base := filepath.Dir(output)
	return func(filename string) (string, error) {
		if _, err := os.Stat(filename); err != nil {
			return "", err
		}
		return filepath.Rel(base, filename)
	}
}

// include template function.
func include(filename string) (string, error) {
	b, err := ioutil.ReadFile(filename)
	if err != nil {
		return "", err
	}
	return string(b), nil
}

// snippet of a file between start and end regular expressions.
func snippet(filename, start, end string) (string, error) {
	// Parse regular expressions.
	startx, err := regexp.Compile(start)
	if err != nil {
		return "", err
	}

	endx, err := regexp.Compile(end)
	if err != nil {
		return "", err
	}

	// Read the full file.
	data, err := include(filename)
	if err != nil {
		return "", err
	}

	// Collect matched lines.
	var buf bytes.Buffer
	output := false
	s := bufio.NewScanner(strings.NewReader(data))
	for s.Scan() {
		line := s.Text()
		if startx.MatchString(line) {
			output = true
		}
		if output {
			fmt.Fprintln(&buf, line)
		}
		if endx.MatchString(line) {
			output = false
		}
	}

	return buf.String(), nil
}

// anchor returns the anchor for a heading in Github.
func anchor(heading string) string {
	r := strings.NewReplacer(" ", "-", "(", "", ")", "", "/", "")
	return r.Replace(strings.ToLower((heading)))
}

// pkgfuncs builds template functions for references to packages linked to
// documentation.
func pkgfuncs(prefix, mod string) template.FuncMap {
	return template.FuncMap{
		// pkgurl returns url to package documentation.
		prefix + "pkgurl": func(pkg string) string {
			return pkgurl(mod, pkg, "")
		},
		// pkg returns markdown for a package with a link to documentation.
		prefix + "pkg": func(pkg string) string {
			return fmt.Sprintf("[`%s`](%s)", pkg, pkgurl(mod, pkg, ""))
		},
		// sym returns markdown for a symbol with a link to documentation.
		prefix + "sym": func(pkg, name string) string {
			return fmt.Sprintf("[`%s.%s`](%s)", path.Base(pkg), name, pkgurl(mod, pkg, name))
		},
	}
}

// pkgurl returns url to go.dev documentation for a given package.
func pkgurl(mod, pkg, fragment string) string {
	u := url.URL{
		Scheme:   "https",
		Host:     "pkg.go.dev",
		Path:     path.Join(mod, pkg),
		Fragment: fragment,
	}
	return u.String()
}

// bibentry returns formatted bibliography entry for the given citation name.
func bibentry(name string) (string, error) {
	for _, entry := range bibliography {
		if entry.CiteName == name {
			return entry.Formatted, nil
		}
	}
	return "", fmt.Errorf("unknown citation %q", name)
}

// biburl returns URL for the bibliography entry with the given citation name.
func biburl(name string) (string, error) {
	for _, entry := range bibliography {
		if entry.CiteName == name {
			return entry.URL, nil
		}
	}
	return "", fmt.Errorf("unknown citation %q", name)
}

// symbol builds a template function that expands to the given unicode symbol.
func symbol(r rune) func() string {
	return func() string {
		return string([]rune{r})
	}
}
