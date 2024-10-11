// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// gorelease is an experimental tool that helps module authors avoid common
// problems before releasing a new version of a module.
//
// Usage:
//
//	gorelease [-base={version|none}] [-version=version]
//
// Examples:
//
//	# Compare with the latest version and suggest a new version.
//	gorelease
//
//	# Compare with a specific version and suggest a new version.
//	gorelease -base=v1.2.3
//
//	# Compare with the latest version and check a specific new version for compatibility.
//	gorelease -version=v1.3.0
//
//	# Compare with a specific version and check a specific new version for compatibility.
//	gorelease -base=v1.2.3 -version=v1.3.0
//
// gorelease analyzes changes in the public API and dependencies of the main
// module. It compares a base version (set with -base) with the currently
// checked out revision. Given a proposed version to release (set with
// -version), gorelease reports whether the changes are consistent with
// semantic versioning. If no version is proposed with -version, gorelease
// suggests the lowest version consistent with semantic versioning.
//
// If there are no visible changes in the module's public API, gorelease
// accepts versions that increment the minor or patch version numbers. For
// example, if the base version is "v2.3.1", gorelease would accept "v2.3.2" or
// "v2.4.0" or any prerelease of those versions, like "v2.4.0-beta". If no
// version is proposed, gorelease would suggest "v2.3.2".
//
// If there are only backward compatible differences in the module's public
// API, gorelease only accepts versions that increment the minor version. For
// example, if the base version is "v2.3.1", gorelease would accept "v2.4.0"
// but not "v2.3.2".
//
// If there are incompatible API differences for a proposed version with
// major version 1 or higher, gorelease will exit with a non-zero status.
// Incompatible differences may only be released in a new major version, which
// requires creating a module with a different path. For example, if
// incompatible changes are made in the module "example.com/mod", a
// new major version must be released as a new module, "example.com/mod/v2".
// For a proposed version with major version 0, which allows incompatible
// changes, gorelease will describe all changes, but incompatible changes
// will not affect its exit status.
//
// For more information on semantic versioning, see https://semver.org.
//
// Note: gorelease does not accept build metadata in releases (like
// v1.0.0+debug). Although it is valid semver, the Go tool and other tools in
// the ecosystem do not support it, so its use is not recommended.
//
// gorelease accepts the following flags:
//
// -base=version: The version that the current version of the module will be
// compared against. This may be a version like "v1.5.2", a version query like
// "latest", or "none". If the version is "none", gorelease will not compare the
// current version against any previous version; it will only validate the
// current version. This is useful for checking the first release of a new major
// version. The version may be preceded by a different module path and an '@',
// like -base=example.com/mod/v2@v2.5.2. This is useful to compare against
// an earlier major version or a fork. If -base is not specified, gorelease will
// attempt to infer a base version from the -version flag and available released
// versions.
//
// -version=version: The proposed version to be released. If specified,
// gorelease will confirm whether this version is consistent with changes made
// to the module's public API. gorelease will exit with a non-zero status if the
// version is not valid.
//
// gorelease is eventually intended to be merged into the go command
// as "go release". See golang.org/issues/26420.
package main

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"go/build"
	"io"
	"log"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"sort"
	"strings"
	"unicode"

	"golang.org/x/exp/apidiff"
	"golang.org/x/mod/modfile"
	"golang.org/x/mod/module"
	"golang.org/x/mod/semver"
	"golang.org/x/mod/zip"
	"golang.org/x/tools/go/packages"
)

// IDEAS:
// * Should we suggest versions at all or should -version be mandatory?
// * Verify downstream modules have licenses. May need an API or library
//   for this. Be clear that we can't provide legal advice.
// * Internal packages may be relevant to submodules (for example,
//   golang.org/x/tools/internal/lsp is imported by golang.org/x/tools).
//   gorelease should detect whether this is the case and include internal
//   directories in comparison. It should be possible to opt out or specify
//   a different list of submodules.
// * Decide what to do about build constraints, particularly GOOS and GOARCH.
//   The API may be different on some platforms (e.g., x/sys).
//   Should gorelease load packages in multiple configurations in the same run?
//   Is it a compatible change if the same API is available for more platforms?
//   Is it an incompatible change for fewer?
//   How about cgo? Is adding a new cgo dependency an incompatible change?
// * Support splits and joins of nested modules. For example, if we are
//   proposing to tag a particular commit as both cloud.google.com/go v0.46.2
//   and cloud.google.com/go/storage v1.0.0, we should ensure that the sets of
//   packages provided by those modules are disjoint, and we should not report
//   the packages moved from one to the other as an incompatible change (since
//   the APIs are still compatible, just with a different module split).

// TODO(jayconrod):
// * Clean up overuse of fmt.Errorf.
// * Support migration to modules after v2.x.y+incompatible. Requires comparing
//   packages with different module paths.
// * Error when packages import from earlier major version of same module.
//   (this may be intentional; look for real examples first).
// * Mechanism to suppress error messages.

func main() {
	log.SetFlags(0)
	log.SetPrefix("gorelease: ")
	wd, err := os.Getwd()
	if err != nil {
		log.Fatal(err)
	}
	ctx := context.WithValue(context.Background(), "env", append(os.Environ(), "GO111MODULE=on"))
	success, err := runRelease(ctx, os.Stdout, wd, os.Args[1:])
	if err != nil {
		if _, ok := err.(*usageError); ok {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(2)
		} else {
			log.Fatal(err)
		}
	}
	if !success {
		os.Exit(1)
	}
}

