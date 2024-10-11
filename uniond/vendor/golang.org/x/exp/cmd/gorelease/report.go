// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package main

import (
	"fmt"
	"strings"

	"golang.org/x/exp/apidiff"
	"golang.org/x/mod/module"
	"golang.org/x/mod/semver"
	"golang.org/x/tools/go/packages"
)

// report describes the differences in the public API between two versions
// of a module.
type report struct {
	// base contains information about the "old" module version being compared
	// against. base.version may be "none", indicating there is no base version
	// (for example, if this is the first release). base.version may not be "".
	base moduleInfo

	// release contains information about the version of the module to release.
	// The version may be set explicitly with -version or suggested using
	// suggestVersion, in which case release.versionInferred is true.
	release moduleInfo

	// packages is a list of package reports, describing the differences
	// for individual packages, sorted by package path.
	packages []packageReport

	// versionInvalid explains why the proposed or suggested version is not valid.
	versionInvalid *versionMessage

	// haveCompatibleChanges is true if there are any backward-compatible
	// changes in non-internal packages.
	haveCompatibleChanges bool

	// haveIncompatibleChanges is true if there are any backward-incompatible
	// changes in non-internal packages.
	haveIncompatibleChanges bool

	// haveBaseErrors is true if there were errors loading packages
	// in the base version.
	haveBaseErrors bool

	// haveReleaseErrors is true if there were errors loading packages
	// in the release version.
	haveReleaseErrors bool
}

// String returns a human-readable report that lists errors, compatible changes,
// and incompatible changes in each package. If releaseVersion is set, the
// report states whether releaseVersion is valid (and why). If releaseVersion is
// not set, it suggests a new version.
func (r *report) String() string {
	buf := &strings.Builder{}
	for _, p := range r.packages {
		buf.WriteString(p.String())
	}

	if !r.canVerifyReleaseVersion() {
		return buf.String()
	}

	if len(r.release.diagnostics) > 0 {
		buf.WriteString("# diagnostics\n")
		for _, d := range r.release.diagnostics {
			fmt.Fprintln(buf, d)
		}
		buf.WriteByte('\n')
	}

	buf.WriteString("# summary\n")
	baseVersion := r.base.version
	if r.base.modPath != r.release.modPath {
		baseVersion = r.base.modPath + "@" + baseVersion
	}
	if r.base.versionInferred {
		fmt.Fprintf(buf, "Inferred base version: %s\n", baseVersion)
	} else if r.base.versionQuery != "" {
		fmt.Fprintf(buf, "Base version: %s (%s)\n", baseVersion, r.base.versionQuery)
	}

	if r.versionInvalid != nil {
		fmt.Fprintln(buf, r.versionInvalid)
	} else if r.release.versionInferred {
		if r.release.tagPrefix == "" {
			fmt.Fprintf(buf, "Suggested version: %s\n", r.release.version)
		} else {
			fmt.Fprintf(buf, "Suggested version: %[1]s (with tag %[2]s%[1]s)\n", r.release.version, r.release.tagPrefix)
		}
	} else if r.release.version != "" {
		if r.release.tagPrefix == "" {
			fmt.Fprintf(buf, "%s is a valid semantic version for this release.\n", r.release.version)

			if semver.Compare(r.release.version, "v0.0.0-99999999999999-zzzzzzzzzzzz") < 0 {
				fmt.Fprintf(buf, `Note: %s sorts lower in MVS than pseudo-versions, which may be
unexpected for users. So, it may be better to choose a different suffix.`, r.release.version)
			}
		} else {
			fmt.Fprintf(buf, "%[1]s (with tag %[2]s%[1]s) is a valid semantic version for this release\n", r.release.version, r.release.tagPrefix)
		}
	}

	if r.versionInvalid == nil && r.haveBaseErrors {
		fmt.Fprintln(buf, "Errors were found in the base version. Some API changes may be omitted.")
	}

	return buf.String()
}

