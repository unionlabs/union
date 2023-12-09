// Copyright 2011 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import "github.com/cockroachdb/pebble/internal/base"

// Compare exports the base.Compare type.
type Compare = base.Compare

// Equal exports the base.Equal type.
type Equal = base.Equal

// AbbreviatedKey exports the base.AbbreviatedKey type.
type AbbreviatedKey = base.AbbreviatedKey

// Separator exports the base.Separator type.
type Separator = base.Separator

// Successor exports the base.Successor type.
type Successor = base.Successor

// Split exports the base.Split type.
type Split = base.Split

// Comparer exports the base.Comparer type.
type Comparer = base.Comparer

// DefaultComparer exports the base.DefaultComparer variable.
var DefaultComparer = base.DefaultComparer

// Merger exports the base.Merger type.
type Merger = base.Merger