// runRelease is the main function of gorelease. It's called by tests, so
// it writes to w instead of os.Stdout and returns an error instead of
// exiting.
func runRelease(ctx context.Context, w io.Writer, dir string, args []string) (success bool, err error) {
	// Validate arguments and flags. We'll print our own errors, since we want to
	// test without printing to stderr.
	fs := flag.NewFlagSet("gorelease", flag.ContinueOnError)
	fs.Usage = func() {}
	fs.SetOutput(io.Discard)
	var baseOpt, releaseVersion string
	fs.StringVar(&baseOpt, "base", "", "previous version to compare against")
	fs.StringVar(&releaseVersion, "version", "", "proposed version to be released")
	if err := fs.Parse(args); err != nil {
		return false, &usageError{err: err}
	}

	if len(fs.Args()) > 0 {
		return false, usageErrorf("no arguments allowed")
	}

	if releaseVersion != "" {
		if semver.Build(releaseVersion) != "" {
			return false, usageErrorf("release version %q is not a canonical semantic version: build metadata is not supported", releaseVersion)
		}
		if c := semver.Canonical(releaseVersion); c != releaseVersion {
			return false, usageErrorf("release version %q is not a canonical semantic version", releaseVersion)
		}
	}

	var baseModPath, baseVersion string
	if at := strings.Index(baseOpt, "@"); at >= 0 {
		baseModPath = baseOpt[:at]
		baseVersion = baseOpt[at+1:]
	} else if dot, slash := strings.Index(baseOpt, "."), strings.Index(baseOpt, "/"); dot >= 0 && slash >= 0 && dot < slash {
		baseModPath = baseOpt
	} else {
		baseVersion = baseOpt
	}
	if baseModPath == "" {
		if baseVersion != "" && semver.Canonical(baseVersion) == baseVersion && releaseVersion != "" {
			if cmp := semver.Compare(baseOpt, releaseVersion); cmp == 0 {
				return false, usageErrorf("-base and -version must be different")
			} else if cmp > 0 {
				return false, usageErrorf("base version (%q) must be lower than release version (%q)", baseVersion, releaseVersion)
			}
		}
	} else if baseModPath != "" && baseVersion == "none" {
		return false, usageErrorf(`base version (%q) cannot have version "none" with explicit module path`, baseOpt)
	}

	// Find the local module and repository root directories.
	modRoot, err := findModuleRoot(dir)
	if err != nil {
		return false, err
	}
	repoRoot := findRepoRoot(modRoot)

	// Load packages for the version to be released from the local directory.
	release, err := loadLocalModule(ctx, modRoot, repoRoot, releaseVersion)
	if err != nil {
		return false, err
	}

	// Find the base version if there is one, download it, and load packages from
	// the module cache.
	var max string
	if baseModPath == "" {
		if baseVersion != "" && semver.Canonical(baseVersion) == baseVersion && module.Check(release.modPath, baseVersion) != nil {
			// Base version was specified, but it's not consistent with the release
			// module path, for example, the module path is example.com/m/v2, but
			// the user said -base=v1.0.0. Instead of making the user explicitly
			// specify the base module path, we'll adjust the major version suffix.
			prefix, _, _ := module.SplitPathVersion(release.modPath)
			major := semver.Major(baseVersion)
			if strings.HasPrefix(prefix, "gopkg.in/") {
				baseModPath = prefix + "." + semver.Major(baseVersion)
			} else if major >= "v2" {
				baseModPath = prefix + "/" + major
			} else {
				baseModPath = prefix
			}
		} else {
			baseModPath = release.modPath
			max = releaseVersion
		}
	}
	base, err := loadDownloadedModule(ctx, baseModPath, baseVersion, max)
	if err != nil {
		return false, err
	}

	// Compare packages and check for other issues.
	report, err := makeReleaseReport(ctx, base, release)
	if err != nil {
		return false, err
	}
	if _, err := fmt.Fprint(w, report.String()); err != nil {
		return false, err
	}
	return report.isSuccessful(), nil
}

type moduleInfo struct {
	modRoot                  string // module root directory
	repoRoot                 string // repository root directory (may be "")
	modPath                  string // module path in go.mod
	version                  string // resolved version or "none"
	versionQuery             string // a query like "latest" or "dev-branch", if specified
	versionInferred          bool   // true if the version was unspecified and inferred
	highestTransitiveVersion string // version of the highest transitive self-dependency (cycle)
	modPathMajor             string // major version suffix like "/v3" or ".v2"
	tagPrefix                string // prefix for version tags if module not in repo root

	goModPath string        // file path to go.mod
	goModData []byte        // content of go.mod
	goSumData []byte        // content of go.sum
	goModFile *modfile.File // parsed go.mod file

	diagnostics []string            // problems not related to loading specific packages
	pkgs        []*packages.Package // loaded packages with type information

	// Versions of this module which already exist. Only loaded for release
	// (not base).
	existingVersions []string
}

