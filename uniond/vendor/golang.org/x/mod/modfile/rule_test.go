// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package modfile

import (
	"bytes"
	"fmt"
	"strings"
	"testing"

	"golang.org/x/mod/module"
)

var addRequireTests = []struct {
	desc string
	in   string
	path string
	vers string
	out  string
}{
	{
		`existing`,
		`
		module m
		require x.y/z v1.2.3
		`,
		"x.y/z", "v1.5.6",
		`
		module m
		require x.y/z v1.5.6
		`,
	},
	{
		`existing2`,
		`
		module m
		require (
			x.y/z v1.2.3 // first
			x.z/a v0.1.0 // first-a
		)
		require x.y/z v1.4.5 // second
		require (
			x.y/z v1.6.7 // third
			x.z/a v0.2.0 // third-a
		)
		`,
		"x.y/z", "v1.8.9",
		`
		module m

		require (
			x.y/z v1.8.9 // first
			x.z/a v0.1.0 // first-a
		)

		require x.z/a v0.2.0 // third-a
		`,
	},
	{
		`new`,
		`
		module m
		require x.y/z v1.2.3
		`,
		"x.y/w", "v1.5.6",
		`
		module m
		require (
			x.y/z v1.2.3
			x.y/w v1.5.6
		)
		`,
	},
	{
		`new2`,
		`
		module m
		require x.y/z v1.2.3
		require x.y/q/v2 v2.3.4
		`,
		"x.y/w", "v1.5.6",
		`
		module m
		require x.y/z v1.2.3
		require (
			x.y/q/v2 v2.3.4
			x.y/w v1.5.6
		)
		`,
	},
	{
		`unattached_comments`,
		`
		module m
		require (
			foo v0.0.0-00010101000000-000000000000
			// bar v0.0.0-00010101000000-000000000000
		)
		`,
		"foo", "v0.0.0-00010101000000-000000000000",
		`
		module m
		require (
			foo v0.0.0-00010101000000-000000000000
			// bar v0.0.0-00010101000000-000000000000
		)
		`,
	},
}

type require struct {
	path, vers string
	indirect   bool
}

var setRequireTests = []struct {
	desc string
	in   string
	mods []require
	out  string
}{
	{
		`https://golang.org/issue/45932`,
		`module m
		require (
			x.y/a v1.2.3 //indirect
			x.y/b v1.2.3
			x.y/c v1.2.3
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", false},
			{"x.y/b", "v1.2.3", false},
			{"x.y/c", "v1.2.3", false},
		},
		`module m
		require (
			x.y/a v1.2.3
			x.y/b v1.2.3
			x.y/c v1.2.3
		)
		`,
	},
	{
		`existing`,
		`module m
		require (
			x.y/b v1.2.3

			x.y/a v1.2.3
			x.y/d v1.2.3
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", false},
			{"x.y/b", "v1.2.3", false},
			{"x.y/c", "v1.2.3", false},
		},
		`module m
		require (
			x.y/a v1.2.3
			x.y/b v1.2.3
			x.y/c v1.2.3
		)
		`,
	},
	{
		`existing_indirect`,
		`module m
		require (
			x.y/a v1.2.3
			x.y/b v1.2.3 //
			x.y/c v1.2.3 //c
			x.y/d v1.2.3 //   c
			x.y/e v1.2.3 // indirect
			x.y/f v1.2.3 //indirect
			x.y/g v1.2.3 //	indirect
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", true},
			{"x.y/b", "v1.2.3", true},
			{"x.y/c", "v1.2.3", true},
			{"x.y/d", "v1.2.3", true},
			{"x.y/e", "v1.2.3", true},
			{"x.y/f", "v1.2.3", true},
			{"x.y/g", "v1.2.3", true},
		},
		`module m
		require (
			x.y/a v1.2.3 // indirect
			x.y/b v1.2.3 // indirect
			x.y/c v1.2.3 // indirect; c
			x.y/d v1.2.3 // indirect; c
			x.y/e v1.2.3 // indirect
			x.y/f v1.2.3 //indirect
			x.y/g v1.2.3 //	indirect
		)
		`,
	},
	{
		`existing_multi`,
		`module m
		require x.y/a v1.2.3
		require x.y/b v1.2.3
		require x.y/c v1.0.0 // not v1.2.3!
		require x.y/d v1.2.3 // comment kept
		require x.y/e v1.2.3 // comment kept
		require x.y/f v1.2.3 // indirect
		require x.y/g v1.2.3 // indirect
		`,
		[]require{
			{"x.y/h", "v1.2.3", false},
			{"x.y/a", "v1.2.3", false},
			{"x.y/b", "v1.2.3", false},
			{"x.y/c", "v1.2.3", false},
			{"x.y/d", "v1.2.3", false},
			{"x.y/e", "v1.2.3", true},
			{"x.y/f", "v1.2.3", false},
			{"x.y/g", "v1.2.3", false},
		},
		`module m
		require x.y/a v1.2.3

		require x.y/b v1.2.3

		require x.y/c v1.2.3 // not v1.2.3!

		require x.y/d v1.2.3 // comment kept

		require x.y/e v1.2.3 // indirect; comment kept

		require x.y/f v1.2.3

		require (
			x.y/g v1.2.3
			x.y/h v1.2.3
		)
		`,
	},
	{
		`existing_duplicate`,
		`module m
		require (
			x.y/a v1.0.0 // zero
			x.y/a v1.1.0 // one
			x.y/a v1.2.3 // two
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", true},
		},
		`module m
		require x.y/a v1.2.3 // indirect; zero
		`,
	},
	{
		`existing_duplicate_multi`,
		`module m
		require x.y/a v1.0.0 // zero
		require x.y/a v1.1.0 // one
		require x.y/a v1.2.3 // two
		`,
		[]require{
			{"x.y/a", "v1.2.3", true},
		},
		`module m
		require x.y/a v1.2.3 // indirect; zero
		`,
	},
}

