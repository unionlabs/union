package main

import (
	"bytes"
	"errors"
	"flag"
	"fmt"
	"go/ast"
	"go/format"
	"go/token"
	"log"
	"os"
	"path"
	"path/filepath"
	"strings"

	"golang.org/x/tools/go/packages"
	"golang.org/x/tools/imports"
)

type options struct {
	pkgs             []string
	dryRun           bool
	debug            bool
	cmpImportName    string
	showLoaderErrors bool
	buildFlags       []string
	localImportPath  string
}

func main() {
	name := os.Args[0]
	flags, opts := setupFlags(name)
	handleExitError(name, flags.Parse(os.Args[1:]))
	setupLogging(opts)
	opts.pkgs = flags.Args()
	handleExitError(name, run(*opts))
}

func setupLogging(opts *options) {
	log.SetFlags(0)
	enableDebug = opts.debug
}

var enableDebug = false

func debugf(msg string, args ...interface{}) {
	if enableDebug {
		log.Printf("DEBUG: "+msg, args...)
	}
}

func setupFlags(name string) (*flag.FlagSet, *options) {
	opts := options{}
	flags := flag.NewFlagSet(name, flag.ContinueOnError)
	flags.BoolVar(&opts.dryRun, "dry-run", false,
		"don't write changes to file")
	flags.BoolVar(&opts.debug, "debug", false, "enable debug logging")
	flags.StringVar(&opts.cmpImportName, "cmp-pkg-import-alias", "is",
		"import alias to use for the assert/cmp package")
	flags.BoolVar(&opts.showLoaderErrors, "print-loader-errors", false,
		"print errors from loading source")
	flags.Var((*stringSliceValue)(&opts.buildFlags), "build-flags",
		"build flags to pass to Go when loading source files")
	flags.StringVar(&opts.localImportPath, "local-import-path", "",
		"value to pass to 'goimports -local' flag for sorting local imports")
	flags.Usage = func() {
		fmt.Fprintf(os.Stderr, `Usage: %s [OPTIONS] PACKAGE [PACKAGE...]

Migrate calls from testify/{assert|require} to gotest.tools/v3/assert.

`, name)
		flags.PrintDefaults()
	}
	return flags, &opts
}

func handleExitError(name string, err error) {
	switch {
	case err == nil:
		return
	case errors.Is(err, flag.ErrHelp):
		os.Exit(0)
	default:
		log.Println(name + ": Error: " + err.Error())
		os.Exit(3)
	}
}

func run(opts options) error {
	imports.LocalPrefix = opts.localImportPath

	fset := token.NewFileSet()
	pkgs, err := loadPackages(opts, fset)
	if err != nil {
		return fmt.Errorf("failed to load program: %w", err)
	}

	debugf("package count: %d", len(pkgs))
	for _, pkg := range pkgs {
		debugf("file count for package %v: %d", pkg.PkgPath, len(pkg.Syntax))
		for _, astFile := range pkg.Syntax {
			absFilename := fset.File(astFile.Pos()).Name()
			filename := relativePath(absFilename)
			importNames := newImportNames(astFile.Imports, opts)
			if !importNames.hasTestifyImports() {
				debugf("skipping file %s, no imports", filename)
				continue
			}

			debugf("migrating %s with imports: %#v", filename, importNames)
			m := migration{
				file:        astFile,
				fileset:     fset,
				importNames: importNames,
				pkgInfo:     pkg.TypesInfo,
			}
			migrateFile(m)
			if opts.dryRun {
				continue
			}

			raw, err := formatFile(m)
			if err != nil {
				return fmt.Errorf("failed to format %s: %w", filename, err)
			}

			if err := os.WriteFile(absFilename, raw, 0); err != nil {
				return fmt.Errorf("failed to write file %s: %w", filename, err)
			}
		}
	}

	return nil
}

var loadMode = packages.NeedName |
	packages.NeedFiles |
	packages.NeedCompiledGoFiles |
	packages.NeedDeps |
	packages.NeedImports |
	packages.NeedTypes |
	packages.NeedTypesInfo |
	packages.NeedTypesSizes |
	packages.NeedSyntax

func loadPackages(opts options, fset *token.FileSet) ([]*packages.Package, error) {
	conf := &packages.Config{
		Mode:       loadMode,
		Fset:       fset,
		Tests:      true,
		Logf:       debugf,
		BuildFlags: opts.buildFlags,
	}

	pkgs, err := packages.Load(conf, opts.pkgs...)
	if err != nil {
		return nil, err
	}
	if opts.showLoaderErrors {
		packages.PrintErrors(pkgs)
	}
	return pkgs, nil
}

func relativePath(p string) string {
	cwd, err := os.Getwd()
	if err != nil {
		return p
	}
	rel, err := filepath.Rel(cwd, p)
	if err != nil {
		return p
	}
	return rel
}

type importNames struct {
	testifyAssert  string
	testifyRequire string
	assert         string
	cmp            string
}

func (p importNames) hasTestifyImports() bool {
	return p.testifyAssert != "" || p.testifyRequire != ""
}

func (p importNames) matchesTestify(ident *ast.Ident) bool {
	return ident.Name == p.testifyAssert || ident.Name == p.testifyRequire
}

func (p importNames) funcNameFromTestifyName(name string) string {
	switch name {
	case p.testifyAssert:
		return funcNameCheck
	case p.testifyRequire:
		return funcNameAssert
	default:
		panic("unexpected testify import name " + name)
	}
}

func newImportNames(imports []*ast.ImportSpec, opt options) importNames {
	defaultAssertAlias := path.Base(pkgAssert)
	importNames := importNames{
		assert: defaultAssertAlias,
		cmp:    path.Base(pkgCmp),
	}
	for _, spec := range imports {
		switch strings.Trim(spec.Path.Value, `"`) {
		case pkgTestifyAssert, pkgGopkgTestifyAssert:
			importNames.testifyAssert = identOrDefault(spec.Name, "assert")
		case pkgTestifyRequire, pkgGopkgTestifyRequire:
			importNames.testifyRequire = identOrDefault(spec.Name, "require")
		default:
			pkgPath := strings.Trim(spec.Path.Value, `"`)

			switch {
			// v3/assert is already imported and has an alias
			case pkgPath == pkgAssert:
				if spec.Name != nil && spec.Name.Name != "" {
					importNames.assert = spec.Name.Name
				}
				continue

			// some other package is imported as assert
			case importedAs(spec, path.Base(pkgAssert)) && importNames.assert == defaultAssertAlias:
				importNames.assert = "gtyassert"
			}
		}
	}

	if opt.cmpImportName != "" {
		importNames.cmp = opt.cmpImportName
	}
	return importNames
}

func importedAs(spec *ast.ImportSpec, pkg string) bool {
	if path.Base(strings.Trim(spec.Path.Value, `"`)) == pkg {
		return true
	}
	return spec.Name != nil && spec.Name.Name == pkg
}

func identOrDefault(ident *ast.Ident, def string) string {
	if ident != nil {
		return ident.Name
	}
	return def
}

func formatFile(migration migration) ([]byte, error) {
	buf := new(bytes.Buffer)
	err := format.Node(buf, migration.fileset, migration.file)
	if err != nil {
		return nil, err
	}
	filename := migration.fileset.File(migration.file.Pos()).Name()
	return imports.Process(filename, buf.Bytes(), nil)
}