// loadLocalModule loads information about a module and its packages from a
// local directory.
//
// modRoot is the directory containing the module's go.mod file.
//
// repoRoot is the root directory of the repository containing the module or "".
//
// version is a proposed version for the module or "".
func loadLocalModule(ctx context.Context, modRoot, repoRoot, version string) (m moduleInfo, err error) {
	if repoRoot != "" && !hasFilePathPrefix(modRoot, repoRoot) {
		return moduleInfo{}, fmt.Errorf("module root %q is not in repository root %q", modRoot, repoRoot)
	}

	// Load the go.mod file and check the module path and go version.
	m = moduleInfo{
		modRoot:   modRoot,
		repoRoot:  repoRoot,
		version:   version,
		goModPath: filepath.Join(modRoot, "go.mod"),
	}

	if version != "" && semver.Compare(version, "v0.0.0-99999999999999-zzzzzzzzzzzz") < 0 {
		m.diagnostics = append(m.diagnostics, fmt.Sprintf("Version %s is lower than most pseudo-versions. Consider releasing v0.1.0-0 instead.", version))
	}

	m.goModData, err = os.ReadFile(m.goModPath)
	if err != nil {
		return moduleInfo{}, err
	}
	m.goModFile, err = modfile.ParseLax(m.goModPath, m.goModData, nil)
	if err != nil {
		return moduleInfo{}, err
	}
	if m.goModFile.Module == nil {
		return moduleInfo{}, fmt.Errorf("%s: module directive is missing", m.goModPath)
	}
	m.modPath = m.goModFile.Module.Mod.Path
	if err := checkModPath(m.modPath); err != nil {
		return moduleInfo{}, err
	}
	var ok bool
	_, m.modPathMajor, ok = module.SplitPathVersion(m.modPath)
	if !ok {
		// we just validated the path above.
		panic(fmt.Sprintf("could not find version suffix in module path %q", m.modPath))
	}
	if m.goModFile.Go == nil {
		m.diagnostics = append(m.diagnostics, "go.mod: go directive is missing")
	}

	// Determine the version tag prefix for the module within the repository.
	if repoRoot != "" && modRoot != repoRoot {
		if strings.HasPrefix(m.modPathMajor, ".") {
			m.diagnostics = append(m.diagnostics, fmt.Sprintf("%s: module path starts with gopkg.in and must be declared in the root directory of the repository", m.modPath))
		} else {
			codeDir := filepath.ToSlash(modRoot[len(repoRoot)+1:])
			var altGoModPath string
			if m.modPathMajor == "" {
				// module has no major version suffix.
				// codeDir must be a suffix of modPath.
				// tagPrefix is codeDir with a trailing slash.
				if strings.HasSuffix(m.modPath, "/"+codeDir) {
					m.tagPrefix = codeDir + "/"
				} else {
					m.diagnostics = append(m.diagnostics, fmt.Sprintf("%s: module path must end with %[2]q, since it is in subdirectory %[2]q", m.modPath, codeDir))
				}
			} else {
				if strings.HasSuffix(m.modPath, "/"+codeDir) {
					// module has a major version suffix and is in a major version subdirectory.
					// codeDir must be a suffix of modPath.
					// tagPrefix must not include the major version.
					m.tagPrefix = codeDir[:len(codeDir)-len(m.modPathMajor)+1]
					altGoModPath = modRoot[:len(modRoot)-len(m.modPathMajor)+1] + "go.mod"
				} else if strings.HasSuffix(m.modPath, "/"+codeDir+m.modPathMajor) {
					// module has a major version suffix and is not in a major version subdirectory.
					// codeDir + modPathMajor is a suffix of modPath.
					// tagPrefix is codeDir with a trailing slash.
					m.tagPrefix = codeDir + "/"
					altGoModPath = filepath.Join(modRoot, m.modPathMajor[1:], "go.mod")
				} else {
					m.diagnostics = append(m.diagnostics, fmt.Sprintf("%s: module path must end with %[2]q or %q, since it is in subdirectory %[2]q", m.modPath, codeDir, codeDir+m.modPathMajor))
				}
			}

			// Modules with major version suffixes can be defined in two places
			// (e.g., sub/go.mod and sub/v2/go.mod). They must not be defined in both.
			if altGoModPath != "" {
				if data, err := os.ReadFile(altGoModPath); err == nil {
					if altModPath := modfile.ModulePath(data); m.modPath == altModPath {
						goModRel, _ := filepath.Rel(repoRoot, m.goModPath)
						altGoModRel, _ := filepath.Rel(repoRoot, altGoModPath)
						m.diagnostics = append(m.diagnostics, fmt.Sprintf("module is defined in two locations:\n\t%s\n\t%s", goModRel, altGoModRel))
					}
				}
			}
		}
	}

	// Load the module's packages.
	// We pack the module into a zip file and extract it to a temporary directory
	// as if it were published and downloaded. We'll detect any errors that would
	// occur (for example, invalid file names). We avoid loading it as the
	// main module.
	tmpModRoot, err := copyModuleToTempDir(repoRoot, m.modPath, m.modRoot)
	if err != nil {
		return moduleInfo{}, err
	}
	defer func() {
		if rerr := os.RemoveAll(tmpModRoot); err == nil && rerr != nil {
			err = fmt.Errorf("removing temporary module directory: %v", rerr)
		}
	}()
	tmpLoadDir, tmpGoModData, tmpGoSumData, pkgPaths, prepareDiagnostics, err := prepareLoadDir(ctx, m.goModFile, m.modPath, tmpModRoot, version, false)
	if err != nil {
		return moduleInfo{}, err
	}
	defer func() {
		if rerr := os.RemoveAll(tmpLoadDir); err == nil && rerr != nil {
			err = fmt.Errorf("removing temporary load directory: %v", rerr)
		}
	}()

	var loadDiagnostics []string
	m.pkgs, loadDiagnostics, err = loadPackages(ctx, m.modPath, tmpModRoot, tmpLoadDir, tmpGoModData, tmpGoSumData, pkgPaths)
	if err != nil {
		return moduleInfo{}, err
	}

	m.diagnostics = append(m.diagnostics, prepareDiagnostics...)
	m.diagnostics = append(m.diagnostics, loadDiagnostics...)

	highestVersion, err := findSelectedVersion(ctx, tmpLoadDir, m.modPath)
	if err != nil {
		return moduleInfo{}, err
	}

	if highestVersion != "" {
		// A version of the module is included in the transitive dependencies.
		// Add it to the moduleInfo so that the release report stage can use it
		// in verifying the version or suggestion a new version, depending on
		// whether the user provided a version already.
		m.highestTransitiveVersion = highestVersion
	}

	retracted, err := loadRetractions(ctx, tmpLoadDir)
	if err != nil {
		return moduleInfo{}, err
	}
	m.diagnostics = append(m.diagnostics, retracted...)

	return m, nil
}

// loadDownloadedModule downloads a module and loads information about it and
// its packages from the module cache.
//
// modPath is the module path used to fetch the module. The module's path in
// go.mod (m.modPath) may be different, for example in a soft fork intended as
// a replacement.
//
// version is the version to load. It may be "none" (indicating nothing should
// be loaded), "" (the highest available version below max should be used), a
// version query (to be resolved with 'go list'), or a canonical version.
//
// If version is "" and max is not "", available versions greater than or equal
// to max will not be considered. Typically, loadDownloadedModule is used to
// load the base version, and max is the release version.
func loadDownloadedModule(ctx context.Context, modPath, version, max string) (m moduleInfo, err error) {
	// Check the module path and version.
	// If the version is a query, resolve it to a canonical version.
	m = moduleInfo{modPath: modPath}
	if err := checkModPath(modPath); err != nil {
		return moduleInfo{}, err
	}

	var ok bool
	_, m.modPathMajor, ok = module.SplitPathVersion(modPath)
	if !ok {
		// we just validated the path above.
		panic(fmt.Sprintf("could not find version suffix in module path %q", modPath))
	}

	if version == "none" {
		// We don't have a base version to compare against.
		m.version = "none"
		return m, nil
	}
	if version == "" {
		// Unspecified version: use the highest version below max.
		m.versionInferred = true
		if m.version, err = inferBaseVersion(ctx, modPath, max); err != nil {
			return moduleInfo{}, err
		}
		if m.version == "none" {
			return m, nil
		}
	} else if version != module.CanonicalVersion(version) {
		// Version query: find the real version.
		m.versionQuery = version
		if m.version, err = queryVersion(ctx, modPath, version); err != nil {
			return moduleInfo{}, err
		}
		if m.version != "none" && max != "" && semver.Compare(m.version, max) >= 0 {
			// TODO(jayconrod): reconsider this comparison for pseudo-versions in
			// general. A query might match different pseudo-versions over time,
			// depending on ancestor versions, so this might start failing with
			// no local change.
			return moduleInfo{}, fmt.Errorf("base version %s (%s) must be lower than release version %s", m.version, m.versionQuery, max)
		}
	} else {
		// Canonical version: make sure it matches the module path.
		if err := module.CheckPathMajor(version, m.modPathMajor); err != nil {
			// TODO(golang.org/issue/39666): don't assume this is the base version
			// or that we're comparing across major versions.
			return moduleInfo{}, fmt.Errorf("can't compare major versions: base version %s does not belong to module %s", version, modPath)
		}
		m.version = version
	}

	// Download the module into the cache and load the mod file.
	// Note that goModPath is $GOMODCACHE/cache/download/$modPath/@v/$version.mod,
	// which is not inside modRoot. This is what the go command uses. Even if
	// the module didn't have a go.mod file, one will be synthesized there.
	v := module.Version{Path: modPath, Version: m.version}
	if m.modRoot, m.goModPath, err = downloadModule(ctx, v); err != nil {
		return moduleInfo{}, err
	}
	if m.goModData, err = os.ReadFile(m.goModPath); err != nil {
		return moduleInfo{}, err
	}
	if m.goModFile, err = modfile.ParseLax(m.goModPath, m.goModData, nil); err != nil {
		return moduleInfo{}, err
	}
	if m.goModFile.Module == nil {
		return moduleInfo{}, fmt.Errorf("%s: missing module directive", m.goModPath)
	}
	m.modPath = m.goModFile.Module.Mod.Path

	// Load packages.
	tmpLoadDir, tmpGoModData, tmpGoSumData, pkgPaths, _, err := prepareLoadDir(ctx, nil, m.modPath, m.modRoot, m.version, true)
	if err != nil {
		return moduleInfo{}, err
	}
	defer func() {
		if rerr := os.RemoveAll(tmpLoadDir); err == nil && rerr != nil {
			err = fmt.Errorf("removing temporary load directory: %v", err)
		}
	}()

	if m.pkgs, _, err = loadPackages(ctx, m.modPath, m.modRoot, tmpLoadDir, tmpGoModData, tmpGoSumData, pkgPaths); err != nil {
		return moduleInfo{}, err
	}

	// Calculate the existing versions.
	ev, err := existingVersions(ctx, m.modPath, tmpLoadDir)
	if err != nil {
		return moduleInfo{}, err
	}
	m.existingVersions = ev

	return m, nil
}