var setRequireSeparateIndirectTests = []struct {
	desc string
	in   string
	mods []require
	out  string
}{
	{
		`https://golang.org/issue/45932`,
		`module m
		require (
			x.y/a v1.2.3 //indirect
			x.y/b v1.2.3
			x.y/c v1.2.3
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", false},
			{"x.y/b", "v1.2.3", false},
			{"x.y/c", "v1.2.3", false},
		},
		`module m
		require (
			x.y/a v1.2.3
			x.y/b v1.2.3
			x.y/c v1.2.3
		)
		`,
	},
	{
		`existing`,
		`module m
		require (
			x.y/b v1.2.3

			x.y/a v1.2.3
			x.y/d v1.2.3
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", false},
			{"x.y/b", "v1.2.3", false},
			{"x.y/c", "v1.2.3", false},
		},
		`module m
		require (
			x.y/a v1.2.3
			x.y/b v1.2.3
			x.y/c v1.2.3
		)
		`,
	},
	{
		`existing_indirect`,
		`module m
		require (
			x.y/a v1.2.3
			x.y/b v1.2.3 //
			x.y/c v1.2.3 //c
			x.y/d v1.2.3 //   c
			x.y/e v1.2.3 // indirect
			x.y/f v1.2.3 //indirect
			x.y/g v1.2.3 //	indirect
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", true},
			{"x.y/b", "v1.2.3", true},
			{"x.y/c", "v1.2.3", true},
			{"x.y/d", "v1.2.3", true},
			{"x.y/e", "v1.2.3", true},
			{"x.y/f", "v1.2.3", true},
			{"x.y/g", "v1.2.3", true},
		},
		`module m
		require (
			x.y/a v1.2.3 // indirect
			x.y/b v1.2.3 // indirect
			x.y/c v1.2.3 // indirect; c
			x.y/d v1.2.3 // indirect; c
			x.y/e v1.2.3 // indirect
			x.y/f v1.2.3 //indirect
			x.y/g v1.2.3 //	indirect
		)
		`,
	},
	{
		`existing_line`,
		`module m
		require x.y/a v1.0.0
		require x.y/c v1.0.0 // indirect
		`,
		[]require{
			{"x.y/a", "v1.2.3", false},
			{"x.y/b", "v1.2.3", false},
			{"x.y/c", "v1.2.3", true},
			{"x.y/d", "v1.2.3", true},
		},
		`module m
		require (
			x.y/a v1.2.3
			x.y/b v1.2.3
		)
		require (
			x.y/c v1.2.3 // indirect
			x.y/d v1.2.3 // indirect
		)`,
	},
	{
		`existing_multi`,
		`module m
		require x.y/a v1.2.3
		require x.y/b v1.2.3 // demoted to indirect
		require x.y/c v1.0.0 // not v1.2.3!
		require x.y/d v1.2.3 // comment kept
		require x.y/e v1.2.3 // comment kept
		require x.y/f v1.2.3 // indirect; promoted to direct
		// promoted to direct
		require x.y/g v1.2.3 // indirect
		require x.y/i v1.2.3 // indirect
		require x.y/j v1.2.3 // indirect
		`,
		[]require{
			{"x.y/h", "v1.2.3", false}, // out of alphabetical order
			{"x.y/i", "v1.2.3", true},
			{"x.y/j", "v1.2.3", true},
			{"x.y/a", "v1.2.3", false},
			{"x.y/b", "v1.2.3", true},
			{"x.y/c", "v1.2.3", false},
			{"x.y/d", "v1.2.3", false},
			{"x.y/e", "v1.2.3", true},
			{"x.y/f", "v1.2.3", false},
			{"x.y/g", "v1.2.3", false},
		},
		`module m
		require (
			x.y/a v1.2.3
			x.y/h v1.2.3
		)
		require x.y/b v1.2.3 // indirect; demoted to indirect
		require x.y/c v1.2.3 // not v1.2.3!
		require x.y/d v1.2.3 // comment kept
		require x.y/e v1.2.3 // indirect; comment kept
		require x.y/f v1.2.3 // promoted to direct
		// promoted to direct
		require x.y/g v1.2.3
		require x.y/i v1.2.3 // indirect
		require x.y/j v1.2.3 // indirect
		`,
	},
	{
		`existing_duplicate`,
		`module m
		require (
			x.y/a v1.0.0 // zero
			x.y/a v1.1.0 // one
			x.y/a v1.2.3 // two
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", true},
		},
		`module m
		require x.y/a v1.2.3 // indirect; zero
		`,
	},
	{
		`existing_duplicate_multi`,
		`module m
		require x.y/a v1.0.0 // zero
		require x.y/a v1.1.0 // one
		require x.y/a v1.2.3 // two
		`,
		[]require{
			{"x.y/a", "v1.2.3", true},
		},
		`module m
		require x.y/a v1.2.3 // indirect; zero
		`,
	},
	{
		`existing_duplicate_mix_indirect`,
		`module m
		require (
			x.y/a v1.0.0 // zero
			x.y/a v1.1.0 // indirect; one
			x.y/a v1.2.3 // indirect; two
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", true},
		},
		`module m
		require x.y/a v1.2.3 // indirect; zero
		`,
	},
	{
		`existing_duplicate_mix_direct`,
		`module m
		require (
			x.y/a v1.0.0 // indirect; zero
			x.y/a v1.1.0 // one
			x.y/a v1.2.3 // two
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", false},
		},
		`module m
		require x.y/a v1.2.3 // zero
		`,
	},
	{
		`add_indirect_after_last_direct`,
		`module m
		require (
			x.y/a v1.0.0 // comment a preserved
			x.y/d v1.0.0 // comment d preserved
		)
		require (
			x.y/b v1.0.0 // comment b preserved
			x.y/e v1.0.0 // comment e preserved
		)
		go 1.17
		`,
		[]require{
			{"x.y/a", "v1.2.3", false},
			{"x.y/b", "v1.2.3", false},
			{"x.y/c", "v1.2.3", true},
			{"x.y/d", "v1.2.3", false},
			{"x.y/e", "v1.2.3", false},
			{"x.y/f", "v1.2.3", true},
		},
		`module m
		require (
			x.y/a v1.2.3 // comment a preserved
			x.y/d v1.2.3 // comment d preserved
		)
		require (
			x.y/b v1.2.3 // comment b preserved
			x.y/e v1.2.3 // comment e preserved
		)
		require (
			x.y/c v1.2.3 // indirect
			x.y/f v1.2.3 // indirect
		)
		go 1.17
		`,
	},
	{
		`add_direct_before_first_indirect`,
		`module m
		require (
			x.y/b v1.0.0 // indirect; comment b preserved
			x.y/e v1.0.0 // indirect; comment d preserved
		)
		require (
			x.y/c v1.0.0 // indirect; comment c preserved
			x.y/f v1.0.0 // indirect; comment e preserved
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", false},
			{"x.y/b", "v1.2.3", true},
			{"x.y/c", "v1.2.3", true},
			{"x.y/d", "v1.2.3", false},
			{"x.y/e", "v1.2.3", true},
			{"x.y/f", "v1.2.3", true},
		},
		`module m
		require (
			x.y/b v1.2.3 // indirect; comment b preserved
			x.y/e v1.2.3 // indirect; comment d preserved
		)
		require (
			x.y/c v1.2.3 // indirect; comment c preserved
			x.y/f v1.2.3 // indirect; comment e preserved
		)
		require (
			x.y/a v1.2.3
			x.y/d v1.2.3
		)
		`,
	},
	{
		`add_indirect_after_mixed`,
		`module m
		require (
			x.y/a v1.0.0
			x.y/b v1.0.0 // indirect
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", false},
			{"x.y/b", "v1.2.3", true},
			{"x.y/c", "v1.2.3", true},
			{"x.y/d", "v1.2.3", false},
			{"x.y/e", "v1.2.3", true},
		},
		`module m
		require (
			x.y/a v1.2.3
			x.y/d v1.2.3
		)
		require (
			x.y/b v1.2.3 // indirect
			x.y/c v1.2.3 // indirect
			x.y/e v1.2.3 // indirect
		)
		`,
	},
	{
		`preserve_block_comment_indirect_to_direct`,
		`module m
		// save
		require (
			x.y/a v1.2.3 // indirect
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", false},
		},
		`module m

		// save
		require x.y/a v1.2.3
		`,
	},
	{
		`preserve_block_comment_direct_to_indirect`,
		`module m
		// save
		require (
			x.y/a v1.2.3
		)
		`,
		[]require{
			{"x.y/a", "v1.2.3", true},
		},
		`module m

		// save
		require x.y/a v1.2.3 // indirect
		`,
	},
	{
		`regroup_flat_uncommented_block`,
		`module m
		require (
			x.y/a v1.0.0 // a
			x.y/b v1.0.0 // indirect; b
			x.y/c v1.0.0 // indirect
		)`,
		[]require{
			{"x.y/a", "v1.2.3", false},
			{"x.y/b", "v1.2.3", true},
			{"x.y/c", "v1.2.3", true},
			{"x.y/d", "v1.2.3", false},
		},
		`module m
		require (
			x.y/a v1.2.3 // a
			x.y/d v1.2.3
		)
		require (
			x.y/b v1.2.3 // indirect; b
			x.y/c v1.2.3 // indirect
		)`,
	},
	{
		`dont_regroup_flat_commented_block`,
		`module m
		// dont regroup
		require (
			x.y/a v1.0.0
			x.y/b v1.0.0 // indirect
			x.y/c v1.0.0 // indirect
		)`,
		[]require{
			{"x.y/a", "v1.2.3", false},
			{"x.y/b", "v1.2.3", true},
			{"x.y/c", "v1.2.3", true},
			{"x.y/d", "v1.2.3", false},
		},
		`module m
		// dont regroup
		require (
			x.y/a v1.2.3
			x.y/b v1.2.3 // indirect
			x.y/c v1.2.3 // indirect
		)
		require x.y/d v1.2.3`,
	},
}

var addGoTests = []struct {
	desc    string
	in      string
	version string
	out     string
}{
	{
		`module_only`,
		`module m
		`,
		`1.14`,
		`module m
		go 1.14
		`,
	},
	{
		`module_before_require`,
		`module m
		require x.y/a v1.2.3
		`,
		`1.14`,
		`module m
		go 1.14
		require x.y/a v1.2.3
		`,
	},
	{
		`require_before_module`,
		`require x.y/a v1.2.3
		module example.com/inverted
		`,
		`1.14`,
		`require x.y/a v1.2.3
		module example.com/inverted
		go 1.14
		`,
	},
	{
		`require_only`,
		`require x.y/a v1.2.3
		`,
		`1.14`,
		`require x.y/a v1.2.3
		go 1.14
		`,
	},
}

var dropGoTests = []struct {
	desc string
	in   string
	out  string
}{
	{
		`module_only`,
		`module m
		go 1.14
		`,
		`module m
		`,
	},
	{
		`module_before_require`,
		`module m
		go 1.14
		require x.y/a v1.2.3
		`,
		`module m
		require x.y/a v1.2.3
		`,
	},
	{
		`require_before_module`,
		`require x.y/a v1.2.3
		module example.com/inverted
		go 1.14
		`,
		`require x.y/a v1.2.3
		module example.com/inverted
		`,
	},
	{
		`require_only`,
		`require x.y/a v1.2.3
		go 1.14
		`,
		`require x.y/a v1.2.3
		`,
	},
}

var addToolchainTests = []struct {
	desc    string
	in      string
	version string
	out     string
}{
	{
		`empty`,
		``,
		`go1.17`,
		`toolchain go1.17
		`,
	},
	{
		`aftergo`,
		`// this is a comment
		require x v1.0.0

		go 1.17

		require y v1.0.0
		`,
		`go1.17`,
		`// this is a comment
		require x v1.0.0

		go 1.17

		toolchain go1.17

		require y v1.0.0
		`,
	},
	{
		`already_have_toolchain`,
		`go 1.17

		toolchain go1.18
		`,
		`go1.19`,
		`go 1.17

		toolchain go1.19
		`,
	},
}

var dropToolchainTests = []struct {
	desc string
	in   string
	out  string
}{
	{
		`empty`,
		`toolchain go1.17
		`,
		``,
	},
	{
		`aftergo`,
		`// this is a comment
		require x v1.0.0

		go 1.17

		toolchain go1.17

		require y v1.0.0
		`,
		`// this is a comment
		require x v1.0.0

		go 1.17

		require y v1.0.0
		`,
	},
	{
		`already_have_toolchain`,
		`go 1.17

		toolchain go1.18
		`,
		`go 1.17
		`,
	},
}

var addExcludeTests = []struct {
	desc    string
	in      string
	path    string
	version string
	out     string
}{
	{
		`compatible`,
		`module m
		`,
		`example.com`,
		`v1.2.3`,
		`module m
		exclude example.com v1.2.3
		`,
	},
	{
		`gopkg.in v0`,
		`module m
		`,
		`gopkg.in/foo.v0`,
		`v0.2.3`,
		`module m
		exclude gopkg.in/foo.v0 v0.2.3
		`,
	},
	{
		`gopkg.in v1`,
		`module m
		`,
		`gopkg.in/foo.v1`,
		`v1.2.3`,
		`module m
		exclude gopkg.in/foo.v1 v1.2.3
		`,
	},
}

var addRetractTests = []struct {
	desc      string
	in        string
	low       string
	high      string
	rationale string
	out       string
}{
	{
		`new_singleton`,
		`module m
		`,
		`v1.2.3`,
		`v1.2.3`,
		``,
		`module m
		retract v1.2.3
		`,
	},
	{
		`new_interval`,
		`module m
		`,
		`v1.0.0`,
		`v1.1.0`,
		``,
		`module m
		retract [v1.0.0, v1.1.0]`,
	},
	{
		`duplicate_with_rationale`,
		`module m
		retract v1.2.3
		`,
		`v1.2.3`,
		`v1.2.3`,
		`bad`,
		`module m
		retract (
			v1.2.3
			// bad
			v1.2.3
		)
		`,
	},
	{
		`duplicate_multiline_rationale`,
		`module m
		retract [v1.2.3, v1.2.3]
		`,
		`v1.2.3`,
		`v1.2.3`,
		`multi
line`,
		`module m
		retract	(
			[v1.2.3, v1.2.3]
			// multi
			// line
			v1.2.3
		)
		`,
	},
	{
		`duplicate_interval`,
		`module m
		retract [v1.0.0, v1.1.0]
		`,
		`v1.0.0`,
		`v1.1.0`,
		``,
		`module m
		retract (
			[v1.0.0, v1.1.0]
			[v1.0.0, v1.1.0]
		)
		`,
	},
	{
		`duplicate_singleton`,
		`module m
		retract v1.2.3
		`,
		`v1.2.3`,
		`v1.2.3`,
		``,
		`module m
		retract	(
			v1.2.3
			v1.2.3
		)
		`,
	},
}

var dropRetractTests = []struct {
	desc string
	in   string
	low  string
	high string
	out  string
}{
	{
		`singleton_no_match`,
		`module m
		retract v1.2.3
		`,
		`v1.0.0`,
		`v1.0.0`,
		`module m
		retract v1.2.3
		`,
	},
	{
		`singleton_match_one`,
		`module m
		retract v1.2.2
		retract v1.2.3
		retract v1.2.4
		`,
		`v1.2.3`,
		`v1.2.3`,
		`module m
		retract v1.2.2
		retract v1.2.4
		`,
	},
	{
		`singleton_match_all`,
		`module m
		retract v1.2.3 // first
		retract v1.2.3 // second
		`,
		`v1.2.3`,
		`v1.2.3`,
		`module m
		`,
	},
	{
		`interval_match`,
		`module m
		retract [v1.2.3, v1.2.3]
		`,
		`v1.2.3`,
		`v1.2.3`,
		`module m
		`,
	},
	{
		`interval_superset_no_match`,
		`module m
		retract [v1.0.0, v1.1.0]
		`,
		`v1.0.0`,
		`v1.2.0`,
		`module m
		retract [v1.0.0, v1.1.0]
		`,
	},
	{
		`singleton_match_middle`,
		`module m
		retract v1.2.3
		`,
		`v1.2.3`,
		`v1.2.3`,
		`module m
		`,
	},
	{
		`interval_match_middle_block`,
		`module m
		retract (
			v1.0.0
			[v1.1.0, v1.2.0]
			v1.3.0
		)
		`,
		`v1.1.0`,
		`v1.2.0`,
		`module m
		retract (
			v1.0.0
			v1.3.0
		)
		`,
	},
	{
		`interval_match_all`,
		`module m
		retract [v1.0.0, v1.1.0]
		retract [v1.0.0, v1.1.0]
		`,
		`v1.0.0`,
		`v1.1.0`,
		`module m
		`,
	},
}

var retractRationaleTests = []struct {
	desc, in, want string
}{
	{
		`no_comment`,
		`module m
		retract v1.0.0`,
		``,
	},
	{
		`prefix_one`,
		`module m
		//   prefix
		retract v1.0.0
		`,
		`prefix`,
	},
	{
		`prefix_multiline`,
		`module m
		//  one
		//
		//     two
		//
		// three
		retract v1.0.0`,
		`one

two

three`,
	},
	{
		`suffix`,
		`module m
		retract v1.0.0 // suffix
		`,
		`suffix`,
	},
	{
		`prefix_suffix_after`,
		`module m
		// prefix
		retract v1.0.0 // suffix
		`,
		`prefix
suffix`,
	},
	{
		`block_only`,
		`// block
		retract (
			v1.0.0
		)
		`,
		`block`,
	},
	{
		`block_and_line`,
		`// block
		retract (
			// line
			v1.0.0
		)
		`,
		`line`,
	},
}

var moduleDeprecatedTests = []struct {
	desc, in, want string
}{
	// retractRationaleTests exercises some of the same code, so these tests
	// don't exhaustively cover comment extraction.
	{
		`no_comment`,
		`module m`,
		``,
	},
	{
		`other_comment`,
		`// yo
		module m`,
		``,
	},
	{
		`deprecated_no_colon`,
		`//Deprecated
		module m`,
		``,
	},
	{
		`deprecated_no_space`,
		`//Deprecated:blah
		module m`,
		`blah`,
	},
	{
		`deprecated_simple`,
		`// Deprecated: blah
		module m`,
		`blah`,
	},
	{
		`deprecated_lowercase`,
		`// deprecated: blah
		module m`,
		``,
	},
	{
		`deprecated_multiline`,
		`// Deprecated: one
		// two
		module m`,
		"one\ntwo",
	},
	{
		`deprecated_mixed`,
		`// some other comment
		// Deprecated: blah
		module m`,
		``,
	},
	{
		`deprecated_middle`,
		`// module m is Deprecated: blah
		module m`,
		``,
	},
	{
		`deprecated_multiple`,
		`// Deprecated: a
		// Deprecated: b
		module m`,
		"a\nDeprecated: b",
	},
	{
		`deprecated_paragraph`,
		`// Deprecated: a
		// b
		//
		// c
		module m`,
		"a\nb",
	},
	{
		`deprecated_paragraph_space`,
		`// Deprecated: the next line has a space
		//
		// c
		module m`,
		"the next line has a space",
	},
	{
		`deprecated_suffix`,
		`module m // Deprecated: blah`,
		`blah`,
	},
	{
		`deprecated_mixed_suffix`,
		`// some other comment
		module m // Deprecated: blah`,
		``,
	},
	{
		`deprecated_mixed_suffix_paragraph`,
		`// some other comment
		//
		module m // Deprecated: blah`,
		`blah`,
	},
	{
		`deprecated_block`,
		`// Deprecated: blah
		module (
			m
		)`,
		`blah`,
	},
}

