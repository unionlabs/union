package main

import (
	"go/token"
	"testing"

	"golang.org/x/tools/go/packages"
	"gotest.tools/v3/assert"
	"gotest.tools/v3/assert/cmp"
	"gotest.tools/v3/env"
	"gotest.tools/v3/fs"
	"gotest.tools/v3/icmd"
)

func TestMigrateFileReplacesTestingT(t *testing.T) {
	source := `
package foo

import (
	"testing"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestSomething(t *testing.T) {
	a := assert.TestingT(t)
	b := require.TestingT(t)
	c := require.TestingT(t)
	if a == b {}
	_ = c
}

func do(t require.TestingT) {}
`
	migration := newMigrationFromSource(t, source)
	migrateFile(migration)

	expected := `package foo

import (
	"testing"

	"gotest.tools/v3/assert"
)

func TestSomething(t *testing.T) {
	a := assert.TestingT(t)
	b := assert.TestingT(t)
	c := assert.TestingT(t)
	if a == b {
	}
	_ = c
}

func do(t assert.TestingT) {}
`
	actual, err := formatFile(migration)
	assert.NilError(t, err)
	assert.Assert(t, cmp.Equal(expected, string(actual)))
}

func newMigrationFromSource(t *testing.T, source string) migration {
	t.Helper()
	goMod := `module example.com/foo

require  github.com/stretchr/testify v1.7.1
`

	dir := fs.NewDir(t, t.Name(),
		fs.WithFile("foo.go", source),
		fs.WithFile("go.mod", goMod))
	fileset := token.NewFileSet()

	env.ChangeWorkingDir(t, dir.Path())
	icmd.RunCommand("go", "mod", "tidy").Assert(t, icmd.Success)

	opts := options{pkgs: []string{"./..."}}
	pkgs, err := loadPackages(opts, fileset)
	assert.NilError(t, err)
	packages.PrintErrors(pkgs)

	pkg := pkgs[0]
	assert.Assert(t, !pkg.IllTyped)

	return migration{
		file:        pkg.Syntax[0],
		fileset:     fileset,
		importNames: newImportNames(pkg.Syntax[0].Imports, opts),
		pkgInfo:     pkg.TypesInfo,
	}
}

func TestMigrateFileWithNamedCmpPackage(t *testing.T) {
	source := `
package foo

import (
	"testing"
	"github.com/stretchr/testify/assert"
)

func TestSomething(t *testing.T) {
	assert.Equal(t, "a", "b")
}
`
	migration := newMigrationFromSource(t, source)
	migration.importNames.cmp = "is"
	migrateFile(migration)

	expected := `package foo

import (
	"testing"

	"gotest.tools/v3/assert"
	is "gotest.tools/v3/assert/cmp"
)

func TestSomething(t *testing.T) {
	assert.Check(t, is.Equal("a", "b"))
}
`
	actual, err := formatFile(migration)
	assert.NilError(t, err)
	assert.Assert(t, cmp.Equal(expected, string(actual)))
}

func TestMigrateFileWithCommentsOnAssert(t *testing.T) {
	source := `
package foo

import (
	"testing"
	"github.com/stretchr/testify/assert"
)

func TestSomething(t *testing.T) {
	// This is going to fail
	assert.Equal(t, "a", "b")
}
`
	migration := newMigrationFromSource(t, source)
	migrateFile(migration)

	expected := `package foo

import (
	"testing"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/assert/cmp"
)

func TestSomething(t *testing.T) {
	// This is going to fail
	assert.Check(t, cmp.Equal("a", "b"))
}
`
	actual, err := formatFile(migration)
	assert.NilError(t, err)
	assert.Assert(t, cmp.Equal(expected, string(actual)))
}

func TestMigrateFileConvertNilToNilError(t *testing.T) {
	source := `
package foo

import (
	"testing"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/assert"
)

func TestSomething(t *testing.T) {
	var err error
	assert.Nil(t, err)
	require.Nil(t, err)
}
`
	migration := newMigrationFromSource(t, source)
	migrateFile(migration)

	expected := `package foo

import (
	"testing"

	"gotest.tools/v3/assert"
)

func TestSomething(t *testing.T) {
	var err error
	assert.Check(t, err)
	assert.NilError(t, err)
}
`
	actual, err := formatFile(migration)
	assert.NilError(t, err)
	assert.Assert(t, cmp.Equal(expected, string(actual)))
}