// makeReleaseReport returns a report comparing the current version of a
// module with a previously released version. The report notes any backward
// compatible and incompatible changes in the module's public API. It also
// diagnoses common problems, such as go.mod or go.sum being incomplete.
// The report recommends or validates a release version and indicates a
// version control tag to use (with an appropriate prefix, for modules not
// in the repository root directory).
func makeReleaseReport(ctx context.Context, base, release moduleInfo) (report, error) {
	// TODO: use apidiff.ModuleChanges.
	// Compare each pair of packages.
	// Ignore internal packages.
	// If we don't have a base version to compare against just check the new
	// packages for errors.
	shouldCompare := base.version != "none"
	isInternal := func(modPath, pkgPath string) bool {
		if !hasPathPrefix(pkgPath, modPath) {
			panic(fmt.Sprintf("package %s not in module %s", pkgPath, modPath))
		}
		for pkgPath != modPath {
			if path.Base(pkgPath) == "internal" {
				return true
			}
			pkgPath = path.Dir(pkgPath)
		}
		return false
	}
	r := report{
		base:    base,
		release: release,
	}
	for _, pair := range zipPackages(base.modPath, base.pkgs, release.modPath, release.pkgs) {
		basePkg, releasePkg := pair.base, pair.release
		switch {
		case releasePkg == nil:
			// Package removed
			if internal := isInternal(base.modPath, basePkg.PkgPath); !internal || len(basePkg.Errors) > 0 {
				pr := packageReport{
					path:       basePkg.PkgPath,
					baseErrors: basePkg.Errors,
				}
				if !internal {
					pr.Report = apidiff.Report{
						Changes: []apidiff.Change{{
							Message:    "package removed",
							Compatible: false,
						}},
					}
				}
				r.addPackage(pr)
			}

		case basePkg == nil:
			// Package added
			if internal := isInternal(release.modPath, releasePkg.PkgPath); !internal && shouldCompare || len(releasePkg.Errors) > 0 {
				pr := packageReport{
					path:          releasePkg.PkgPath,
					releaseErrors: releasePkg.Errors,
				}
				if !internal && shouldCompare {
					// If we aren't comparing against a base version, don't say
					// "package added". Only report packages with errors.
					pr.Report = apidiff.Report{
						Changes: []apidiff.Change{{
							Message:    "package added",
							Compatible: true,
						}},
					}
				}
				r.addPackage(pr)
			}

		default:
			// Matched packages
			// Both packages are internal or neither; we only consider path components
			// after the module path.
			internal := isInternal(release.modPath, releasePkg.PkgPath)
			if !internal && basePkg.Name != "main" && releasePkg.Name != "main" {
				pr := packageReport{
					path:          basePkg.PkgPath,
					baseErrors:    basePkg.Errors,
					releaseErrors: releasePkg.Errors,
					Report:        apidiff.Changes(basePkg.Types, releasePkg.Types),
				}
				r.addPackage(pr)
			}
		}
	}

	if r.canVerifyReleaseVersion() {
		if release.version == "" {
			r.suggestReleaseVersion()
		} else {
			r.validateReleaseVersion()
		}
	}

	return r, nil
}

// existingVersions returns the versions that already exist for the given
// modPath.
func existingVersions(ctx context.Context, modPath, modRoot string) (versions []string, err error) {
	defer func() {
		if err != nil {
			err = fmt.Errorf("listing versions of %s: %w", modPath, err)
		}
	}()

	type listVersions struct {
		Versions []string
	}
	cmd := exec.CommandContext(ctx, "go", "list", "-json", "-m", "-versions", modPath)
	cmd.Env = copyEnv(ctx, cmd.Env)
	cmd.Dir = modRoot
	out, err := cmd.Output()
	if err != nil {
		return nil, cleanCmdError(err)
	}
	if len(out) == 0 {
		return nil, nil
	}

	var lv listVersions
	if err := json.Unmarshal(out, &lv); err != nil {
		return nil, err
	}
	return lv.Versions, nil
}

// findRepoRoot finds the root directory of the repository that contains dir.
// findRepoRoot returns "" if it can't find the repository root.
func findRepoRoot(dir string) string {
	vcsDirs := []string{".git", ".hg", ".svn", ".bzr"}
	d := filepath.Clean(dir)
	for {
		for _, vcsDir := range vcsDirs {
			if _, err := os.Stat(filepath.Join(d, vcsDir)); err == nil {
				return d
			}
		}
		parent := filepath.Dir(d)
		if parent == d {
			return ""
		}
		d = parent
	}
}

// findModuleRoot finds the root directory of the module that contains dir.
func findModuleRoot(dir string) (string, error) {
	d := filepath.Clean(dir)
	for {
		if fi, err := os.Stat(filepath.Join(d, "go.mod")); err == nil && !fi.IsDir() {
			return dir, nil
		}
		parent := filepath.Dir(d)
		if parent == d {
			break
		}
		d = parent
	}
	return "", fmt.Errorf("%s: cannot find go.mod file", dir)
}

// checkModPath is like golang.org/x/mod/module.CheckPath, but it returns
// friendlier error messages for common mistakes.
//
// TODO(jayconrod): update module.CheckPath and delete this function.
func checkModPath(modPath string) error {
	if path.IsAbs(modPath) || filepath.IsAbs(modPath) {
		// TODO(jayconrod): improve error message in x/mod instead of checking here.
		return fmt.Errorf("module path %q must not be an absolute path.\nIt must be an address where your module may be found.", modPath)
	}
	if suffix := dirMajorSuffix(modPath); suffix == "v0" || suffix == "v1" {
		return fmt.Errorf("module path %q has major version suffix %q.\nA major version suffix is only allowed for v2 or later.", modPath, suffix)
	} else if strings.HasPrefix(suffix, "v0") {
		return fmt.Errorf("module path %q has major version suffix %q.\nA major version may not have a leading zero.", modPath, suffix)
	} else if strings.ContainsRune(suffix, '.') {
		return fmt.Errorf("module path %q has major version suffix %q.\nA major version may not contain dots.", modPath, suffix)
	}
	return module.CheckPath(modPath)
}

