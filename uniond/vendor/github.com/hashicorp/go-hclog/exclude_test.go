// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MIT

package hclog

import (
	"regexp"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestExclude(t *testing.T) {
	t.Run("excludes by message", func(t *testing.T) {
		var em ExcludeByMessage
		em.Add("foo")
		em.Add("bar")

		assert.True(t, em.Exclude(Info, "foo"))
		assert.True(t, em.Exclude(Info, "bar"))
		assert.False(t, em.Exclude(Info, "qux"))
		assert.False(t, em.Exclude(Info, "foo qux"))
		assert.False(t, em.Exclude(Info, "qux bar"))
	})

	t.Run("excludes by prefix", func(t *testing.T) {
		ebp := ExcludeByPrefix("foo: ")

		assert.True(t, ebp.Exclude(Info, "foo: rocks"))
		assert.False(t, ebp.Exclude(Info, "foo"))
		assert.False(t, ebp.Exclude(Info, "qux foo: bar"))
	})

	t.Run("exclude by regexp", func(t *testing.T) {
		ebr := &ExcludeByRegexp{
			Regexp: regexp.MustCompile("(foo|bar)"),
		}

		assert.True(t, ebr.Exclude(Info, "foo"))
		assert.True(t, ebr.Exclude(Info, "bar"))
		assert.True(t, ebr.Exclude(Info, "foo qux"))
		assert.True(t, ebr.Exclude(Info, "qux bar"))
		assert.False(t, ebr.Exclude(Info, "qux"))
	})

	t.Run("excludes many funcs", func(t *testing.T) {
		ef := ExcludeFuncs{
			ExcludeByPrefix("foo: ").Exclude,
			ExcludeByPrefix("bar: ").Exclude,
		}

		assert.True(t, ef.Exclude(Info, "foo: rocks"))
		assert.True(t, ef.Exclude(Info, "bar: rocks"))
		assert.False(t, ef.Exclude(Info, "foo"))
		assert.False(t, ef.Exclude(Info, "qux foo: bar"))

	})
}