func TestMigrateFileConvertAssertNew(t *testing.T) {
	source := `
package foo

import (
	"testing"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestSomething(t *testing.T) {
	is := assert.New(t)
	is.Equal("one", "two")
	is.NotEqual("one", "two")

	assert := require.New(t)
	assert.Equal("one", "two")
	assert.NotEqual("one", "two")
}

func TestOtherName(z *testing.T) {
	is := require.New(z)
	is.Equal("one", "two")
}

`
	migration := newMigrationFromSource(t, source)
	migrateFile(migration)

	expected := `package foo

import (
	"testing"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/assert/cmp"
)

func TestSomething(t *testing.T) {

	assert.Check(t, cmp.Equal("one", "two"))
	assert.Check(t, "one" != "two")

	assert.Equal(t, "one", "two")
	assert.Assert(t, "one" != "two")
}

func TestOtherName(z *testing.T) {

	assert.Equal(z, "one", "two")
}
`
	actual, err := formatFile(migration)
	assert.NilError(t, err)
	assert.Assert(t, cmp.Equal(expected, string(actual)))
}

func TestMigrateFileWithExtraArgs(t *testing.T) {
	source := `
package foo

import (
	"testing"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestSomething(t *testing.T) {
	var err error
	assert.Error(t, err, "this is a comment")
	require.ErrorContains(t, err, "this in the error")
	assert.Empty(t, nil, "more comment")
	require.Equal(t, []string{}, []string{}, "because")
}
`
	migration := newMigrationFromSource(t, source)
	migration.importNames.cmp = "is"
	migrateFile(migration)

	expected := `package foo

import (
	"testing"

	"gotest.tools/v3/assert"
	is "gotest.tools/v3/assert/cmp"
)

func TestSomething(t *testing.T) {
	var err error
	assert.Check(t, is.ErrorContains(err, ""), "this is a comment")
	assert.ErrorContains(t, err, "this in the error")
	assert.Check(t, is.Len(nil, 0), "more comment")
	assert.Assert(t, is.DeepEqual([]string{}, []string{}), "because")
}
`
	actual, err := formatFile(migration)
	assert.NilError(t, err)
	assert.Assert(t, cmp.Equal(expected, string(actual)))
}

func TestMigrate_AssertAlreadyImported(t *testing.T) {
	source := `
package foo

import (
	"testing"
	"github.com/stretchr/testify/require"
	"gotest.tools/v3/assert"
)

func TestSomething(t *testing.T) {
	var err error
	assert.Error(t, err, "this is the error")
	require.Equal(t, []string{}, []string{}, "because")
}
`
	migration := newMigrationFromSource(t, source)
	migration.importNames.cmp = "is"
	migrateFile(migration)

	expected := `package foo

import (
	"testing"

	"gotest.tools/v3/assert"
	is "gotest.tools/v3/assert/cmp"
)

func TestSomething(t *testing.T) {
	var err error
	assert.Error(t, err, "this is the error")
	assert.Assert(t, is.DeepEqual([]string{}, []string{}), "because")
}
`
	actual, err := formatFile(migration)
	assert.NilError(t, err)
	assert.Assert(t, cmp.Equal(expected, string(actual)))
}

func TestMigrate_AssertAlreadyImportedWithAlias(t *testing.T) {
	source := `
package foo

import (
	"testing"
	"github.com/stretchr/testify/require"
	gtya "gotest.tools/v3/assert"
)

func TestSomething(t *testing.T) {
	var err error
	gtya.Error(t, err, "this is the error")
	require.Equal(t, []string{}, []string{}, "because")
}
`
	migration := newMigrationFromSource(t, source)
	migration.importNames.cmp = "is"
	migrateFile(migration)

	expected := `package foo

import (
	"testing"

	gtya "gotest.tools/v3/assert"
	is "gotest.tools/v3/assert/cmp"
)

func TestSomething(t *testing.T) {
	var err error
	gtya.Error(t, err, "this is the error")
	gtya.Assert(t, is.DeepEqual([]string{}, []string{}), "because")
}
`
	actual, err := formatFile(migration)
	assert.NilError(t, err)
	assert.Assert(t, cmp.Equal(expected, string(actual)))
}