var sortBlocksTests = []struct {
	desc, in, out string
	strict        bool
}{
	{
		`exclude_duplicates_removed`,
		`module m
		exclude x.y/z v1.0.0 // a
		exclude x.y/z v1.0.0 // b
		exclude (
			x.y/w v1.1.0
			x.y/z v1.0.0 // c
		)
		`,
		`module m
		exclude x.y/z v1.0.0 // a
		exclude (
			x.y/w v1.1.0
		)`,
		true,
	},
	{
		`replace_duplicates_removed`,
		`module m
		replace x.y/z v1.0.0 => ./a
		replace x.y/z v1.1.0 => ./b
		replace (
			x.y/z v1.0.0 => ./c
		)
		`,
		`module m
		replace x.y/z v1.1.0 => ./b
		replace (
			x.y/z v1.0.0 => ./c
		)
		`,
		true,
	},
	{
		`retract_duplicates_not_removed`,
		`module m
		// block
		retract (
			v1.0.0 // one
			v1.0.0 // two
		)`,
		`module m
		// block
		retract (
			v1.0.0 // one
			v1.0.0 // two
		)`,
		true,
	},
	// Tests below this point just check sort order.
	// Non-retract blocks are sorted lexicographically in ascending order.
	// retract blocks are sorted using semver in descending order.
	{
		`sort_lexicographically`,
		`module m
		sort (
			aa
			cc
			bb
			zz
			v1.2.0
			v1.11.0
		)`,
		`module m
		sort (
			aa
			bb
			cc
			v1.11.0
			v1.2.0
			zz
		)
		`,
		false,
	},
	{
		`sort_retract`,
		`module m
		retract (
			[v1.2.0, v1.3.0]
			[v1.1.0, v1.3.0]
			[v1.1.0, v1.2.0]
			v1.0.0
			v1.1.0
			v1.2.0
			v1.3.0
			v1.4.0
		)
		`,
		`module m
		retract (
			v1.4.0
			v1.3.0
			[v1.2.0, v1.3.0]
			v1.2.0
			[v1.1.0, v1.3.0]
			[v1.1.0, v1.2.0]
			v1.1.0
			v1.0.0
		)
		`,
		false,
	},
	// Exclude blocks are sorted using semver in ascending order
	// in go.mod files that opt in to Go version 1.21 or newer.
	{
		`sort_exclude_go121_semver`,
		`module m
		go 1.21
		exclude (
			b.example/m v0.9.0
			a.example/m v1.0.0
			b.example/m v0.10.0
			c.example/m v1.1.0
			b.example/m v0.11.0
		)`,
		`module m
		go 1.21
		exclude (
			a.example/m v1.0.0
			b.example/m v0.9.0
			b.example/m v0.10.0
			b.example/m v0.11.0
			c.example/m v1.1.0
		)
		`,
		true,
	},
	{
		`sort_exclude_!go121_lexicographically`, // Maintain the previous (less featureful) behavior to avoid unnecessary churn.
		`module m
		exclude (
			b.example/m v0.9.0
			a.example/m v1.0.0
			b.example/m v0.10.0
			c.example/m v1.1.0
			b.example/m v0.11.0
		)`,
		`module m
		exclude (
			a.example/m v1.0.0
			b.example/m v0.10.0
			b.example/m v0.11.0
			b.example/m v0.9.0
			c.example/m v1.1.0
		)
		`,
		true,
	},
}

