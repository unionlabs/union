// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"io"

	"github.com/cockroachdb/pebble/internal/base"
)

// Merge exports the base.Merge type.
type Merge = base.Merge

// Merger exports the base.Merger type.
type Merger = base.Merger

// ValueMerger exports the base.ValueMerger type.
type ValueMerger = base.ValueMerger

// DeletableValueMerger exports the base.DeletableValueMerger type.
type DeletableValueMerger = base.DeletableValueMerger

// DefaultMerger exports the base.DefaultMerger variable.
var DefaultMerger = base.DefaultMerger

func finishValueMerger(
	valueMerger ValueMerger, includesBase bool,
) (value []byte, needDelete bool, closer io.Closer, err error) {
	if valueMerger2, ok := valueMerger.(DeletableValueMerger); ok {
		value, needDelete, closer, err = valueMerger2.DeletableFinish(includesBase)
	} else {
		value, closer, err = valueMerger.Finish(includesBase)
	}
	return
}