func (r *report) addPackage(p packageReport) {
	r.packages = append(r.packages, p)
	if len(p.baseErrors) == 0 && len(p.releaseErrors) == 0 {
		// Only count compatible and incompatible changes if there were no errors.
		// When there are errors, definitions may be missing, and fixes may appear
		// incompatible when they are not. Changes will still be reported, but
		// they won't affect version validation or suggestions.
		for _, c := range p.Changes {
			if !c.Compatible && len(p.releaseErrors) == 0 {
				r.haveIncompatibleChanges = true
			} else if c.Compatible && len(p.baseErrors) == 0 && len(p.releaseErrors) == 0 {
				r.haveCompatibleChanges = true
			}
		}
	}
	if len(p.baseErrors) > 0 {
		r.haveBaseErrors = true
	}
	if len(p.releaseErrors) > 0 {
		r.haveReleaseErrors = true
	}
}

// validateReleaseVersion checks whether r.release.version is valid.
// If r.release.version is not valid, an error is returned explaining why.
// r.release.version must be set.
func (r *report) validateReleaseVersion() {
	if r.release.version == "" {
		panic("validateVersion called without version")
	}
	setNotValid := func(format string, args ...interface{}) {
		r.versionInvalid = &versionMessage{
			message: fmt.Sprintf("%s is not a valid semantic version for this release.", r.release.version),
			reason:  fmt.Sprintf(format, args...),
		}
	}

	if r.haveReleaseErrors {
		if r.haveReleaseErrors {
			setNotValid("Errors were found in one or more packages.")
			return
		}
	}

	// TODO(jayconrod): link to documentation for all of these errors.

	// Check that the major version matches the module path.
	_, suffix, ok := module.SplitPathVersion(r.release.modPath)
	if !ok {
		setNotValid("%s: could not find version suffix in module path", r.release.modPath)
		return
	}
	if suffix != "" {
		if suffix[0] != '/' && suffix[0] != '.' {
			setNotValid("%s: unknown module path version suffix: %q", r.release.modPath, suffix)
			return
		}
		pathMajor := suffix[1:]
		major := semver.Major(r.release.version)
		if pathMajor != major {
			setNotValid(`The major version %s does not match the major version suffix
in the module path: %s`, major, r.release.modPath)
			return
		}
	} else if major := semver.Major(r.release.version); major != "v0" && major != "v1" {
		setNotValid(`The module path does not end with the major version suffix /%s,
which is required for major versions v2 or greater.`, major)
		return
	}

	for _, v := range r.base.existingVersions {
		if semver.Compare(v, r.release.version) == 0 {
			setNotValid("version %s already exists", v)
		}
	}

	// Check that compatible / incompatible changes are consistent.
	if semver.Major(r.base.version) == "v0" || r.base.modPath != r.release.modPath {
		return
	}
	if r.haveIncompatibleChanges {
		setNotValid("There are incompatible changes.")
		return
	}
	if r.haveCompatibleChanges && semver.MajorMinor(r.base.version) == semver.MajorMinor(r.release.version) {
		setNotValid(`There are compatible changes, but the minor version is not incremented
over the base version (%s).`, r.base.version)
		return
	}

	if r.release.highestTransitiveVersion != "" && semver.Compare(r.release.highestTransitiveVersion, r.release.version) > 0 {
		setNotValid(`Module indirectly depends on a higher version of itself (%s).
		`, r.release.highestTransitiveVersion)
	}
}