var addRetractValidateVersionTests = []struct {
	desc      string
	path      string
	low, high string
	wantErr   string
}{
	{
		`blank_version`,
		`example.com/m`,
		``,
		``,
		`version "" invalid: must be of the form v1.2.3`,
	},
	{
		`missing prefix`,
		`example.com/m`,
		`1.0.0`,
		`1.0.0`,
		`version "1.0.0" invalid: must be of the form v1.2.3`,
	},
	{
		`non-canonical`,
		`example.com/m`,
		`v1.2`,
		`v1.2`,
		`version "v1.2" invalid: must be of the form v1.2.3`,
	},
	{
		`invalid range`,
		`example.com/m`,
		`v1.2.3`,
		`v1.3`,
		`version "v1.3" invalid: must be of the form v1.2.3`,
	},
	{
		`mismatched major`,
		`example.com/m/v2`,
		`v1.0.0`,
		`v1.0.0`,
		`version "v1.0.0" invalid: should be v2, not v1`,
	},
	{
		`missing +incompatible`,
		`example.com/m`,
		`v2.0.0`,
		`v2.0.0`,
		`version "v2.0.0" invalid: should be v2.0.0+incompatible (or module example.com/m/v2)`,
	},
}

var addExcludeValidateVersionTests = []struct {
	desc    string
	path    string
	version string
	wantErr string
}{
	{
		`blank version`,
		`example.com/m`,
		``,
		`version "" invalid: must be of the form v1.2.3`,
	},
	{
		`missing prefix`,
		`example.com/m`,
		`1.0.0`,
		`version "1.0.0" invalid: must be of the form v1.2.3`,
	},
	{
		`non-canonical`,
		`example.com/m`,
		`v1.2`,
		`version "v1.2" invalid: must be of the form v1.2.3`,
	},
	{
		`mismatched major`,
		`example.com/m/v2`,
		`v1.2.3`,
		`version "v1.2.3" invalid: should be v2, not v1`,
	},
	{
		`missing +incompatible`,
		`example.com/m`,
		`v2.3.4`,
		`version "v2.3.4" invalid: should be v2.3.4+incompatible (or module example.com/m/v2)`,
	},
}