// inferBaseVersion returns an appropriate base version if one was not specified
// explicitly.
//
// If max is not "", inferBaseVersion returns the highest available release
// version of the module lower than max. Otherwise, inferBaseVersion returns the
// highest available release version. Pre-release versions are not considered.
// If there is no available version, and max appears to be the first release
// version (for example, "v0.1.0", "v2.0.0"), "none" is returned.
func inferBaseVersion(ctx context.Context, modPath, max string) (baseVersion string, err error) {
	defer func() {
		if err != nil {
			err = &baseVersionError{err: err, modPath: modPath}
		}
	}()

	versions, err := loadVersions(ctx, modPath)
	if err != nil {
		return "", err
	}

	for i := len(versions) - 1; i >= 0; i-- {
		v := versions[i]
		if semver.Prerelease(v) == "" &&
			(max == "" || semver.Compare(v, max) < 0) {
			return v, nil
		}
	}

	if max == "" || maybeFirstVersion(max) {
		return "none", nil
	}
	return "", fmt.Errorf("no versions found lower than %s", max)
}

// queryVersion returns the canonical version for a given module version query.
func queryVersion(ctx context.Context, modPath, query string) (resolved string, err error) {
	defer func() {
		if err != nil {
			err = fmt.Errorf("could not resolve version %s@%s: %w", modPath, query, err)
		}
	}()
	if query == "upgrade" || query == "patch" {
		return "", errors.New("query is based on requirements in main go.mod file")
	}

	tmpDir, err := os.MkdirTemp("", "")
	if err != nil {
		return "", err
	}
	defer func() {
		if rerr := os.Remove(tmpDir); rerr != nil && err == nil {
			err = rerr
		}
	}()
	arg := modPath + "@" + query
	cmd := exec.CommandContext(ctx, "go", "list", "-m", "-f", "{{.Version}}", "--", arg)
	cmd.Env = copyEnv(ctx, cmd.Env)
	cmd.Dir = tmpDir
	cmd.Env = append(cmd.Env, "GO111MODULE=on")
	out, err := cmd.Output()
	if err != nil {
		return "", cleanCmdError(err)
	}
	return strings.TrimSpace(string(out)), nil
}

// loadVersions loads the list of versions for the given module using
// 'go list -m -versions'. The returned versions are sorted in ascending
// semver order.
func loadVersions(ctx context.Context, modPath string) (versions []string, err error) {
	defer func() {
		if err != nil {
			err = fmt.Errorf("could not load versions for %s: %v", modPath, err)
		}
	}()

	tmpDir, err := os.MkdirTemp("", "")
	if err != nil {
		return nil, err
	}
	defer func() {
		if rerr := os.Remove(tmpDir); rerr != nil && err == nil {
			err = rerr
		}
	}()
	cmd := exec.CommandContext(ctx, "go", "list", "-m", "-versions", "--", modPath)
	cmd.Env = copyEnv(ctx, cmd.Env)
	cmd.Dir = tmpDir
	out, err := cmd.Output()
	if err != nil {
		return nil, cleanCmdError(err)
	}
	versions = strings.Fields(string(out))
	if len(versions) > 0 {
		versions = versions[1:] // skip module path
	}

	// Sort versions defensively. 'go list -m -versions' should always returns
	// a sorted list of versions, but it's fast and easy to sort them here, too.
	sort.Slice(versions, func(i, j int) bool {
		return semver.Compare(versions[i], versions[j]) < 0
	})
	return versions, nil
}

// maybeFirstVersion returns whether v appears to be the first version
// of a module.
func maybeFirstVersion(v string) bool {
	major, minor, patch, _, _, err := parseVersion(v)
	if err != nil {
		return false
	}
	if major == "0" {
		return minor == "0" && patch == "0" ||
			minor == "0" && patch == "1" ||
			minor == "1" && patch == "0"
	}
	return minor == "0" && patch == "0"
}

// dirMajorSuffix returns a major version suffix for a slash-separated path.
// For example, for the path "foo/bar/v2", dirMajorSuffix would return "v2".
// If no major version suffix is found, "" is returned.
//
// dirMajorSuffix is less strict than module.SplitPathVersion so that incorrect
// suffixes like "v0", "v02", "v1.2" can be detected. It doesn't handle
// special cases for gopkg.in paths.
func dirMajorSuffix(path string) string {
	i := len(path)
	for i > 0 && ('0' <= path[i-1] && path[i-1] <= '9') || path[i-1] == '.' {
		i--
	}
	if i <= 1 || i == len(path) || path[i-1] != 'v' || (i > 1 && path[i-2] != '/') {
		return ""
	}
	return path[i-1:]
}

// copyModuleToTempDir copies module files from modRoot to a subdirectory of
// scratchDir. Submodules, vendor directories, and irregular files are excluded.
// An error is returned if the module contains any files or directories that
// can't be included in a module zip file (due to special characters,
// excessive sizes, etc.).
func copyModuleToTempDir(repoRoot, modPath, modRoot string) (dir string, err error) {
	// Generate a fake version consistent with modPath. We need a canonical
	// version to create a zip file.
	version := "v0.0.0-gorelease"
	_, majorPathSuffix, _ := module.SplitPathVersion(modPath)
	if majorPathSuffix != "" {
		version = majorPathSuffix[1:] + ".0.0-gorelease"
	}
	m := module.Version{Path: modPath, Version: version}

	zipFile, err := os.CreateTemp("", "gorelease-*.zip")
	if err != nil {
		return "", err
	}
	defer func() {
		zipFile.Close()
		os.Remove(zipFile.Name())
	}()

	dir, err = os.MkdirTemp("", "gorelease")
	if err != nil {
		return "", err
	}
	defer func() {
		if err != nil {
			os.RemoveAll(dir)
			dir = ""
		}
	}()

	var fallbackToDir bool
	if repoRoot != "" {
		var err error
		fallbackToDir, err = tryCreateFromVCS(zipFile, m, modRoot, repoRoot)
		if err != nil {
			return "", err
		}
	}

	if repoRoot == "" || fallbackToDir {
		// Not a recognised repo: fall back to creating from dir.
		if err := zip.CreateFromDir(zipFile, m, modRoot); err != nil {
			var e zip.FileErrorList
			if errors.As(err, &e) {
				return "", e
			}
			return "", err
		}
	}

	if err := zipFile.Close(); err != nil {
		return "", err
	}
	if err := zip.Unzip(dir, m, zipFile.Name()); err != nil {
		return "", err
	}
	return dir, nil
}

