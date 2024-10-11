// Package metavars allows manipulation of "meta variables" stored as variable
// declarations in Go files.
package metavars

import (
	"bytes"
	"fmt"
	"go/ast"
	"go/format"
	"go/parser"
	"go/token"
	"io"
	"os"
	"strconv"
	"strings"

	"github.com/mmcloughlin/addchain/internal/errutil"
	"github.com/mmcloughlin/addchain/internal/print"
)

// Property is a string variable.
type Property struct {
	Name  string
	Doc   string
	Value string
}

// File of property definitions.
type File struct {
	Package    string
	Properties []Property
}

// Get property with given name.
func (f *File) Get(name string) (string, bool) {
	if p := f.get(name); p != nil {
		return p.Value, true
	}
	return "", false
}

// Add property to the file, which must not already exist.
func (f *File) Add(p Property) error {
	if q := f.get(p.Name); q != nil {
		return fmt.Errorf("property %q already exists", p.Name)
	}
	f.Properties = append(f.Properties, p)
	return nil
}

// Set property name to value. The property must exist.
func (f *File) Set(name, value string) error {
	if p := f.get(name); p != nil {
		p.Value = value
		return nil
	}
	return fmt.Errorf("unknown property %q", name)
}

func (f *File) get(name string) *Property {
	for i := range f.Properties {
		p := &f.Properties[i]
		if p.Name == name {
			return p
		}
	}
	return nil
}

// Write file f to write w.
func Write(w io.Writer, f *File) error {
	// Write source code.
	buf := bytes.NewBuffer(nil)
	p := print.New(buf)

	p.Linef("package %s", f.Package)
	p.NL()
	p.Linef("var (")
	p.Indent()
	for _, prop := range f.Properties {
		if prop.Doc != "" {
			p.Linef("// %s", prop.Doc)
		}
		p.Linef("%s = %q", prop.Name, prop.Value)
	}
	p.Dedent()
	p.Linef(")")

	if err := p.Error(); err != nil {
		return err
	}

	// Format and write.
	src, err := format.Source(buf.Bytes())
	if err != nil {
		return err
	}

	_, err = w.Write(src)
	return err
}

// WriteFile writes f to the given file.
func WriteFile(filename string, f *File) error {
	return writefile(filename, f)
}

func writefile(filename string, f *File) (err error) {
	w, err := os.Create(filename)
	if err != nil {
		return err
	}
	defer errutil.CheckClose(&err, w)

	return Write(w, f)
}

// Read properties from the given reader.
func Read(r io.Reader) (*File, error) {
	return parse("", r)
}

// ReadFile reads properties file.
func ReadFile(filename string) (*File, error) {
	return parse(filename, nil)
}

func parse(filename string, src interface{}) (*File, error) {
	// Parse AST.
	fset := token.NewFileSet() // positions are relative to fset
	f, err := parser.ParseFile(fset, filename, src, parser.ParseComments)
	if err != nil {
		return nil, err
	}

	// Build file from AST.
	b := &builder{f: &File{}}
	b.file(f)

	if b.err != nil {
		return nil, b.err
	}

	return b.f, nil
}

type builder struct {
	f   *File
	err error
}

func (b *builder) file(f *ast.File) {
	// Package name.
	b.f.Package = f.Name.Name

	// Declarations.
	for _, d := range f.Decls {
		b.decl(d)
	}
}

func (b *builder) decl(i ast.Decl) {
	switch d := i.(type) {
	case *ast.GenDecl:
		b.gendecl(d)
	default:
		b.seterrorf("unexpected declaration type %T", i)
	}
}

func (b *builder) gendecl(g *ast.GenDecl) {
	// Confirm this is a variable declaration.
	if g.Tok != token.VAR {
		b.seterrorf("expected variable declaration, got %s", g.Tok)
		return
	}

	// Process each spec.
	for _, spec := range g.Specs {
		v, ok := spec.(*ast.ValueSpec)
		if !ok {
			b.seterrorf("expected value spec")
		}
		b.valuespec(v)
	}
}

func (b *builder) valuespec(v *ast.ValueSpec) {
	p := Property{}

	// Extract name.
	if len(v.Names) != 1 {
		b.seterrorf("expected one name, got %d", len(v.Names))
		return
	}
	p.Name = v.Names[0].Name

	// Extract value.
	if len(v.Values) != 1 {
		b.seterrorf("expected one value, got %d", len(v.Values))
		return
	}

	expr := v.Values[0]
	lit, ok := expr.(*ast.BasicLit)
	if !ok {
		b.seterrorf("expected basic literal")
		return
	}

	if lit.Kind != token.STRING {
		b.seterrorf("expected string literal")
		return
	}

	s, err := strconv.Unquote(lit.Value)
	if err != nil {
		b.seterrorf("parse string literal: %w", err)
		return
	}

	p.Value = s

	// Optional documentation.
	if v.Doc != nil {
		const leader = "// "
		if len(v.Doc.List) != 1 {
			b.seterrorf("expect single line documentation")
			return
		}
		comment := v.Doc.List[0]
		if !strings.HasPrefix(comment.Text, leader) {
			b.seterrorf("expected documentation to have %q leader", leader)
			return
		}
		p.Doc = strings.TrimPrefix(comment.Text, leader)
	}

	// Add property to file.
	b.f.Properties = append(b.f.Properties, p)
}

func (b *builder) seterrorf(format string, args ...interface{}) {
	if b.err == nil {
		b.err = fmt.Errorf(format, args...)
	}
}