var fixVersionTests = []struct {
	desc, in, want, wantErr string
	fix                     VersionFixer
}{
	{
		desc: `require`,
		in:   `require example.com/m 1.0.0`,
		want: `require example.com/m v1.0.0`,
		fix:  fixV,
	},
	{
		desc: `replace`,
		in:   `replace example.com/m 1.0.0 => example.com/m 1.1.0`,
		want: `replace example.com/m v1.0.0 => example.com/m v1.1.0`,
		fix:  fixV,
	},
	{
		desc:    `replace_version_in_path`,
		in:      `replace example.com/m@v1.0.0 => example.com/m@v1.1.0`,
		wantErr: `replacement module must match format 'path version', not 'path@version'`,
		fix:     fixV,
	},
	{
		desc:    `replace_version_in_later_path`,
		in:      `replace example.com/m => example.com/m@v1.1.0`,
		wantErr: `replacement module must match format 'path version', not 'path@version'`,
		fix:     fixV,
	},
	{
		desc: `exclude`,
		in:   `exclude example.com/m 1.0.0`,
		want: `exclude example.com/m v1.0.0`,
		fix:  fixV,
	},
	{
		desc: `retract_single`,
		in: `module example.com/m
		retract 1.0.0`,
		want: `module example.com/m
		retract v1.0.0`,
		fix: fixV,
	},
	{
		desc: `retract_interval`,
		in: `module example.com/m
		retract [1.0.0, 1.1.0]`,
		want: `module example.com/m
		retract [v1.0.0, v1.1.0]`,
		fix: fixV,
	},
	{
		desc:    `retract_nomod`,
		in:      `retract 1.0.0`,
		wantErr: `in:1: no module directive found, so retract cannot be used`,
		fix:     fixV,
	},
}

