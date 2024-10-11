// Command apidiff determines whether two versions of a package are compatible
package main

import (
	"bufio"
	"flag"
	"fmt"
	"go/token"
	"go/types"
	"os"
	"strings"

	"golang.org/x/exp/apidiff"
	"golang.org/x/tools/go/gcexportdata"
	"golang.org/x/tools/go/packages"
)

var (
	exportDataOutfile = flag.String("w", "", "file for export data")
	incompatibleOnly  = flag.Bool("incompatible", false, "display only incompatible changes")
	allowInternal     = flag.Bool("allow-internal", false, "allow apidiff to compare internal packages")
	moduleMode        = flag.Bool("m", false, "compare modules instead of packages")
)

func main() {
	flag.Usage = func() {
		w := flag.CommandLine.Output()
		fmt.Fprintf(w, "usage:\n")
		fmt.Fprintf(w, "apidiff OLD NEW\n")
		fmt.Fprintf(w, "   compares OLD and NEW package APIs\n")
		fmt.Fprintf(w, "   where OLD and NEW are either import paths or files of export data\n")
		fmt.Fprintf(w, "apidiff -m OLD NEW\n")
		fmt.Fprintf(w, "   compares OLD and NEW module APIs\n")
		fmt.Fprintf(w, "   where OLD and NEW are module paths\n")
		fmt.Fprintf(w, "apidiff -w FILE IMPORT_PATH\n")
		fmt.Fprintf(w, "   writes export data of the package at IMPORT_PATH to FILE\n")
		fmt.Fprintf(w, "   NOTE: In a GOPATH-less environment, this option consults the\n")
		fmt.Fprintf(w, "   module cache by default, unless used in the directory that\n")
		fmt.Fprintf(w, "   contains the go.mod module definition that IMPORT_PATH belongs\n")
		fmt.Fprintf(w, "   to. In most cases users want the latter behavior, so be sure\n")
		fmt.Fprintf(w, "   to cd to the exact directory which contains the module\n")
		fmt.Fprintf(w, "   definition of IMPORT_PATH.\n")
		fmt.Fprintf(w, "apidiff -m -w FILE MODULE_PATH\n")
		fmt.Fprintf(w, "   writes export data of the module at MODULE_PATH to FILE\n")
		fmt.Fprintf(w, "   Same NOTE for packages applies to modules.\n")
		flag.PrintDefaults()
	}

	flag.Parse()
	if *exportDataOutfile != "" {
		if len(flag.Args()) != 1 {
			flag.Usage()
			os.Exit(2)
		}
		if err := loadAndWrite(flag.Arg(0)); err != nil {
			die("writing export data: %v", err)
		}
		os.Exit(0)
	}

	if len(flag.Args()) != 2 {
		flag.Usage()
		os.Exit(2)
	}

	var report apidiff.Report
	if *moduleMode {
		oldmod := mustLoadOrReadModule(flag.Arg(0))
		newmod := mustLoadOrReadModule(flag.Arg(1))

		report = apidiff.ModuleChanges(oldmod, newmod)
	} else {
		oldpkg := mustLoadOrReadPackage(flag.Arg(0))
		newpkg := mustLoadOrReadPackage(flag.Arg(1))
		if !*allowInternal {
			if isInternalPackage(oldpkg.Path(), "") && isInternalPackage(newpkg.Path(), "") {
				fmt.Fprintf(os.Stderr, "Ignoring internal package %s\n", oldpkg.Path())
				os.Exit(0)
			}
		}
		report = apidiff.Changes(oldpkg, newpkg)
	}

	var err error
	if *incompatibleOnly {
		err = report.TextIncompatible(os.Stdout, false)
	} else {
		err = report.Text(os.Stdout)
	}
	if err != nil {
		die("writing report: %v", err)
	}
}

func loadAndWrite(path string) error {
	if *moduleMode {
		module := mustLoadModule(path)
		return writeModuleExportData(module, *exportDataOutfile)
	}

	// Loading and writing data for only a single package.
	pkg := mustLoadPackage(path)
	return writePackageExportData(pkg, *exportDataOutfile)
}

func mustLoadOrReadPackage(importPathOrFile string) *types.Package {
	fileInfo, err := os.Stat(importPathOrFile)
	if err == nil && fileInfo.Mode().IsRegular() {
		pkg, err := readPackageExportData(importPathOrFile)
		if err != nil {
			die("reading export data from %s: %v", importPathOrFile, err)
		}
		return pkg
	} else {
		return mustLoadPackage(importPathOrFile).Types
	}
}

func mustLoadPackage(importPath string) *packages.Package {
	pkg, err := loadPackage(importPath)
	if err != nil {
		die("loading %s: %v", importPath, err)
	}
	return pkg
}