// tryCreateFromVCS tries to create a module zip file from VCS. If it succeeds,
// it returns fallBackToDir false and a nil err. If it fails in a recoverable
// way, it returns fallBackToDir true and a nil err. If it fails in an
// unrecoverable way, it returns a non-nil err.
func tryCreateFromVCS(zipFile io.Writer, m module.Version, modRoot, repoRoot string) (fallbackToDir bool, _ error) {
	// We recognised a repo: create from VCS.
	if !hasFilePathPrefix(modRoot, repoRoot) {
		panic(fmt.Sprintf("repo root %q is not a prefix of mod root %q", repoRoot, modRoot))
	}
	hasUncommitted, err := hasGitUncommittedChanges(repoRoot)
	if err != nil {
		// Fallback to CreateFromDir.
		return true, nil
	}
	if hasUncommitted {
		return false, fmt.Errorf("repo %s has uncommitted changes", repoRoot)
	}
	modRel := filepath.ToSlash(trimFilePathPrefix(modRoot, repoRoot))
	if err := zip.CreateFromVCS(zipFile, m, repoRoot, "HEAD", modRel); err != nil {
		var fel zip.FileErrorList
		if errors.As(err, &fel) {
			return false, fel
		}
		var uve *zip.UnrecognizedVCSError
		if errors.As(err, &uve) {
			// Fallback to CreateFromDir.
			return true, nil
		}
		return false, err
	}
	// Success!
	return false, nil
}

// downloadModule downloads a specific version of a module to the
// module cache using 'go mod download'.
func downloadModule(ctx context.Context, m module.Version) (modRoot, goModPath string, err error) {
	defer func() {
		if err != nil {
			err = &downloadError{m: m, err: cleanCmdError(err)}
		}
	}()

	// Run 'go mod download' from a temporary directory to avoid needing to load
	// go.mod from gorelease's working directory (or a parent).
	// go.mod may be broken, and we don't need it.
	// TODO(golang.org/issue/36812): 'go mod download' reads go.mod even though
	// we don't need information about the main module or the build list.
	// If it didn't read go.mod in this case, we wouldn't need a temp directory.
	tmpDir, err := os.MkdirTemp("", "gorelease-download")
	if err != nil {
		return "", "", err
	}
	defer os.Remove(tmpDir)
	cmd := exec.CommandContext(ctx, "go", "mod", "download", "-json", "--", m.Path+"@"+m.Version)
	cmd.Env = copyEnv(ctx, cmd.Env)
	cmd.Dir = tmpDir
	out, err := cmd.Output()
	var xerr *exec.ExitError
	if err != nil {
		var ok bool
		if xerr, ok = err.(*exec.ExitError); !ok {
			return "", "", err
		}
	}

	// If 'go mod download' exited unsuccessfully but printed well-formed JSON
	// with an error, return that error.
	parsed := struct{ Dir, GoMod, Error string }{}
	if jsonErr := json.Unmarshal(out, &parsed); jsonErr != nil {
		if xerr != nil {
			return "", "", cleanCmdError(xerr)
		}
		return "", "", jsonErr
	}
	if parsed.Error != "" {
		return "", "", errors.New(parsed.Error)
	}
	if xerr != nil {
		return "", "", cleanCmdError(xerr)
	}
	return parsed.Dir, parsed.GoMod, nil
}