var modifyEmptyFilesTests = []struct {
	desc       string
	operations func(f *File)
	want       string
}{
	{
		desc: `addGoStmt`,
		operations: func(f *File) {
			f.AddGoStmt("1.20")
		},
		want: `go 1.20`,
	},
}

var addGodebugTests = []struct {
	desc  string
	in    string
	key   string
	value string
	out   string
}{
	{
		`existing`,
		`
		module m
		godebug key=old
		`,
		"key", "new",
		`
		module m
		godebug key=new
		`,
	},
	{
		`existing2`,
		`
		module m
		godebug (
			key=first // first
			other=first-a // first-a
		)
		godebug key=second // second
		godebug (
			key=third // third
			other=third-a // third-a
		)
		`,
		"key", "new",
		`
		module m

		godebug (
			key=new // first
			other=first-a// first-a
		)

		godebug other=third-a // third-a
		`,
	},
	{
		`new`,
		`
		module m
		godebug other=foo
		`,
		"key", "new",
		`
		module m
		godebug (
			other=foo
			key=new
		)
		`,
	},
	{
		`new2`,
		`
		module m
		godebug first=1
		godebug second=2
		`,
		"third", "3",
		`
		module m
		godebug first=1
		godebug (
			second=2
			third=3
		)
		`,
	},
}

var dropGodebugTests = []struct {
	desc string
	in   string
	key  string
	out  string
}{
	{
		`existing`,
		`
		module m
		godebug key=old
		`,
		"key",
		`
		module m
		`,
	},
	{
		`existing2`,
		`
		module m
		godebug (
			key=first // first
			other=first-a // first-a
		)
		godebug key=second // second
		godebug (
			key=third // third
			other=third-a // third-a
		)
		`,
		"key",
		`
		module m

		godebug other=first-a// first-a

		godebug other=third-a // third-a
		`,
	},
	{
		`new`,
		`
		module m
		godebug other=foo
		`,
		"key",
		`
		module m
		godebug other=foo
		`,
	},
}

var addToolTests = []struct {
	desc, in, path, want string
}{
	{
		`add_first`,
		`module example.com/m`,
		`example.com/tool/v1`,
		`module example.com/m
		tool example.com/tool/v1`,
	},
	{
		`sorted_correctly`,
		`module example.com/m
		tool example.com/tool2
		`,
		`example.com/tool1`,
		`module example.com/m
		tool (
			example.com/tool1
			example.com/tool2
		)`,
	},
	{
		`duplicates_ignored`,
		`module example.com/m
		tool example.com/tool1
		`,
		`example.com/tool1`,
		`module example.com/m
		tool example.com/tool1`,
	},
}