func loadPackage(importPath string) (*packages.Package, error) {
	cfg := &packages.Config{Mode: packages.LoadTypes |
		packages.NeedName | packages.NeedTypes | packages.NeedImports | packages.NeedDeps,
	}
	pkgs, err := packages.Load(cfg, importPath)
	if err != nil {
		return nil, err
	}
	if len(pkgs) == 0 {
		return nil, fmt.Errorf("found no packages for import %s", importPath)
	}
	if len(pkgs[0].Errors) > 0 {
		// TODO: use errors.Join once Go 1.21 is released.
		return nil, pkgs[0].Errors[0]
	}
	return pkgs[0], nil
}

func mustLoadOrReadModule(modulePathOrFile string) *apidiff.Module {
	var module *apidiff.Module
	fileInfo, err := os.Stat(modulePathOrFile)
	if err == nil && fileInfo.Mode().IsRegular() {
		module, err = readModuleExportData(modulePathOrFile)
		if err != nil {
			die("reading export data from %s: %v", modulePathOrFile, err)
		}
	} else {
		module = mustLoadModule(modulePathOrFile)
	}

	filterInternal(module, *allowInternal)

	return module
}

func mustLoadModule(modulepath string) *apidiff.Module {
	module, err := loadModule(modulepath)
	if err != nil {
		die("loading %s: %v", modulepath, err)
	}
	return module
}

func loadModule(modulepath string) (*apidiff.Module, error) {
	cfg := &packages.Config{Mode: packages.LoadTypes |
		packages.NeedName | packages.NeedTypes | packages.NeedImports | packages.NeedDeps | packages.NeedModule,
	}
	loaded, err := packages.Load(cfg, fmt.Sprintf("%s/...", modulepath))
	if err != nil {
		return nil, err
	}
	if len(loaded) == 0 {
		return nil, fmt.Errorf("found no packages for module %s", modulepath)
	}
	var tpkgs []*types.Package
	for _, p := range loaded {
		if len(p.Errors) > 0 {
			// TODO: use errors.Join once Go 1.21 is released.
			return nil, p.Errors[0]
		}
		tpkgs = append(tpkgs, p.Types)
	}

	return &apidiff.Module{Path: loaded[0].Module.Path, Packages: tpkgs}, nil
}

func readModuleExportData(filename string) (*apidiff.Module, error) {
	f, err := os.Open(filename)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	r := bufio.NewReader(f)
	modPath, err := r.ReadString('\n')
	if err != nil {
		return nil, err
	}
	modPath = modPath[:len(modPath)-1] // remove delimiter
	m := map[string]*types.Package{}
	pkgs, err := gcexportdata.ReadBundle(r, token.NewFileSet(), m)
	if err != nil {
		return nil, err
	}

	return &apidiff.Module{Path: modPath, Packages: pkgs}, nil
}

func writeModuleExportData(module *apidiff.Module, filename string) error {
	f, err := os.Create(filename)
	if err != nil {
		return err
	}
	fmt.Fprintln(f, module.Path)
	// TODO: Determine if token.NewFileSet is appropriate here.
	if err := gcexportdata.WriteBundle(f, token.NewFileSet(), module.Packages); err != nil {
		return err
	}
	return f.Close()
}

func readPackageExportData(filename string) (*types.Package, error) {
	f, err := os.Open(filename)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	r := bufio.NewReader(f)
	m := map[string]*types.Package{}
	pkgPath, err := r.ReadString('\n')
	if err != nil {
		return nil, err
	}
	pkgPath = pkgPath[:len(pkgPath)-1] // remove delimiter
	return gcexportdata.Read(r, token.NewFileSet(), m, pkgPath)
}

func writePackageExportData(pkg *packages.Package, filename string) error {
	f, err := os.Create(filename)
	if err != nil {
		return err
	}
	// Include the package path in the file. The exportdata format does
	// not record the path of the package being written.
	fmt.Fprintln(f, pkg.PkgPath)
	err1 := gcexportdata.Write(f, pkg.Fset, pkg.Types)
	err2 := f.Close()
	if err1 != nil {
		return err1
	}
	return err2
}

func die(format string, args ...interface{}) {
	fmt.Fprintf(os.Stderr, format+"\n", args...)
	os.Exit(1)
}

func filterInternal(m *apidiff.Module, allow bool) {
	if allow {
		return
	}

	var nonInternal []*types.Package
	for _, p := range m.Packages {
		if !isInternalPackage(p.Path(), m.Path) {
			nonInternal = append(nonInternal, p)
		} else {
			fmt.Fprintf(os.Stderr, "Ignoring internal package %s\n", p.Path())
		}
	}
	m.Packages = nonInternal
}

func isInternalPackage(pkgPath, modulePath string) bool {
	pkgPath = strings.TrimPrefix(pkgPath, modulePath)
	switch {
	case strings.HasSuffix(pkgPath, "/internal"):
		return true
	case strings.Contains(pkgPath, "/internal/"):
		return true
	case pkgPath == "internal":
		return true
	case strings.HasPrefix(pkgPath, "internal/"):
		return true
	}
	return false
}