// prepareLoadDir creates a temporary directory and a go.mod file that requires
// the module being loaded. go.sum is copied if present. It also creates a .go
// file that imports every package in the given modPath. This temporary module
// is useful for two reasons. First, replace and exclude directives from the
// target module aren't applied, so we have the same view as a dependent module.
// Second, we can run commands like 'go get' without modifying the original
// go.mod and go.sum files.
//
// modFile is the pre-parsed go.mod file. If non-nil, its requirements and
// go version will be copied so that incomplete and out-of-date requirements
// may be reported later.
//
// modPath is the module's path.
//
// modRoot is the module's root directory.
//
// version is the version of the module being loaded. If must be canonical
// for modules loaded from the cache. Otherwise, it may be empty (for example,
// when no release version is proposed).
//
// cached indicates whether the module is being loaded from the module cache.
// If cached is true, then the module lives in the cache at
// $GOMODCACHE/$modPath@$version/. Its go.mod file is at
// $GOMODCACHE/cache/download/$modPath/@v/$version.mod. It must be referenced
// with a simple require. A replace directive won't work because it may not have
// a go.mod file in modRoot.
// If cached is false, then modRoot is somewhere outside the module cache
// (ex /tmp). We'll reference it with a local replace directive. It must have a
// go.mod file in modRoot.
//
// dir is the location of the temporary directory.
//
// goModData and goSumData are the contents of the go.mod and go.sum files,
// respectively.
//
// pkgPaths are the import paths of the module being loaded, including the path
// to any main packages (as if they were importable).
func prepareLoadDir(ctx context.Context, modFile *modfile.File, modPath, modRoot, version string, cached bool) (dir string, goModData, goSumData []byte, pkgPaths []string, diagnostics []string, err error) {
	defer func() {
		if err != nil {
			if cached {
				err = fmt.Errorf("preparing to load packages for %s@%s: %w", modPath, version, err)
			} else {
				err = fmt.Errorf("preparing to load packages for %s: %w", modPath, err)
			}
		}
	}()

	if module.Check(modPath, version) != nil {
		// If no version is proposed or if the version isn't valid, use a fake
		// version that matches the module's major version suffix. If the version
		// is invalid, that will be reported elsewhere.
		version = "v0.0.0-gorelease"
		if _, pathMajor, _ := module.SplitPathVersion(modPath); pathMajor != "" {
			version = pathMajor[1:] + ".0.0-gorelease"
		}
	}

	dir, err = os.MkdirTemp("", "gorelease-load")
	if err != nil {
		return "", nil, nil, nil, nil, err
	}

	f := &modfile.File{}
	f.AddModuleStmt("gorelease-load-module")
	f.AddRequire(modPath, version)
	if !cached {
		f.AddReplace(modPath, version, modRoot, "")
	}
	if modFile != nil {
		if modFile.Go != nil {
			f.AddGoStmt(modFile.Go.Version)
		}
		for _, r := range modFile.Require {
			f.AddRequire(r.Mod.Path, r.Mod.Version)
		}
	}
	goModData, err = f.Format()
	if err != nil {
		return "", nil, nil, nil, nil, err
	}
	if err := os.WriteFile(filepath.Join(dir, "go.mod"), goModData, 0666); err != nil {
		return "", nil, nil, nil, nil, err
	}

	goSumData, err = os.ReadFile(filepath.Join(modRoot, "go.sum"))
	if err != nil && !os.IsNotExist(err) {
		return "", nil, nil, nil, nil, err
	}
	if err := os.WriteFile(filepath.Join(dir, "go.sum"), goSumData, 0666); err != nil {
		return "", nil, nil, nil, nil, err
	}

	// Add a .go file with requirements, so that `go get` won't blat
	// requirements.
	fakeImports := &strings.Builder{}
	fmt.Fprint(fakeImports, "package tmp\n")
	imps, err := collectImportPaths(modPath, modRoot)
	if err != nil {
		return "", nil, nil, nil, nil, err
	}
	for _, imp := range imps {
		fmt.Fprintf(fakeImports, "import _ %q\n", imp)
	}
	if err := os.WriteFile(filepath.Join(dir, "tmp.go"), []byte(fakeImports.String()), 0666); err != nil {
		return "", nil, nil, nil, nil, err
	}

	// Add missing requirements.
	cmd := exec.CommandContext(ctx, "go", "get", "-d", ".")
	cmd.Env = copyEnv(ctx, cmd.Env)
	cmd.Dir = dir
	if _, err := cmd.Output(); err != nil {
		return "", nil, nil, nil, nil, fmt.Errorf("looking for missing dependencies: %w", cleanCmdError(err))
	}

	// Report new requirements in go.mod.
	goModPath := filepath.Join(dir, "go.mod")
	loadReqs := func(data []byte) (reqs []module.Version, err error) {
		modFile, err := modfile.ParseLax(goModPath, data, nil)
		if err != nil {
			return nil, err
		}
		for _, r := range modFile.Require {
			reqs = append(reqs, r.Mod)
		}
		return reqs, nil
	}

	oldReqs, err := loadReqs(goModData)
	if err != nil {
		return "", nil, nil, nil, nil, err
	}
	newGoModData, err := os.ReadFile(goModPath)
	if err != nil {
		return "", nil, nil, nil, nil, err
	}
	newReqs, err := loadReqs(newGoModData)
	if err != nil {
		return "", nil, nil, nil, nil, err
	}

	oldMap := make(map[module.Version]bool)
	for _, req := range oldReqs {
		oldMap[req] = true
	}
	var missing []module.Version
	for _, req := range newReqs {
		// Ignore cyclic imports, since a module never needs to require itself.
		if req.Path == modPath {
			continue
		}
		if !oldMap[req] {
			missing = append(missing, req)
		}
	}

	if len(missing) > 0 {
		var missingReqs []string
		for _, m := range missing {
			missingReqs = append(missingReqs, m.String())
		}
		diagnostics = append(diagnostics, fmt.Sprintf("go.mod: the following requirements are needed\n\t%s\nRun 'go mod tidy' to add missing requirements.", strings.Join(missingReqs, "\n\t")))
		return dir, goModData, goSumData, imps, diagnostics, nil
	}

	// Cached modules may have no go.sum.
	// We skip comparison because a downloaded module is outside the user's
	// control.
	if !cached {
		// Check if 'go get' added new hashes to go.sum.
		goSumPath := filepath.Join(dir, "go.sum")
		newGoSumData, err := os.ReadFile(goSumPath)
		if err != nil {
			if !os.IsNotExist(err) {
				return "", nil, nil, nil, nil, err
			}
			// If the sum doesn't exist, that's ok: we'll treat "no go.sum" like
			// "empty go.sum".
		}

		if !sumsMatchIgnoringPath(string(goSumData), string(newGoSumData), modPath) {
			diagnostics = append(diagnostics, "go.sum: one or more sums are missing. Run 'go mod tidy' to add missing sums.")
		}
	}

	return dir, goModData, goSumData, imps, diagnostics, nil
}

// sumsMatchIgnoringPath checks whether the two sums match. It ignores any lines
// which contains the given modPath.
func sumsMatchIgnoringPath(sum1, sum2, modPathToIgnore string) bool {
	lines1 := make(map[string]bool)
	for _, line := range strings.Split(string(sum1), "\n") {
		if line == "" {
			continue
		}
		lines1[line] = true
	}
	for _, line := range strings.Split(string(sum2), "\n") {
		if line == "" {
			continue
		}
		parts := strings.Fields(line)
		if len(parts) < 1 {
			panic(fmt.Sprintf("go.sum malformed: unexpected line %s", line))
		}
		if parts[0] == modPathToIgnore {
			continue
		}

		if !lines1[line] {
			return false
		}
	}

	lines2 := make(map[string]bool)
	for _, line := range strings.Split(string(sum2), "\n") {
		if line == "" {
			continue
		}
		lines2[line] = true
	}
	for _, line := range strings.Split(string(sum1), "\n") {
		if line == "" {
			continue
		}
		parts := strings.Fields(line)
		if len(parts) < 1 {
			panic(fmt.Sprintf("go.sum malformed: unexpected line %s", line))
		}
		if parts[0] == modPathToIgnore {
			continue
		}

		if !lines2[line] {
			return false
		}
	}

	return true
}

