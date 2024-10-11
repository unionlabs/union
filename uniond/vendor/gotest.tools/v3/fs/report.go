package fs

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strings"

	"gotest.tools/v3/assert/cmp"
	"gotest.tools/v3/internal/format"
)

// Equal compares a directory to the expected structured described by a manifest
// and returns success if they match. If they do not match the failure message
// will contain all the differences between the directory structure and the
// expected structure defined by the [Manifest].
//
// Equal is a [cmp.Comparison] which can be used with [gotest.tools/v3/assert.Assert].
func Equal(path string, expected Manifest) cmp.Comparison {
	return func() cmp.Result {
		actual, err := manifestFromDir(path)
		if err != nil {
			return cmp.ResultFromError(err)
		}
		failures := eqDirectory(string(os.PathSeparator), expected.root, actual.root)
		if len(failures) == 0 {
			return cmp.ResultSuccess
		}
		msg := fmt.Sprintf("directory %s does not match expected:\n", path)
		return cmp.ResultFailure(msg + formatFailures(failures))
	}
}

type failure struct {
	path     string
	problems []problem
}

type problem string

func notEqual(property string, x, y interface{}) problem {
	return problem(fmt.Sprintf("%s: expected %s got %s", property, x, y))
}

func errProblem(reason string, err error) problem {
	return problem(fmt.Sprintf("%s: %s", reason, err))
}

func existenceProblem(filename, reason string, args ...interface{}) problem {
	return problem(filename + ": " + fmt.Sprintf(reason, args...))
}

func eqResource(x, y resource) []problem {
	var p []problem
	if x.uid != y.uid {
		p = append(p, notEqual("uid", x.uid, y.uid))
	}
	if x.gid != y.gid {
		p = append(p, notEqual("gid", x.gid, y.gid))
	}
	if x.mode != anyFileMode && x.mode != y.mode {
		p = append(p, notEqual("mode", x.mode, y.mode))
	}
	return p
}

func removeCarriageReturn(in []byte) []byte {
	return bytes.Replace(in, []byte("\r\n"), []byte("\n"), -1)
}

func eqFile(x, y *file) []problem {
	p := eqResource(x.resource, y.resource)

	switch {
	case x.content == nil:
		p = append(p, existenceProblem("content", "expected content is nil"))
		return p
	case x.content == anyFileContent:
		return p
	case y.content == nil:
		p = append(p, existenceProblem("content", "actual content is nil"))
		return p
	}

	xContent, xErr := io.ReadAll(x.content)
	defer x.content.Close()
	yContent, yErr := io.ReadAll(y.content)
	defer y.content.Close()

	if xErr != nil {
		p = append(p, errProblem("failed to read expected content", xErr))
	}
	if yErr != nil {
		p = append(p, errProblem("failed to read actual content", xErr))
	}
	if xErr != nil || yErr != nil {
		return p
	}

	if x.compareContentFunc != nil {
		r := x.compareContentFunc(yContent)
		if !r.Success() {
			p = append(p, existenceProblem("content", r.FailureMessage()))
		}
		return p
	}

	if x.ignoreCariageReturn || y.ignoreCariageReturn {
		xContent = removeCarriageReturn(xContent)
		yContent = removeCarriageReturn(yContent)
	}

	if !bytes.Equal(xContent, yContent) {
		p = append(p, diffContent(xContent, yContent))
	}
	return p
}

func diffContent(x, y []byte) problem {
	diff := format.UnifiedDiff(format.DiffConfig{
		A:    string(x),
		B:    string(y),
		From: "expected",
		To:   "actual",
	})
	// Remove the trailing newline in the diff. A trailing newline is always
	// added to a problem by formatFailures.
	diff = strings.TrimSuffix(diff, "\n")
	return problem("content:\n" + indent(diff, "    "))
}

func indent(s, prefix string) string {
	buf := new(bytes.Buffer)
	lines := strings.SplitAfter(s, "\n")
	for _, line := range lines {
		buf.WriteString(prefix + line)
	}
	return buf.String()
}

func eqSymlink(x, y *symlink) []problem {
	p := eqResource(x.resource, y.resource)
	xTarget := x.target
	yTarget := y.target
	if runtime.GOOS == "windows" {
		xTarget = strings.ToLower(xTarget)
		yTarget = strings.ToLower(yTarget)
	}
	if xTarget != yTarget {
		p = append(p, notEqual("target", x.target, y.target))
	}
	return p
}

func eqDirectory(path string, x, y *directory) []failure {
	p := eqResource(x.resource, y.resource)
	var f []failure
	matchedFiles := make(map[string]bool)

	for _, name := range sortedKeys(x.items) {
		if name == anyFile {
			continue
		}
		matchedFiles[name] = true
		xEntry := x.items[name]
		yEntry, ok := y.items[name]
		if !ok {
			p = append(p, existenceProblem(name, "expected %s to exist", xEntry.Type()))
			continue
		}

		if xEntry.Type() != yEntry.Type() {
			p = append(p, notEqual(name, xEntry.Type(), yEntry.Type()))
			continue
		}

		f = append(f, eqEntry(filepath.Join(path, name), xEntry, yEntry)...)
	}

	if len(x.filepathGlobs) != 0 {
		for _, name := range sortedKeys(y.items) {
			m := matchGlob(name, y.items[name], x.filepathGlobs)
			matchedFiles[name] = m.match
			f = append(f, m.failures...)
		}
	}

	if _, ok := x.items[anyFile]; ok {
		return maybeAppendFailure(f, path, p)
	}
	for _, name := range sortedKeys(y.items) {
		if !matchedFiles[name] {
			p = append(p, existenceProblem(name, "unexpected %s", y.items[name].Type()))
		}
	}
	return maybeAppendFailure(f, path, p)
}

func maybeAppendFailure(failures []failure, path string, problems []problem) []failure {
	if len(problems) > 0 {
		return append(failures, failure{path: path, problems: problems})
	}
	return failures
}

func sortedKeys(items map[string]dirEntry) []string {
	keys := make([]string, 0, len(items))
	for key := range items {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	return keys
}

// eqEntry assumes x and y to be the same type
func eqEntry(path string, x, y dirEntry) []failure {
	resp := func(problems []problem) []failure {
		if len(problems) == 0 {
			return nil
		}
		return []failure{{path: path, problems: problems}}
	}

	switch typed := x.(type) {
	case *file:
		return resp(eqFile(typed, y.(*file)))
	case *symlink:
		return resp(eqSymlink(typed, y.(*symlink)))
	case *directory:
		return eqDirectory(path, typed, y.(*directory))
	}
	return nil
}

type globMatch struct {
	match    bool
	failures []failure
}

func matchGlob(name string, yEntry dirEntry, globs map[string]*filePath) globMatch {
	m := globMatch{}

	for glob, expectedFile := range globs {
		ok, err := filepath.Match(glob, name)
		if err != nil {
			p := errProblem("failed to match glob pattern", err)
			f := failure{path: name, problems: []problem{p}}
			m.failures = append(m.failures, f)
		}
		if ok {
			m.match = true
			m.failures = eqEntry(name, expectedFile.file, yEntry)
			return m
		}
	}
	return m
}

func formatFailures(failures []failure) string {
	sort.Slice(failures, func(i, j int) bool {
		return failures[i].path < failures[j].path
	})

	buf := new(bytes.Buffer)
	for _, failure := range failures {
		buf.WriteString(failure.path + "\n")
		for _, problem := range failure.problems {
			buf.WriteString("  " + string(problem) + "\n")
		}
	}
	return buf.String()
}