var dropToolTests = []struct {
	desc, in, path, want string
}{
	{
		`only`,
		`module example.com/m
		tool example.com/tool1`,
		`example.com/tool1`,
		`module example.com/m`,
	},
	{
		`parenthesized`,
		`module example.com/m
		tool (
			example.com/tool1
			example.com/tool2
		)`,
		`example.com/tool1`,
		`module example.com/m
		tool example.com/tool2`,
	},
	{
		`missing`,
		`module example.com/m
		tool (
		example.com/tool2
		)`,
		`example.com/tool1`,
		`module example.com/m
		tool example.com/tool2`,
	},
}

func fixV(path, version string) (string, error) {
	if path != "example.com/m" {
		return "", fmt.Errorf("module path must be example.com/m")
	}
	return "v" + version, nil
}

func TestAddRequire(t *testing.T) {
	for _, tt := range addRequireTests {
		t.Run(tt.desc, func(t *testing.T) {
			testEdit(t, tt.in, tt.out, true, func(f *File) error {
				err := f.AddRequire(tt.path, tt.vers)
				f.Cleanup()
				return err
			})
		})
	}
}

func TestAddGodebug(t *testing.T) {
	for _, tt := range addGodebugTests {
		t.Run(tt.desc, func(t *testing.T) {
			testEdit(t, tt.in, tt.out, true, func(f *File) error {
				err := f.AddGodebug(tt.key, tt.value)
				f.Cleanup()
				return err
			})
		})
	}
}

func TestSetRequire(t *testing.T) {
	for _, tt := range setRequireTests {
		t.Run(tt.desc, func(t *testing.T) {
			var mods []*Require
			for _, mod := range tt.mods {
				mods = append(mods, &Require{
					Mod: module.Version{
						Path:    mod.path,
						Version: mod.vers,
					},
					Indirect: mod.indirect,
				})
			}

			f := testEdit(t, tt.in, tt.out, true, func(f *File) error {
				f.SetRequire(mods)
				f.Cleanup()
				return nil
			})

			if len(f.Require) != len(mods) {
				t.Errorf("after Cleanup, len(Require) = %v; want %v", len(f.Require), len(mods))
			}
		})
	}
}

func TestSetRequireSeparateIndirect(t *testing.T) {
	for _, tt := range setRequireSeparateIndirectTests {
		t.Run(tt.desc, func(t *testing.T) {
			var mods []*Require
			for _, mod := range tt.mods {
				mods = append(mods, &Require{
					Mod: module.Version{
						Path:    mod.path,
						Version: mod.vers,
					},
					Indirect: mod.indirect,
				})
			}

			f := testEdit(t, tt.in, tt.out, true, func(f *File) error {
				f.SetRequireSeparateIndirect(mods)
				f.Cleanup()
				return nil
			})

			if len(f.Require) != len(mods) {
				t.Errorf("after Cleanup, len(Require) = %v; want %v", len(f.Require), len(mods))
			}
		})
	}
}

func TestAddGo(t *testing.T) {
	for _, tt := range addGoTests {
		t.Run(tt.desc, func(t *testing.T) {
			testEdit(t, tt.in, tt.out, true, func(f *File) error {
				return f.AddGoStmt(tt.version)
			})
		})
	}
}

func TestDropGo(t *testing.T) {
	for _, tt := range dropGoTests {
		t.Run(tt.desc, func(t *testing.T) {
			testEdit(t, tt.in, tt.out, true, func(f *File) error {
				f.DropGoStmt()
				return nil
			})
		})
	}
}

func TestAddToolchain(t *testing.T) {
	for _, tt := range addToolchainTests {
		t.Run(tt.desc, func(t *testing.T) {
			testEdit(t, tt.in, tt.out, true, func(f *File) error {
				return f.AddToolchainStmt(tt.version)
			})
		})
	}
}

func TestDropToolchain(t *testing.T) {
	for _, tt := range dropToolchainTests {
		t.Run(tt.desc, func(t *testing.T) {
			testEdit(t, tt.in, tt.out, true, func(f *File) error {
				f.DropToolchainStmt()
				return nil
			})
		})
	}
}

func TestDropGodebug(t *testing.T) {
	for _, tt := range dropGodebugTests {
		t.Run(tt.desc, func(t *testing.T) {
			testEdit(t, tt.in, tt.out, true, func(f *File) error {
				f.DropGodebug(tt.key)
				f.Cleanup()
				return nil
			})
		})
	}
}

func TestAddExclude(t *testing.T) {
	for _, tt := range addExcludeTests {
		t.Run(tt.desc, func(t *testing.T) {
			testEdit(t, tt.in, tt.out, true, func(f *File) error {
				return f.AddExclude(tt.path, tt.version)
			})
		})
	}
}

func TestAddRetract(t *testing.T) {
	for _, tt := range addRetractTests {
		t.Run(tt.desc, func(t *testing.T) {
			testEdit(t, tt.in, tt.out, true, func(f *File) error {
				return f.AddRetract(VersionInterval{Low: tt.low, High: tt.high}, tt.rationale)
			})
		})
	}
}

func TestDropRetract(t *testing.T) {
	for _, tt := range dropRetractTests {
		t.Run(tt.desc, func(t *testing.T) {
			testEdit(t, tt.in, tt.out, true, func(f *File) error {
				if err := f.DropRetract(VersionInterval{Low: tt.low, High: tt.high}); err != nil {
					return err
				}
				f.Cleanup()
				return nil
			})
		})
	}
}

func TestRetractRationale(t *testing.T) {
	for _, tt := range retractRationaleTests {
		t.Run(tt.desc, func(t *testing.T) {
			f, err := Parse("in", []byte(tt.in), nil)
			if err != nil {
				t.Fatal(err)
			}
			if len(f.Retract) != 1 {
				t.Fatalf("got %d retract directives; want 1", len(f.Retract))
			}
			if got := f.Retract[0].Rationale; got != tt.want {
				t.Errorf("got %q; want %q", got, tt.want)
			}
		})
	}
}