// collectImportPaths visits the given root and traverses its directories
// recursively, collecting the import paths of all importable packages in each
// directory along the way.
//
// modPath is the module path.
// root is the root directory of the module to collect imports for (the root
// of the modPath module).
//
// Note: the returned importPaths will include main if it exists in root.
func collectImportPaths(modPath, root string) (importPaths []string, _ error) {
	err := filepath.Walk(root, func(walkPath string, fi os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Avoid .foo, _foo, and testdata subdirectory trees.
		if !fi.IsDir() {
			return nil
		}
		base := filepath.Base(walkPath)
		if strings.HasPrefix(base, ".") || strings.HasPrefix(base, "_") || base == "testdata" || base == "internal" {
			return filepath.SkipDir
		}

		p, err := build.Default.ImportDir(walkPath, 0)
		if err != nil {
			if nogoErr := (*build.NoGoError)(nil); errors.As(err, &nogoErr) {
				// No .go files found in directory. That's ok, we'll keep
				// searching.
				return nil
			}
			return err
		}

		// Construct the import path.
		importPath := path.Join(modPath, filepath.ToSlash(trimFilePathPrefix(p.Dir, root)))
		importPaths = append(importPaths, importPath)

		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("listing packages in %s: %v", root, err)
	}

	return importPaths, nil
}

// loadPackages returns a list of all packages in the module modPath, sorted by
// package path. modRoot is the module root directory, but packages are loaded
// from loadDir, which must contain go.mod and go.sum containing goModData and
// goSumData.
//
// We load packages from a temporary external module so that replace and exclude
// directives are not applied. The loading process may also modify go.mod and
// go.sum, and we want to detect and report differences.
//
// Package loading errors will be returned in the Errors field of each package.
// Other diagnostics (such as the go.sum file being incomplete) will be
// returned through diagnostics.
// err will be non-nil in case of a fatal error that prevented packages
// from being loaded.
func loadPackages(ctx context.Context, modPath, modRoot, loadDir string, goModData, goSumData []byte, pkgPaths []string) (pkgs []*packages.Package, diagnostics []string, err error) {
	// Load packages.
	// TODO(jayconrod): if there are errors loading packages in the release
	// version, try loading in the release directory. Errors there would imply
	// that packages don't load without replace / exclude directives.
	cfg := &packages.Config{
		Mode:    packages.NeedName | packages.NeedTypes | packages.NeedImports | packages.NeedDeps,
		Dir:     loadDir,
		Context: ctx,
	}
	cfg.Env = copyEnv(ctx, cfg.Env)
	if len(pkgPaths) > 0 {
		pkgs, err = packages.Load(cfg, pkgPaths...)
		if err != nil {
			return nil, nil, err
		}
	}

	// Sort the returned packages by path.
	// packages.Load makes no guarantee about the order of returned packages.
	sort.Slice(pkgs, func(i, j int) bool {
		return pkgs[i].PkgPath < pkgs[j].PkgPath
	})

	// Trim modRoot from file paths in errors.
	prefix := modRoot + string(os.PathSeparator)
	for _, pkg := range pkgs {
		for i := range pkg.Errors {
			pkg.Errors[i].Pos = strings.TrimPrefix(pkg.Errors[i].Pos, prefix)
		}
	}

	return pkgs, diagnostics, nil
}

type packagePair struct {
	base, release *packages.Package
}

// zipPackages combines two lists of packages, sorted by package path,
// and returns a sorted list of pairs of packages with matching paths.
// If a package is in one list but not the other (because it was added or
// removed between releases), a pair will be returned with a nil
// base or release field.
func zipPackages(baseModPath string, basePkgs []*packages.Package, releaseModPath string, releasePkgs []*packages.Package) []packagePair {
	baseIndex, releaseIndex := 0, 0
	var pairs []packagePair
	for baseIndex < len(basePkgs) || releaseIndex < len(releasePkgs) {
		var basePkg, releasePkg *packages.Package
		var baseSuffix, releaseSuffix string
		if baseIndex < len(basePkgs) {
			basePkg = basePkgs[baseIndex]
			baseSuffix = trimPathPrefix(basePkg.PkgPath, baseModPath)
		}
		if releaseIndex < len(releasePkgs) {
			releasePkg = releasePkgs[releaseIndex]
			releaseSuffix = trimPathPrefix(releasePkg.PkgPath, releaseModPath)
		}

		var pair packagePair
		if basePkg != nil && (releasePkg == nil || baseSuffix < releaseSuffix) {
			// Package removed
			pair = packagePair{basePkg, nil}
			baseIndex++
		} else if releasePkg != nil && (basePkg == nil || releaseSuffix < baseSuffix) {
			// Package added
			pair = packagePair{nil, releasePkg}
			releaseIndex++
		} else {
			// Matched packages.
			pair = packagePair{basePkg, releasePkg}
			baseIndex++
			releaseIndex++
		}
		pairs = append(pairs, pair)
	}
	return pairs
}

// findSelectedVersion returns the highest version of the given modPath at
// modDir, if a module cycle exists. modDir should be a writable directory
// containing the go.mod for modPath.
//
// If no module cycle exists, it returns empty string.
func findSelectedVersion(ctx context.Context, modDir, modPath string) (latestVersion string, err error) {
	defer func() {
		if err != nil {
			err = fmt.Errorf("could not find selected version for %s: %v", modPath, err)
		}
	}()

	cmd := exec.CommandContext(ctx, "go", "list", "-m", "-f", "{{.Version}}", "--", modPath)
	cmd.Env = copyEnv(ctx, cmd.Env)
	cmd.Dir = modDir
	out, err := cmd.Output()
	if err != nil {
		return "", cleanCmdError(err)
	}
	return strings.TrimSpace(string(out)), nil
}

func copyEnv(ctx context.Context, current []string) []string {
	env, ok := ctx.Value("env").([]string)
	if !ok {
		return current
	}
	clone := make([]string, len(env))
	copy(clone, env)
	return clone
}

// loadRetractions lists all retracted deps found at the modRoot.
func loadRetractions(ctx context.Context, modRoot string) ([]string, error) {
	cmd := exec.CommandContext(ctx, "go", "list", "-json", "-m", "-u", "all")
	if env, ok := ctx.Value("env").([]string); ok {
		cmd.Env = env
	}
	cmd.Dir = modRoot
	out, err := cmd.Output()
	if err != nil {
		return nil, cleanCmdError(err)
	}

	var retracted []string
	type message struct {
		Path      string
		Version   string
		Retracted []string
	}

	dec := json.NewDecoder(bytes.NewBuffer(out))
	for {
		var m message
		if err := dec.Decode(&m); err == io.EOF {
			break
		} else if err != nil {
			return nil, err
		}
		if len(m.Retracted) == 0 {
			continue
		}
		rationale, ok := shortRetractionRationale(m.Retracted)
		if ok {
			retracted = append(retracted, fmt.Sprintf("required module %s@%s retracted by module author: %s", m.Path, m.Version, rationale))
		} else {
			retracted = append(retracted, fmt.Sprintf("required module %s@%s retracted by module author", m.Path, m.Version))
		}
	}

	return retracted, nil
}

// shortRetractionRationale returns a retraction rationale string that is safe
// to print in a terminal. It returns hard-coded strings if the rationale
// is empty, too long, or contains non-printable characters.
//
// It returns true if the rationale was printable, and false if it was not (too
// long, contains graphics, etc).
func shortRetractionRationale(rationales []string) (string, bool) {
	if len(rationales) == 0 {
		return "", false
	}
	rationale := rationales[0]

	const maxRationaleBytes = 500
	if i := strings.Index(rationale, "\n"); i >= 0 {
		rationale = rationale[:i]
	}
	rationale = strings.TrimSpace(rationale)
	if rationale == "" || rationale == "retracted by module author" {
		return "", false
	}
	if len(rationale) > maxRationaleBytes {
		return "", false
	}
	for _, r := range rationale {
		if !unicode.IsGraphic(r) && !unicode.IsSpace(r) {
			return "", false
		}
	}
	// NOTE: the go.mod parser rejects invalid UTF-8, so we don't check that here.
	return rationale, true
}

// hasGitUncommittedChanges checks if the given directory has uncommitteed git
// changes.
func hasGitUncommittedChanges(dir string) (bool, error) {
	stdout := &bytes.Buffer{}
	cmd := exec.Command("git", "status", "--porcelain")
	cmd.Dir = dir
	cmd.Stdout = stdout
	if err := cmd.Run(); err != nil {
		return false, cleanCmdError(err)
	}
	return stdout.Len() != 0, nil
}