// suggestReleaseVersion suggests a new version consistent with observed
// changes.
func (r *report) suggestReleaseVersion() {
	setNotValid := func(format string, args ...interface{}) {
		r.versionInvalid = &versionMessage{
			message: "Cannot suggest a release version.",
			reason:  fmt.Sprintf(format, args...),
		}
	}
	setVersion := func(v string) {
		r.release.version = v
		r.release.versionInferred = true
	}

	if r.base.modPath != r.release.modPath {
		setNotValid("Base module path is different from release.")
		return
	}

	if r.haveReleaseErrors || r.haveBaseErrors {
		setNotValid("Errors were found.")
		return
	}

	var major, minor, patch, pre string
	if r.base.version != "none" {
		minVersion := r.base.version
		if r.release.highestTransitiveVersion != "" && semver.Compare(r.release.highestTransitiveVersion, minVersion) > 0 {
			setNotValid("Module indirectly depends on a higher version of itself (%s) than the base version (%s).", r.release.highestTransitiveVersion, r.base.version)
			return
		}

		var err error
		major, minor, patch, pre, _, err = parseVersion(minVersion)
		if err != nil {
			panic(fmt.Sprintf("could not parse base version: %v", err))
		}
	}

	if r.haveIncompatibleChanges && r.base.version != "none" && pre == "" && major != "0" {
		setNotValid("Incompatible changes were detected.")
		return
		// TODO(jayconrod): briefly explain how to prepare major version releases
		// and link to documentation.
	}

	// Check whether we're comparing to the latest version of base.
	//
	// This could happen further up, but we want the more pressing errors above
	// to take precedence.
	var latestForBaseMajor string
	for _, v := range r.base.existingVersions {
		if semver.Major(v) != semver.Major(r.base.version) {
			continue
		}
		if latestForBaseMajor == "" || semver.Compare(latestForBaseMajor, v) < 0 {
			latestForBaseMajor = v
		}
	}
	if latestForBaseMajor != "" && latestForBaseMajor != r.base.version {
		setNotValid(fmt.Sprintf("Can only suggest a release version when compared against the most recent version of this major: %s.", latestForBaseMajor))
		return
	}

	if r.base.version == "none" {
		if _, pathMajor, ok := module.SplitPathVersion(r.release.modPath); !ok {
			panic(fmt.Sprintf("could not parse module path %q", r.release.modPath))
		} else if pathMajor == "" {
			setVersion("v0.1.0")
		} else {
			setVersion(pathMajor[1:] + ".0.0")
		}
		return
	}

	if pre != "" {
		// suggest non-prerelease version
	} else if r.haveCompatibleChanges || (r.haveIncompatibleChanges && major == "0") || r.requirementsChanged() {
		minor = incDecimal(minor)
		patch = "0"
	} else {
		patch = incDecimal(patch)
	}
	setVersion(fmt.Sprintf("v%s.%s.%s", major, minor, patch))
	return
}

// canVerifyReleaseVersion returns true if we can safely suggest a new version
// or if we can verify the version passed in with -version is safe to tag.
func (r *report) canVerifyReleaseVersion() bool {
	// For now, return true if the base and release module paths are the same,
	// ignoring the major version suffix.
	// TODO(#37562, #39192, #39666, #40267): there are many more situations when
	// we can't verify a new version.
	basePath := strings.TrimSuffix(r.base.modPath, r.base.modPathMajor)
	releasePath := strings.TrimSuffix(r.release.modPath, r.release.modPathMajor)
	return basePath == releasePath
}

// requirementsChanged reports whether requirements have changed from base to
// version.
//
// requirementsChanged reports true for,
//   - A requirement was upgraded to a higher minor version.
//   - A requirement was added.
//   - The version of Go was incremented.
//
// It does not report true when, for example, a requirement was downgraded or
// remove. We care more about the former since that might force dependent
// modules that have the same dependency to upgrade.
func (r *report) requirementsChanged() bool {
	if r.base.goModFile == nil {
		// There wasn't a modfile before, and now there is.
		return true
	}

	// baseReqs is a map of module path to MajorMinor of the base module
	// requirements.
	baseReqs := make(map[string]string)
	for _, r := range r.base.goModFile.Require {
		baseReqs[r.Mod.Path] = r.Mod.Version
	}

	for _, r := range r.release.goModFile.Require {
		if _, ok := baseReqs[r.Mod.Path]; !ok {
			// A module@version was added to the "require" block between base
			// and release.
			return true
		}
		if semver.Compare(semver.MajorMinor(r.Mod.Version), semver.MajorMinor(baseReqs[r.Mod.Path])) > 0 {
			// The version of r.Mod.Path increased from base to release.
			return true
		}
	}

	if r.release.goModFile.Go != nil && r.base.goModFile.Go != nil {
		if r.release.goModFile.Go.Version > r.base.goModFile.Go.Version {
			// The Go version increased from base to release.
			return true
		}
	}

	return false
}

