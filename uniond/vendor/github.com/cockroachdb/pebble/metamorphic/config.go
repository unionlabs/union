// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package metamorphic

import "github.com/cockroachdb/pebble/internal/randvar"

type opType int

const (
	batchAbort opType = iota
	batchCommit
	dbCheckpoint
	dbClose
	dbCompact
	dbFlush
	dbRatchetFormatMajorVersion
	dbRestart
	iterClose
	iterFirst
	iterLast
	iterNext
	iterNextWithLimit
	iterNextPrefix
	iterCanSingleDelete
	iterPrev
	iterPrevWithLimit
	iterSeekGE
	iterSeekGEWithLimit
	iterSeekLT
	iterSeekLTWithLimit
	iterSeekPrefixGE
	iterSetBounds
	iterSetOptions
	newBatch
	newIndexedBatch
	newIter
	newIterUsingClone
	newSnapshot
	readerGet
	snapshotClose
	writerApply
	writerDelete
	writerDeleteRange
	writerIngest
	writerMerge
	writerRangeKeyDelete
	writerRangeKeySet
	writerRangeKeyUnset
	writerSet
	writerSingleDelete
)

type config struct {
	// Weights for the operation mix to generate. ops[i] corresponds to the
	// weight for opType(i).
	ops []int

	// newPrefix configures the probability that when generating a new user key,
	// the generated key uses a new key prefix rather than an existing prefix
	// with a suffix.
	newPrefix float64
	// writeSuffixDist defines the distribution of key suffixes during writing.
	// It's a dynamic randvar to roughly emulate workloads with MVCC timestamps,
	// skewing towards most recent timestamps.
	writeSuffixDist randvar.Dynamic

	// TODO(peter): unimplemented
	// keyDist        randvar.Dynamic
	// keySizeDist    randvar.Static
	// valueSizeDist  randvar.Static
	// updateFrac     float64
	// lowerBoundFrac float64
	// upperBoundFrac float64
}

func (c config) withNewPrefixProbability(p float64) config {
	c.newPrefix = p
	return c
}

func (c config) withOpWeight(op opType, weight int) config {
	c.ops[op] = weight
	return c
}

var presetConfigs = []config{
	defaultConfig(),
	// Generate a configuration that helps exercise code paths dependent on many
	// versions of keys with the same prefixes. The default configuration does
	// not tend to generate many versions of the same key. Additionally, its
	// relatively high weight for deletion write operations makes it less likely
	// that we'll accumulate enough versions to exercise some code paths (eg,
	// see #2921 which requires >16 SETs for versions of the same prefix to
	// reside in a single block to exercise the code path).
	//
	// To encourage generation of many versions of the same keys, generate a new
	// prefix only 4% of the time when generating a new key. The remaining 96%
	// of new key generations will use an existing prefix. To keep the size of
	// the database growing, we also reduce the probability of delete write
	// operations significantly.
	defaultConfig().
		withNewPrefixProbability(0.04).
		withOpWeight(writerDeleteRange, 1).
		withOpWeight(writerDelete, 5).
		withOpWeight(writerSingleDelete, 5).
		withOpWeight(writerMerge, 0),
}

func defaultConfig() config {
	return config{
		// dbClose is not in this list since it is deterministically generated once, at the end of the test.
		ops: []int{
			batchAbort:                  5,
			batchCommit:                 5,
			dbCheckpoint:                1,
			dbCompact:                   1,
			dbFlush:                     2,
			dbRatchetFormatMajorVersion: 1,
			dbRestart:                   2,
			iterClose:                   5,
			iterFirst:                   100,
			iterLast:                    100,
			iterNext:                    100,
			iterNextWithLimit:           20,
			iterNextPrefix:              20,
			iterCanSingleDelete:         20,
			iterPrev:                    100,
			iterPrevWithLimit:           20,
			iterSeekGE:                  100,
			iterSeekGEWithLimit:         20,
			iterSeekLT:                  100,
			iterSeekLTWithLimit:         20,
			iterSeekPrefixGE:            100,
			iterSetBounds:               100,
			iterSetOptions:              10,
			newBatch:                    5,
			newIndexedBatch:             5,
			newIter:                     10,
			newIterUsingClone:           5,
			newSnapshot:                 10,
			readerGet:                   100,
			snapshotClose:               10,
			writerApply:                 10,
			writerDelete:                100,
			writerDeleteRange:           50,
			writerIngest:                100,
			writerMerge:                 100,
			writerRangeKeySet:           10,
			writerRangeKeyUnset:         10,
			writerRangeKeyDelete:        5,
			writerSet:                   100,
			writerSingleDelete:          50,
		},
		// Use a new prefix 75% of the time (and 25% of the time use an existing
		// prefix with an alternative suffix).
		newPrefix: 0.75,
		// Use a skewed distribution of suffixes to mimic MVCC timestamps. The
		// range will be widened whenever a suffix is found to already be in use
		// for a particular prefix.
		writeSuffixDist: mustDynamic(randvar.NewSkewedLatest(0, 1, 0.99)),
	}
}

func mustDynamic(dyn randvar.Dynamic, err error) randvar.Dynamic {
	if err != nil {
		panic(err)
	}
	return dyn
}