func TestModuleDeprecated(t *testing.T) {
	for _, tt := range moduleDeprecatedTests {
		t.Run(tt.desc, func(t *testing.T) {
			f, err := Parse("in", []byte(tt.in), nil)
			if err != nil {
				t.Fatal(err)
			}
			if f.Module.Deprecated != tt.want {
				t.Errorf("got %q; want %q", f.Module.Deprecated, tt.want)
			}
		})
	}
}

func TestSortBlocks(t *testing.T) {
	for _, tt := range sortBlocksTests {
		t.Run(tt.desc, func(t *testing.T) {
			testEdit(t, tt.in, tt.out, tt.strict, func(f *File) error {
				f.SortBlocks()
				return nil
			})
		})
	}
}

func testEdit(t *testing.T, in, want string, strict bool, transform func(f *File) error) *File {
	t.Helper()
	parse := Parse
	if !strict {
		parse = ParseLax
	}
	f, err := parse("in", []byte(in), nil)
	if err != nil {
		t.Fatal(err)
	}
	g, err := parse("out", []byte(want), nil)
	if err != nil {
		t.Fatal(err)
	}
	golden, err := g.Format()
	if err != nil {
		t.Fatal(err)
	}

	if err := transform(f); err != nil {
		t.Fatal(err)
	}
	out, err := f.Format()
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(out, golden) {
		t.Errorf("have:\n%s\nwant:\n%s", out, golden)
	}

	return f
}

func TestAddRetractValidateVersion(t *testing.T) {
	for _, tt := range addRetractValidateVersionTests {
		t.Run(tt.desc, func(t *testing.T) {
			f := new(File)
			if tt.path != "" {
				if err := f.AddModuleStmt(tt.path); err != nil {
					t.Fatal(err)
				}
				t.Logf("module %s", AutoQuote(tt.path))
			}
			interval := VersionInterval{Low: tt.low, High: tt.high}
			if err := f.AddRetract(interval, ``); err == nil || err.Error() != tt.wantErr {
				errStr := "<nil>"
				if err != nil {
					errStr = fmt.Sprintf("%#q", err)
				}
				t.Fatalf("f.AddRetract(%+v, ``) = %s\nwant %#q", interval, errStr, tt.wantErr)
			}
		})
	}
}

func TestAddExcludeValidateVersion(t *testing.T) {
	for _, tt := range addExcludeValidateVersionTests {
		t.Run(tt.desc, func(t *testing.T) {
			f, err := Parse("in", []byte("module m"), nil)
			if err != nil {
				t.Fatal(err)
			}
			if err = f.AddExclude(tt.path, tt.version); err == nil || err.Error() != tt.wantErr {
				errStr := "<nil>"
				if err != nil {
					errStr = fmt.Sprintf("%#q", err)
				}
				t.Fatalf("f.AddExclude(%q, %q) = %s\nwant %#q", tt.path, tt.version, errStr, tt.wantErr)
			}
		})
	}
}

func TestFixVersion(t *testing.T) {
	for _, tt := range fixVersionTests {
		t.Run(tt.desc, func(t *testing.T) {
			inFile, err := Parse("in", []byte(tt.in), tt.fix)
			if err != nil {
				if tt.wantErr == "" {
					t.Fatalf("unexpected error: %v", err)
				}
				if errMsg := err.Error(); !strings.Contains(errMsg, tt.wantErr) {
					t.Fatalf("got error %q; want error containing %q", errMsg, tt.wantErr)
				}
				return
			}
			got, err := inFile.Format()
			if err != nil {
				t.Fatal(err)
			}

			outFile, err := Parse("out", []byte(tt.want), nil)
			if err != nil {
				t.Fatal(err)
			}
			want, err := outFile.Format()
			if err != nil {
				t.Fatal(err)
			}

			if !bytes.Equal(got, want) {
				t.Fatalf("got:\n%s\nwant:\n%s", got, want)
			}
		})
	}
}

func TestAddOnEmptyFile(t *testing.T) {
	for _, tt := range modifyEmptyFilesTests {
		t.Run(tt.desc, func(t *testing.T) {
			f := &File{}
			tt.operations(f)

			expect, err := Parse("out", []byte(tt.want), nil)
			if err != nil {
				t.Fatal(err)
			}
			golden, err := expect.Format()
			if err != nil {
				t.Fatal(err)
			}
			got, err := f.Format()

			if err != nil {
				t.Fatal(err)
			}

			if !bytes.Equal(got, golden) {
				t.Fatalf("got:\n%s\nwant:\n%s", got, golden)
			}
		})
	}
}

func TestAddTool(t *testing.T) {
	for _, tt := range addToolTests {
		t.Run(tt.desc, func(t *testing.T) {
			inFile, err := Parse("in", []byte(tt.in), nil)
			if err != nil {
				t.Fatal(err)
			}
			if err := inFile.AddTool(tt.path); err != nil {
				t.Fatal(err)
			}
			inFile.Cleanup()
			got, err := inFile.Format()
			if err != nil {
				t.Fatal(err)
			}

			outFile, err := Parse("out", []byte(tt.want), nil)
			if err != nil {
				t.Fatal(err)
			}
			want, err := outFile.Format()
			if err != nil {
				t.Fatal(err)
			}
			if !bytes.Equal(got, want) {
				t.Fatalf("got:\n%s\nwant:\n%s", got, want)
			}
		})
	}
}

func TestDropTool(t *testing.T) {
	for _, tt := range dropToolTests {
		t.Run(tt.desc, func(t *testing.T) {
			inFile, err := Parse("in", []byte(tt.in), nil)
			if err != nil {
				t.Fatal(err)
			}
			if err := inFile.DropTool(tt.path); err != nil {
				t.Fatal(err)
			}
			inFile.Cleanup()
			got, err := inFile.Format()
			if err != nil {
				t.Fatal(err)
			}

			outFile, err := Parse("out", []byte(tt.want), nil)
			if err != nil {
				t.Fatal(err)
			}
			want, err := outFile.Format()
			if err != nil {
				t.Fatal(err)
			}
			if !bytes.Equal(got, want) {
				t.Fatalf("got:\n%s\nwant:\n%s", got, want)
			}
		})
	}
}