// isSuccessful returns true the module appears to be safe to release at the
// proposed or suggested version.
func (r *report) isSuccessful() bool {
	return len(r.release.diagnostics) == 0 && r.versionInvalid == nil
}

type versionMessage struct {
	message, reason string
}

func (m versionMessage) String() string {
	return m.message + "\n" + m.reason + "\n"
}

// incDecimal returns the decimal string incremented by 1.
func incDecimal(decimal string) string {
	// Scan right to left turning 9s to 0s until you find a digit to increment.
	digits := []byte(decimal)
	i := len(digits) - 1
	for ; i >= 0 && digits[i] == '9'; i-- {
		digits[i] = '0'
	}
	if i >= 0 {
		digits[i]++
	} else {
		// digits is all zeros
		digits[0] = '1'
		digits = append(digits, '0')
	}
	return string(digits)
}

type packageReport struct {
	apidiff.Report
	path                      string
	baseErrors, releaseErrors []packages.Error
}

func (p *packageReport) String() string {
	if len(p.Changes) == 0 && len(p.baseErrors) == 0 && len(p.releaseErrors) == 0 {
		return ""
	}
	buf := &strings.Builder{}
	fmt.Fprintf(buf, "# %s\n", p.path)
	if len(p.baseErrors) > 0 {
		fmt.Fprintf(buf, "## errors in base version:\n")
		for _, e := range p.baseErrors {
			fmt.Fprintln(buf, e)
		}
		buf.WriteByte('\n')
	}
	if len(p.releaseErrors) > 0 {
		fmt.Fprintf(buf, "## errors in release version:\n")
		for _, e := range p.releaseErrors {
			fmt.Fprintln(buf, e)
		}
		buf.WriteByte('\n')
	}
	if len(p.Changes) > 0 {
		var compatible, incompatible []apidiff.Change
		for _, c := range p.Changes {
			if c.Compatible {
				compatible = append(compatible, c)
			} else {
				incompatible = append(incompatible, c)
			}
		}
		if len(incompatible) > 0 {
			fmt.Fprintf(buf, "## incompatible changes\n")
			for _, c := range incompatible {
				fmt.Fprintln(buf, c.Message)
			}
		}
		if len(compatible) > 0 {
			fmt.Fprintf(buf, "## compatible changes\n")
			for _, c := range compatible {
				fmt.Fprintln(buf, c.Message)
			}
		}
		buf.WriteByte('\n')
	}
	return buf.String()
}

// parseVersion returns the major, minor, and patch numbers, prerelease text,
// and metadata for a given version.
//
// TODO(jayconrod): extend semver to do this and delete this function.
func parseVersion(vers string) (major, minor, patch, pre, meta string, err error) {
	if !strings.HasPrefix(vers, "v") {
		return "", "", "", "", "", fmt.Errorf("version %q does not start with 'v'", vers)
	}
	base := vers[1:]
	if i := strings.IndexByte(base, '+'); i >= 0 {
		meta = base[i+1:]
		base = base[:i]
	}
	if i := strings.IndexByte(base, '-'); i >= 0 {
		pre = base[i+1:]
		base = base[:i]
	}
	parts := strings.Split(base, ".")
	if len(parts) != 3 {
		return "", "", "", "", "", fmt.Errorf("version %q should have three numbers", vers)
	}
	major, minor, patch = parts[0], parts[1], parts[2]
	return major, minor, patch, pre, meta, nil
}
