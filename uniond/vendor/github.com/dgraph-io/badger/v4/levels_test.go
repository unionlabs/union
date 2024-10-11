/*
 * Copyright 2019 Dgraph Labs, Inc. and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package badger

import (
	"fmt"
	"math"
	"math/rand"
	"os"
	"sort"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/dgraph-io/badger/v4/options"
	"github.com/dgraph-io/badger/v4/pb"
	"github.com/dgraph-io/badger/v4/table"
	"github.com/dgraph-io/badger/v4/y"
)

// createAndOpen creates a table with the given data and adds it to the given level.
func createAndOpen(db *DB, td []keyValVersion, level int) {
	opts := table.Options{
		BlockSize:          db.opt.BlockSize,
		BloomFalsePositive: db.opt.BloomFalsePositive,
		ChkMode:            options.NoVerification,
	}
	b := table.NewTableBuilder(opts)
	defer b.Close()

	// Add all keys and versions to the table.
	for _, item := range td {
		key := y.KeyWithTs([]byte(item.key), uint64(item.version))
		val := y.ValueStruct{Value: []byte(item.val), Meta: item.meta}
		b.Add(key, val, 0)
	}
	fname := table.NewFilename(db.lc.reserveFileID(), db.opt.Dir)
	tab, err := table.CreateTable(fname, b)
	if err != nil {
		panic(err)
	}
	if err := db.manifest.addChanges([]*pb.ManifestChange{
		newCreateChange(tab.ID(), level, 0, tab.CompressionType()),
	}); err != nil {
		panic(err)
	}
	db.lc.levels[level].Lock()
	// Add table to the given level.
	db.lc.levels[level].tables = append(db.lc.levels[level].tables, tab)
	db.lc.levels[level].Unlock()
}

type keyValVersion struct {
	key     string
	val     string
	version int
	meta    byte
}

func TestCheckOverlap(t *testing.T) {
	t.Run("overlap", func(t *testing.T) {
		// This test consists of one table on level 0 and one on level 1.
		// There is an overlap amongst the tables but there is no overlap
		// with rest of the levels.
		t.Run("same keys", func(t *testing.T) {
			runBadgerTest(t, nil, func(t *testing.T, db *DB) {
				l0 := []keyValVersion{{"foo", "bar", 3, 0}}
				l1 := []keyValVersion{{"foo", "bar", 2, 0}}
				createAndOpen(db, l0, 0)
				createAndOpen(db, l1, 1)

				// Level 0 should overlap with level 0 tables.
				require.True(t, db.lc.checkOverlap(db.lc.levels[0].tables, 0))
				// Level 1 should overlap with level 0 tables (they have the same keys).
				require.True(t, db.lc.checkOverlap(db.lc.levels[0].tables, 1))
				// Level 2 and 3 should not overlap with level 0 tables.
				require.False(t, db.lc.checkOverlap(db.lc.levels[0].tables, 2))
				require.False(t, db.lc.checkOverlap(db.lc.levels[1].tables, 2))
				require.False(t, db.lc.checkOverlap(db.lc.levels[0].tables, 3))
				require.False(t, db.lc.checkOverlap(db.lc.levels[1].tables, 3))

			})
		})
		t.Run("overlapping keys", func(t *testing.T) {
			runBadgerTest(t, nil, func(t *testing.T, db *DB) {
				l0 := []keyValVersion{{"a", "x", 1, 0}, {"b", "x", 1, 0}, {"foo", "bar", 3, 0}}
				l1 := []keyValVersion{{"foo", "bar", 2, 0}}
				createAndOpen(db, l0, 0)
				createAndOpen(db, l1, 1)

				// Level 0 should overlap with level 0 tables.
				require.True(t, db.lc.checkOverlap(db.lc.levels[0].tables, 0))
				require.True(t, db.lc.checkOverlap(db.lc.levels[1].tables, 1))
				// Level 1 should overlap with level 0 tables, "foo" key is common.
				require.True(t, db.lc.checkOverlap(db.lc.levels[0].tables, 1))
				// Level 2 and 3 should not overlap with level 0 tables.
				require.False(t, db.lc.checkOverlap(db.lc.levels[0].tables, 2))
				require.False(t, db.lc.checkOverlap(db.lc.levels[0].tables, 3))
			})
		})
	})
	t.Run("non-overlapping", func(t *testing.T) {
		runBadgerTest(t, nil, func(t *testing.T, db *DB) {
			l0 := []keyValVersion{{"a", "x", 1, 0}, {"b", "x", 1, 0}, {"c", "bar", 3, 0}}
			l1 := []keyValVersion{{"foo", "bar", 2, 0}}
			createAndOpen(db, l0, 0)
			createAndOpen(db, l1, 1)

			// Level 1 should not overlap with level 0 tables
			require.False(t, db.lc.checkOverlap(db.lc.levels[0].tables, 1))
			// Level 2 and 3 should not overlap with level 0 tables.
			require.False(t, db.lc.checkOverlap(db.lc.levels[0].tables, 2))
			require.False(t, db.lc.checkOverlap(db.lc.levels[0].tables, 3))
		})
	})
}

func getAllAndCheck(t *testing.T, db *DB, expected []keyValVersion) {
	require.NoError(t, db.View(func(txn *Txn) error {
		opt := DefaultIteratorOptions
		opt.AllVersions = true
		opt.InternalAccess = true
		it := txn.NewIterator(opt)
		defer it.Close()
		i := 0
		for it.Rewind(); it.Valid(); it.Next() {
			item := it.Item()
			v, err := item.ValueCopy(nil)
			require.NoError(t, err)
			// fmt.Printf("k: %s v: %d val: %s\n", item.key, item.Version(), v)
			require.Less(t, i, len(expected), "DB has more number of key than expected")
			expect := expected[i]
			require.Equal(t, expect.key, string(item.Key()), "expected key: %s actual key: %s",
				expect.key, item.Key())
			require.Equal(t, expect.val, string(v), "key: %s expected value: %s actual %s",
				item.key, expect.val, v)
			require.Equal(t, expect.version, int(item.Version()),
				"key: %s expected version: %d actual %d", item.key, expect.version, item.Version())
			require.Equal(t, expect.meta, item.meta,
				"key: %s expected meta: %d meta %d", item.key, expect.meta, item.meta)
			i++
		}
		require.Equal(t, len(expected), i, "keys examined should be equal to keys expected")
		return nil
	}))
}

func TestCompaction(t *testing.T) {
	// Disable compactions and keep single version of each key.
	opt := DefaultOptions("").WithNumCompactors(0).WithNumVersionsToKeep(1)
	opt.managedTxns = true
	t.Run("level 0 to level 1", func(t *testing.T) {
		runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
			l0 := []keyValVersion{{"foo", "bar", 3, 0}, {"fooz", "baz", 1, 0}}
			l01 := []keyValVersion{{"foo", "bar", 2, 0}}
			l1 := []keyValVersion{{"foo", "bar", 1, 0}}
			// Level 0 has table l0 and l01.
			createAndOpen(db, l0, 0)
			createAndOpen(db, l01, 0)
			// Level 1 has table l1.
			createAndOpen(db, l1, 1)

			// Set a high discard timestamp so that all the keys are below the discard timestamp.
			db.SetDiscardTs(10)

			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 3, 0}, {"foo", "bar", 2, 0},
				{"foo", "bar", 1, 0}, {"fooz", "baz", 1, 0},
			})
			cdef := compactDef{
				thisLevel: db.lc.levels[0],
				nextLevel: db.lc.levels[1],
				top:       db.lc.levels[0].tables,
				bot:       db.lc.levels[1].tables,
				t:         db.lc.levelTargets(),
			}
			cdef.t.baseLevel = 1
			require.NoError(t, db.lc.runCompactDef(-1, 0, cdef))
			// foo version 2 should be dropped after compaction.
			getAllAndCheck(t, db, []keyValVersion{{"foo", "bar", 3, 0}, {"fooz", "baz", 1, 0}})
		})
	})
	t.Run("level 0 to level 1 with duplicates", func(t *testing.T) {
		runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
			// We have foo version 3 on L0 because we gc'ed it.
			l0 := []keyValVersion{{"foo", "barNew", 3, 0}, {"fooz", "baz", 1, 0}}
			l01 := []keyValVersion{{"foo", "bar", 4, 0}}
			l1 := []keyValVersion{{"foo", "bar", 3, 0}}
			// Level 0 has table l0 and l01.
			createAndOpen(db, l0, 0)
			createAndOpen(db, l01, 0)
			// Level 1 has table l1.
			createAndOpen(db, l1, 1)

			// Set a high discard timestamp so that all the keys are below the discard timestamp.
			db.SetDiscardTs(10)

			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 4, 0}, {"foo", "barNew", 3, 0},
				{"fooz", "baz", 1, 0},
			})
			cdef := compactDef{
				thisLevel: db.lc.levels[0],
				nextLevel: db.lc.levels[1],
				top:       db.lc.levels[0].tables,
				bot:       db.lc.levels[1].tables,
				t:         db.lc.levelTargets(),
			}
			cdef.t.baseLevel = 1
			require.NoError(t, db.lc.runCompactDef(-1, 0, cdef))
			// foo version 3 (both) should be dropped after compaction.
			getAllAndCheck(t, db, []keyValVersion{{"foo", "bar", 4, 0}, {"fooz", "baz", 1, 0}})
		})
	})

	t.Run("level 0 to level 1 with lower overlap", func(t *testing.T) {
		runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
			l0 := []keyValVersion{{"foo", "bar", 4, 0}, {"fooz", "baz", 1, 0}}
			l01 := []keyValVersion{{"foo", "bar", 3, 0}}
			l1 := []keyValVersion{{"foo", "bar", 2, 0}}
			l2 := []keyValVersion{{"foo", "bar", 1, 0}}
			// Level 0 has table l0 and l01.
			createAndOpen(db, l0, 0)
			createAndOpen(db, l01, 0)
			// Level 1 has table l1.
			createAndOpen(db, l1, 1)
			// Level 2 has table l2.
			createAndOpen(db, l2, 2)

			// Set a high discard timestamp so that all the keys are below the discard timestamp.
			db.SetDiscardTs(10)

			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 4, 0}, {"foo", "bar", 3, 0}, {"foo", "bar", 2, 0},
				{"foo", "bar", 1, 0}, {"fooz", "baz", 1, 0},
			})
			cdef := compactDef{
				thisLevel: db.lc.levels[0],
				nextLevel: db.lc.levels[1],
				top:       db.lc.levels[0].tables,
				bot:       db.lc.levels[1].tables,
				t:         db.lc.levelTargets(),
			}
			cdef.t.baseLevel = 1
			require.NoError(t, db.lc.runCompactDef(-1, 0, cdef))
			// foo version 2 and version 1 should be dropped after compaction.
			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 4, 0}, {"foo", "bar", 1, 0}, {"fooz", "baz", 1, 0},
			})
		})
	})

	t.Run("level 1 to level 2", func(t *testing.T) {
		runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
			l1 := []keyValVersion{{"foo", "bar", 3, 0}, {"fooz", "baz", 1, 0}}
			l2 := []keyValVersion{{"foo", "bar", 2, 0}}
			createAndOpen(db, l1, 1)
			createAndOpen(db, l2, 2)

			// Set a high discard timestamp so that all the keys are below the discard timestamp.
			db.SetDiscardTs(10)

			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 3, 0}, {"foo", "bar", 2, 0}, {"fooz", "baz", 1, 0},
			})
			cdef := compactDef{
				thisLevel: db.lc.levels[1],
				nextLevel: db.lc.levels[2],
				top:       db.lc.levels[1].tables,
				bot:       db.lc.levels[2].tables,
				t:         db.lc.levelTargets(),
			}
			cdef.t.baseLevel = 2
			require.NoError(t, db.lc.runCompactDef(-1, 1, cdef))
			// foo version 2 should be dropped after compaction.
			getAllAndCheck(t, db, []keyValVersion{{"foo", "bar", 3, 0}, {"fooz", "baz", 1, 0}})
		})
	})

	t.Run("level 1 to level 2 with delete", func(t *testing.T) {
		t.Run("with overlap", func(t *testing.T) {
			runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
				l1 := []keyValVersion{{"foo", "bar", 3, bitDelete}, {"fooz", "baz", 1, bitDelete}}
				l2 := []keyValVersion{{"foo", "bar", 2, 0}}
				l3 := []keyValVersion{{"foo", "bar", 1, 0}}
				createAndOpen(db, l1, 1)
				createAndOpen(db, l2, 2)
				createAndOpen(db, l3, 3)

				// Set a high discard timestamp so that all the keys are below the discard timestamp.
				db.SetDiscardTs(10)

				getAllAndCheck(t, db, []keyValVersion{
					{"foo", "bar", 3, 1},
					{"foo", "bar", 2, 0},
					{"foo", "bar", 1, 0},
					{"fooz", "baz", 1, 1},
				})
				cdef := compactDef{
					thisLevel: db.lc.levels[1],
					nextLevel: db.lc.levels[2],
					top:       db.lc.levels[1].tables,
					bot:       db.lc.levels[2].tables,
					t:         db.lc.levelTargets(),
				}
				cdef.t.baseLevel = 2
				require.NoError(t, db.lc.runCompactDef(-1, 1, cdef))
				// foo bar version 2 should be dropped after compaction. fooz
				// baz version 1 will remain because overlap exists, which is
				// expected because `hasOverlap` is only checked once at the
				// beginning of `compactBuildTables` method.
				// everything from level 1 is now in level 2.
				getAllAndCheck(t, db, []keyValVersion{
					{"foo", "bar", 3, bitDelete},
					{"foo", "bar", 1, 0},
					{"fooz", "baz", 1, 1},
				})

				cdef = compactDef{
					thisLevel: db.lc.levels[2],
					nextLevel: db.lc.levels[3],
					top:       db.lc.levels[2].tables,
					bot:       db.lc.levels[3].tables,
					t:         db.lc.levelTargets(),
				}
				cdef.t.baseLevel = 3
				require.NoError(t, db.lc.runCompactDef(-1, 2, cdef))
				// everything should be removed now
				getAllAndCheck(t, db, []keyValVersion{})
			})
		})
		t.Run("with bottom overlap", func(t *testing.T) {
			runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
				l1 := []keyValVersion{{"foo", "bar", 3, bitDelete}}
				l2 := []keyValVersion{{"foo", "bar", 2, 0}, {"fooz", "baz", 2, bitDelete}}
				l3 := []keyValVersion{{"fooz", "baz", 1, 0}}
				createAndOpen(db, l1, 1)
				createAndOpen(db, l2, 2)
				createAndOpen(db, l3, 3)

				// Set a high discard timestamp so that all the keys are below the discard timestamp.
				db.SetDiscardTs(10)

				getAllAndCheck(t, db, []keyValVersion{
					{"foo", "bar", 3, bitDelete},
					{"foo", "bar", 2, 0},
					{"fooz", "baz", 2, bitDelete},
					{"fooz", "baz", 1, 0},
				})
				cdef := compactDef{
					thisLevel: db.lc.levels[1],
					nextLevel: db.lc.levels[2],
					top:       db.lc.levels[1].tables,
					bot:       db.lc.levels[2].tables,
					t:         db.lc.levelTargets(),
				}
				cdef.t.baseLevel = 2
				require.NoError(t, db.lc.runCompactDef(-1, 1, cdef))
				// the top table at L1 doesn't overlap L3, but the bottom table at L2
				// does, delete keys should not be removed.
				getAllAndCheck(t, db, []keyValVersion{
					{"foo", "bar", 3, bitDelete},
					{"fooz", "baz", 2, bitDelete},
					{"fooz", "baz", 1, 0},
				})
			})
		})
		t.Run("without overlap", func(t *testing.T) {
			runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
				l1 := []keyValVersion{{"foo", "bar", 3, bitDelete}, {"fooz", "baz", 1, bitDelete}}
				l2 := []keyValVersion{{"fooo", "barr", 2, 0}}
				createAndOpen(db, l1, 1)
				createAndOpen(db, l2, 2)

				// Set a high discard timestamp so that all the keys are below the discard timestamp.
				db.SetDiscardTs(10)

				getAllAndCheck(t, db, []keyValVersion{
					{"foo", "bar", 3, 1}, {"fooo", "barr", 2, 0}, {"fooz", "baz", 1, 1},
				})
				cdef := compactDef{
					thisLevel: db.lc.levels[1],
					nextLevel: db.lc.levels[2],
					top:       db.lc.levels[1].tables,
					bot:       db.lc.levels[2].tables,
					t:         db.lc.levelTargets(),
				}
				cdef.t.baseLevel = 2
				require.NoError(t, db.lc.runCompactDef(-1, 1, cdef))
				// foo version 2 should be dropped after compaction.
				getAllAndCheck(t, db, []keyValVersion{{"fooo", "barr", 2, 0}})
			})
		})
		t.Run("with splits", func(t *testing.T) {
			runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
				l1 := []keyValVersion{{"C", "bar", 3, bitDelete}}
				l21 := []keyValVersion{{"A", "bar", 2, 0}}
				l22 := []keyValVersion{{"B", "bar", 2, 0}}
				l23 := []keyValVersion{{"C", "bar", 2, 0}}
				l24 := []keyValVersion{{"D", "bar", 2, 0}}
				l3 := []keyValVersion{{"fooz", "baz", 1, 0}}
				createAndOpen(db, l1, 1)
				createAndOpen(db, l21, 2)
				createAndOpen(db, l22, 2)
				createAndOpen(db, l23, 2)
				createAndOpen(db, l24, 2)
				createAndOpen(db, l3, 3)

				// Set a high discard timestamp so that all the keys are below the discard timestamp.
				db.SetDiscardTs(10)

				getAllAndCheck(t, db, []keyValVersion{
					{"A", "bar", 2, 0},
					{"B", "bar", 2, 0},
					{"C", "bar", 3, bitDelete},
					{"C", "bar", 2, 0},
					{"D", "bar", 2, 0},
					{"fooz", "baz", 1, 0},
				})
				cdef := compactDef{
					thisLevel: db.lc.levels[1],
					nextLevel: db.lc.levels[2],
					top:       db.lc.levels[1].tables,
					bot:       db.lc.levels[2].tables,
					t:         db.lc.levelTargets(),
				}
				cdef.t.baseLevel = 2
				require.NoError(t, db.lc.runCompactDef(-1, 1, cdef))
				getAllAndCheck(t, db, []keyValVersion{
					{"A", "bar", 2, 0},
					{"B", "bar", 2, 0},
					{"D", "bar", 2, 0},
					{"fooz", "baz", 1, 0},
				})
			})
		})
	})
}

func TestCompactionTwoVersions(t *testing.T) {
	// Disable compactions and keep two versions of each key.
	opt := DefaultOptions("").WithNumCompactors(0).WithNumVersionsToKeep(2)
	opt.managedTxns = true
	t.Run("with overlap", func(t *testing.T) {
		runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
			l1 := []keyValVersion{{"foo", "bar", 3, 0}, {"fooz", "baz", 1, bitDelete}}
			l2 := []keyValVersion{{"foo", "bar", 2, 0}}
			l3 := []keyValVersion{{"foo", "bar", 1, 0}}
			createAndOpen(db, l1, 1)
			createAndOpen(db, l2, 2)
			createAndOpen(db, l3, 3)

			// Set a high discard timestamp so that all the keys are below the discard timestamp.
			db.SetDiscardTs(10)

			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 3, 0},
				{"foo", "bar", 2, 0},
				{"foo", "bar", 1, 0},
				{"fooz", "baz", 1, 1},
			})
			cdef := compactDef{
				thisLevel: db.lc.levels[1],
				nextLevel: db.lc.levels[2],
				top:       db.lc.levels[1].tables,
				bot:       db.lc.levels[2].tables,
				t:         db.lc.levelTargets(),
			}
			cdef.t.baseLevel = 2
			require.NoError(t, db.lc.runCompactDef(-1, 1, cdef))
			// Nothing should be dropped after compaction because number of
			// versions to keep is 2.
			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 3, 0},
				{"foo", "bar", 2, 0},
				{"foo", "bar", 1, 0},
				{"fooz", "baz", 1, 1},
			})

			cdef = compactDef{
				thisLevel: db.lc.levels[2],
				nextLevel: db.lc.levels[3],
				top:       db.lc.levels[2].tables,
				bot:       db.lc.levels[3].tables,
				t:         db.lc.levelTargets(),
			}
			cdef.t.baseLevel = 3
			require.NoError(t, db.lc.runCompactDef(-1, 2, cdef))
			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 3, 0},
				{"foo", "bar", 2, 0},
			})
		})
	})
}

func TestCompactionAllVersions(t *testing.T) {
	// Disable compactions and keep all versions of the each key.
	opt := DefaultOptions("").WithNumCompactors(0).WithNumVersionsToKeep(math.MaxInt32)
	opt.managedTxns = true
	t.Run("without overlap", func(t *testing.T) {
		runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
			l1 := []keyValVersion{{"foo", "bar", 3, 0}, {"fooz", "baz", 1, bitDelete}}
			l2 := []keyValVersion{{"foo", "bar", 2, 0}}
			l3 := []keyValVersion{{"foo", "bar", 1, 0}}
			createAndOpen(db, l1, 1)
			createAndOpen(db, l2, 2)
			createAndOpen(db, l3, 3)

			// Set a high discard timestamp so that all the keys are below the discard timestamp.
			db.SetDiscardTs(10)

			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 3, 0},
				{"foo", "bar", 2, 0},
				{"foo", "bar", 1, 0},
				{"fooz", "baz", 1, 1},
			})
			cdef := compactDef{
				thisLevel: db.lc.levels[1],
				nextLevel: db.lc.levels[2],
				top:       db.lc.levels[1].tables,
				bot:       db.lc.levels[2].tables,
				t:         db.lc.levelTargets(),
			}
			cdef.t.baseLevel = 2
			require.NoError(t, db.lc.runCompactDef(-1, 1, cdef))
			// Nothing should be dropped after compaction because all versions
			// should be kept.
			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 3, 0},
				{"foo", "bar", 2, 0},
				{"foo", "bar", 1, 0},
				{"fooz", "baz", 1, 1},
			})

			cdef = compactDef{
				thisLevel: db.lc.levels[2],
				nextLevel: db.lc.levels[3],
				top:       db.lc.levels[2].tables,
				bot:       db.lc.levels[3].tables,
				t:         db.lc.levelTargets(),
			}
			cdef.t.baseLevel = 3
			require.NoError(t, db.lc.runCompactDef(-1, 2, cdef))
			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 3, 0},
				{"foo", "bar", 2, 0},
				{"foo", "bar", 1, 0},
			})
		})
	})
	t.Run("without overlap", func(t *testing.T) {
		runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
			l1 := []keyValVersion{{"foo", "bar", 3, bitDelete}, {"fooz", "baz", 1, bitDelete}}
			l2 := []keyValVersion{{"fooo", "barr", 2, 0}}
			createAndOpen(db, l1, 1)
			createAndOpen(db, l2, 2)

			// Set a high discard timestamp so that all the keys are below the discard timestamp.
			db.SetDiscardTs(10)

			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 3, 1}, {"fooo", "barr", 2, 0}, {"fooz", "baz", 1, 1},
			})
			cdef := compactDef{
				thisLevel: db.lc.levels[1],
				nextLevel: db.lc.levels[2],
				top:       db.lc.levels[1].tables,
				bot:       db.lc.levels[2].tables,
				t:         db.lc.levelTargets(),
			}
			cdef.t.baseLevel = 2
			require.NoError(t, db.lc.runCompactDef(-1, 1, cdef))
			// foo version 2 should be dropped after compaction.
			getAllAndCheck(t, db, []keyValVersion{{"fooo", "barr", 2, 0}})
		})
	})
}

func TestDiscardTs(t *testing.T) {
	// Disable compactions and keep single version of each key.
	opt := DefaultOptions("").WithNumCompactors(0).WithNumVersionsToKeep(1)
	opt.managedTxns = true

	t.Run("all keys above discardTs", func(t *testing.T) {
		runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
			l0 := []keyValVersion{{"foo", "bar", 4, 0}, {"fooz", "baz", 3, 0}}
			l01 := []keyValVersion{{"foo", "bar", 3, 0}}
			l1 := []keyValVersion{{"foo", "bar", 2, 0}}
			// Level 0 has table l0 and l01.
			createAndOpen(db, l0, 0)
			createAndOpen(db, l01, 0)
			// Level 1 has table l1.
			createAndOpen(db, l1, 1)

			// Set dicardTs to 1. All the keys are above discardTs.
			db.SetDiscardTs(1)

			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 4, 0}, {"foo", "bar", 3, 0},
				{"foo", "bar", 2, 0}, {"fooz", "baz", 3, 0},
			})
			cdef := compactDef{
				thisLevel: db.lc.levels[0],
				nextLevel: db.lc.levels[1],
				top:       db.lc.levels[0].tables,
				bot:       db.lc.levels[1].tables,
				t:         db.lc.levelTargets(),
			}
			cdef.t.baseLevel = 1
			require.NoError(t, db.lc.runCompactDef(-1, 0, cdef))
			// No keys should be dropped.
			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 4, 0}, {"foo", "bar", 3, 0},
				{"foo", "bar", 2, 0}, {"fooz", "baz", 3, 0},
			})
		})
	})
	t.Run("some keys above discardTs", func(t *testing.T) {
		runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
			l0 := []keyValVersion{
				{"foo", "bar", 4, 0}, {"foo", "bar", 3, 0},
				{"foo", "bar", 2, 0}, {"fooz", "baz", 2, 0},
			}
			l1 := []keyValVersion{{"foo", "bbb", 1, 0}}
			createAndOpen(db, l0, 0)
			createAndOpen(db, l1, 1)

			// Set dicardTs to 3. foo2 and foo1 should be dropped.
			db.SetDiscardTs(3)

			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 4, 0}, {"foo", "bar", 3, 0}, {"foo", "bar", 2, 0},
				{"foo", "bbb", 1, 0}, {"fooz", "baz", 2, 0},
			})
			cdef := compactDef{
				thisLevel: db.lc.levels[0],
				nextLevel: db.lc.levels[1],
				top:       db.lc.levels[0].tables,
				bot:       db.lc.levels[1].tables,
				t:         db.lc.levelTargets(),
			}
			cdef.t.baseLevel = 1
			require.NoError(t, db.lc.runCompactDef(-1, 0, cdef))
			// foo1 and foo2 should be dropped.
			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 4, 0}, {"foo", "bar", 3, 0}, {"fooz", "baz", 2, 0},
			})
		})
	})
	t.Run("all keys below discardTs", func(t *testing.T) {
		runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
			l0 := []keyValVersion{{"foo", "bar", 4, 0}, {"fooz", "baz", 3, 0}}
			l01 := []keyValVersion{{"foo", "bar", 3, 0}}
			l1 := []keyValVersion{{"foo", "bar", 2, 0}}
			// Level 0 has table l0 and l01.
			createAndOpen(db, l0, 0)
			createAndOpen(db, l01, 0)
			// Level 1 has table l1.
			createAndOpen(db, l1, 1)

			// Set dicardTs to 10. All the keys are below discardTs.
			db.SetDiscardTs(10)

			getAllAndCheck(t, db, []keyValVersion{
				{"foo", "bar", 4, 0}, {"foo", "bar", 3, 0},
				{"foo", "bar", 2, 0}, {"fooz", "baz", 3, 0},
			})
			cdef := compactDef{
				thisLevel: db.lc.levels[0],
				nextLevel: db.lc.levels[1],
				top:       db.lc.levels[0].tables,
				bot:       db.lc.levels[1].tables,
				t:         db.lc.levelTargets(),
			}
			cdef.t.baseLevel = 1
			require.NoError(t, db.lc.runCompactDef(-1, 0, cdef))
			// Only one version of every key should be left.
			getAllAndCheck(t, db, []keyValVersion{{"foo", "bar", 4, 0}, {"fooz", "baz", 3, 0}})
		})
	})
}

// This is a test to ensure that the first entry with DiscardEarlierversion bit < DiscardTs
// is kept around (when numversionstokeep is infinite).
func TestDiscardFirstVersion(t *testing.T) {
	opt := DefaultOptions("")
	opt.NumCompactors = 0
	opt.NumVersionsToKeep = math.MaxInt32
	opt.managedTxns = true

	runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
		l0 := []keyValVersion{{"foo", "bar", 1, 0}}
		l01 := []keyValVersion{{"foo", "bar", 2, bitDiscardEarlierVersions}}
		l02 := []keyValVersion{{"foo", "bar", 3, 0}}
		l03 := []keyValVersion{{"foo", "bar", 4, 0}}
		l04 := []keyValVersion{{"foo", "bar", 9, 0}}
		l05 := []keyValVersion{{"foo", "bar", 10, bitDiscardEarlierVersions}}

		// Level 0 has all the tables.
		createAndOpen(db, l0, 0)
		createAndOpen(db, l01, 0)
		createAndOpen(db, l02, 0)
		createAndOpen(db, l03, 0)
		createAndOpen(db, l04, 0)
		createAndOpen(db, l05, 0)

		// Discard Time stamp is set to 7.
		db.SetDiscardTs(7)

		// Compact L0 to L1
		cdef := compactDef{
			thisLevel: db.lc.levels[0],
			nextLevel: db.lc.levels[1],
			top:       db.lc.levels[0].tables,
			bot:       db.lc.levels[1].tables,
			t:         db.lc.levelTargets(),
		}
		cdef.t.baseLevel = 1
		require.NoError(t, db.lc.runCompactDef(-1, 0, cdef))

		// - Version 10, 9 lie above version 7 so they should be there.
		// - Version 4, 3, 2 lie below the discardTs but they don't have the
		//   "bitDiscardEarlierVersions" versions set so they should not be removed because number
		//    of versions to keep is set to infinite.
		// - Version 1 is below DiscardTS and below the first "bitDiscardEarlierVersions"
		//   marker so IT WILL BE REMOVED.
		ExpectedKeys := []keyValVersion{
			{"foo", "bar", 10, bitDiscardEarlierVersions},
			{"foo", "bar", 9, 0},
			{"foo", "bar", 4, 0},
			{"foo", "bar", 3, 0},
			{"foo", "bar", 2, bitDiscardEarlierVersions}}

		getAllAndCheck(t, db, ExpectedKeys)
	})
}

// This test ensures we don't stall when L1's size is greater than opt.LevelOneSize.
// We should stall only when L0 tables more than the opt.NumLevelZeroTableStall.
func TestL1Stall(t *testing.T) {
	// TODO(ibrahim): Is this test still valid?
	t.Skip()
	opt := DefaultOptions("")
	// Disable all compactions.
	opt.NumCompactors = 0
	// Number of level zero tables.
	opt.NumLevelZeroTables = 3
	// Addition of new tables will stall if there are 4 or more L0 tables.
	opt.NumLevelZeroTablesStall = 4
	// Level 1 size is 10 bytes.
	opt.BaseLevelSize = 10

	runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
		// Level 0 has 4 tables.
		db.lc.levels[0].Lock()
		db.lc.levels[0].tables = []*table.Table{createEmptyTable(db), createEmptyTable(db),
			createEmptyTable(db), createEmptyTable(db)}
		db.lc.levels[0].Unlock()

		timeout := time.After(5 * time.Second)
		done := make(chan bool)

		// This is important. Set level 1 size more than the opt.LevelOneSize (we've set it to 10).
		db.lc.levels[1].totalSize = 100
		go func() {
			tab := createEmptyTable(db)
			require.NoError(t, db.lc.addLevel0Table(tab))
			require.NoError(t, tab.DecrRef())
			done <- true
		}()
		time.Sleep(time.Second)

		db.lc.levels[0].Lock()
		// Drop two tables from Level 0 so that addLevel0Table can make progress. Earlier table
		// count was 4 which is equal to L0 stall count.
		toDrop := db.lc.levels[0].tables[:2]
		require.NoError(t, decrRefs(toDrop))
		db.lc.levels[0].tables = db.lc.levels[0].tables[2:]
		db.lc.levels[0].Unlock()

		select {
		case <-timeout:
			t.Fatal("Test didn't finish in time")
		case <-done:
		}
	})
}

func createEmptyTable(db *DB) *table.Table {
	opts := table.Options{
		BloomFalsePositive: db.opt.BloomFalsePositive,
		ChkMode:            options.NoVerification,
	}
	b := table.NewTableBuilder(opts)
	defer b.Close()
	// Add one key so that we can open this table.
	b.Add(y.KeyWithTs([]byte("foo"), 1), y.ValueStruct{}, 0)

	// Open table in memory to avoid adding changes to manifest file.
	tab, err := table.OpenInMemoryTable(b.Finish(), db.lc.reserveFileID(), &opts)
	if err != nil {
		panic(err)
	}

	return tab
}

func TestL0Stall(t *testing.T) {
	// TODO(ibrahim): Is this test still valid?
	t.Skip()
	opt := DefaultOptions("")
	// Disable all compactions.
	opt.NumCompactors = 0
	// Number of level zero tables.
	opt.NumLevelZeroTables = 3
	// Addition of new tables will stall if there are 4 or more L0 tables.
	opt.NumLevelZeroTablesStall = 4

	runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
		db.lc.levels[0].Lock()
		// Add NumLevelZeroTableStall+1 number of tables to level 0. This would fill up level
		// zero and all new additions are expected to stall if L0 is in memory.
		for i := 0; i < opt.NumLevelZeroTablesStall+1; i++ {
			db.lc.levels[0].tables = append(db.lc.levels[0].tables, createEmptyTable(db))
		}
		db.lc.levels[0].Unlock()

		timeout := time.After(5 * time.Second)
		done := make(chan bool)

		go func() {
			tab := createEmptyTable(db)
			require.NoError(t, db.lc.addLevel0Table(tab))
			require.NoError(t, tab.DecrRef())
			done <- true
		}()
		// Let it stall for a second.
		time.Sleep(time.Second)

		select {
		case <-timeout:
			t.Log("Timeout triggered")
			// Mark this test as successful since L0 is in memory and the
			// addition of new table to L0 is supposed to stall.

			// Remove tables from level 0 so that the stalled
			// compaction can make progress. This does not have any
			// effect on the test. This is done so that the goroutine
			// stuck on addLevel0Table can make progress and end.
			db.lc.levels[0].Lock()
			db.lc.levels[0].tables = nil
			db.lc.levels[0].Unlock()
			<-done
		case <-done:
			// The test completed before 5 second timeout. Mark it as successful.
			t.Fatal("Test did not stall")
		}
	})
}

func TestLevelGet(t *testing.T) {
	createLevel := func(db *DB, level int, data [][]keyValVersion) {
		for _, v := range data {
			createAndOpen(db, v, level)
		}
	}
	type testData struct {
		name string
		// Keys on each level. keyValVersion[0] is the first table and so on.
		levelData map[int][][]keyValVersion
		expect    []keyValVersion
	}
	test := func(t *testing.T, ti testData, db *DB) {
		for level, data := range ti.levelData {
			createLevel(db, level, data)
		}
		for _, item := range ti.expect {
			key := y.KeyWithTs([]byte(item.key), uint64(item.version))
			vs, err := db.get(key)
			require.NoError(t, err)
			require.Equal(t, item.val, string(vs.Value), "key:%s ver:%d", item.key, item.version)
		}
	}
	tt := []testData{
		{
			"Normal",
			map[int][][]keyValVersion{
				0: { // Level 0 has 2 tables and each table has single key.
					{{"foo", "bar10", 10, 0}},
					{{"foo", "barSeven", 7, 0}},
				},
				1: { // Level 1 has 1 table with a single key.
					{{"foo", "bar", 1, 0}},
				},
			},
			[]keyValVersion{
				{"foo", "bar", 1, 0},
				{"foo", "barSeven", 7, 0},
				{"foo", "bar10", 10, 0},
				{"foo", "bar10", 11, 0},     // ver 11 doesn't exist so we should get bar10.
				{"foo", "barSeven", 9, 0},   // ver 9 doesn't exist so we should get barSeven.
				{"foo", "bar10", 100000, 0}, // ver doesn't exist so we should get bar10.
			},
		},
		{"after gc",
			map[int][][]keyValVersion{
				0: { // Level 0 has 3 tables and each table has single key.
					{{"foo", "barNew", 1, 0}}, // foo1 is above foo10 because of the GC.
					{{"foo", "bar10", 10, 0}},
					{{"foo", "barSeven", 7, 0}},
				},
				1: { // Level 1 has 1 table with a single key.
					{{"foo", "bar", 1, 0}},
				},
			},
			[]keyValVersion{
				{"foo", "barNew", 1, 0},
				{"foo", "barSeven", 7, 0},
				{"foo", "bar10", 10, 0},
				{"foo", "bar10", 11, 0}, // Should return biggest version.
			},
		},
		{"after two gc",
			map[int][][]keyValVersion{
				0: { // Level 0 has 4 tables and each table has single key.
					{{"foo", "barL0", 1, 0}}, // foo1 is above foo10 because of the GC.
					{{"foo", "bar10", 10, 0}},
					{{"foo", "barSeven", 7, 0}},
				},
				1: { // Level 1 has 1 table with a single key.
					// Level 1 also has a foo because it was moved twice during GC.
					{{"foo", "barL1", 1, 0}},
				},
				2: { // Level 1 has 1 table with a single key.
					{{"foo", "bar", 1, 0}},
				},
			},
			[]keyValVersion{
				{"foo", "barL0", 1, 0},
				{"foo", "barSeven", 7, 0},
				{"foo", "bar10", 10, 0},
				{"foo", "bar10", 11, 0}, // Should return biggest version.
			},
		},
	}
	for _, ti := range tt {
		t.Run(ti.name, func(t *testing.T) {
			runBadgerTest(t, nil, func(t *testing.T, db *DB) {
				test(t, ti, db)
			})

		})
	}
}

func TestKeyVersions(t *testing.T) {
	inMemoryOpt := DefaultOptions("").
		WithSyncWrites(false).
		WithInMemory(true)

	t.Run("disk", func(t *testing.T) {
		t.Run("small table", func(t *testing.T) {
			runBadgerTest(t, nil, func(t *testing.T, db *DB) {
				l0 := make([]keyValVersion, 0)
				for i := 0; i < 10; i++ {
					l0 = append(l0, keyValVersion{fmt.Sprintf("%05d", i), "foo", 1, 0})
				}
				createAndOpen(db, l0, 0)
				require.Equal(t, 2, len(db.Ranges(nil, 10000)))
			})
		})
		t.Run("medium table", func(t *testing.T) {
			runBadgerTest(t, nil, func(t *testing.T, db *DB) {
				l0 := make([]keyValVersion, 0)
				for i := 0; i < 1000; i++ {
					l0 = append(l0, keyValVersion{fmt.Sprintf("%05d", i), "foo", 1, 0})
				}
				createAndOpen(db, l0, 0)
				require.Equal(t, 8, len(db.Ranges(nil, 10000)))
			})
		})
		t.Run("large table", func(t *testing.T) {
			runBadgerTest(t, nil, func(t *testing.T, db *DB) {
				l0 := make([]keyValVersion, 0)
				for i := 0; i < 10000; i++ {
					l0 = append(l0, keyValVersion{fmt.Sprintf("%05d", i), "foo", 1, 0})
				}
				createAndOpen(db, l0, 0)
				require.Equal(t, 62, len(db.Ranges(nil, 10000)))
			})
		})
		t.Run("prefix", func(t *testing.T) {
			runBadgerTest(t, nil, func(t *testing.T, db *DB) {
				l0 := make([]keyValVersion, 0)
				for i := 0; i < 1000; i++ {
					l0 = append(l0, keyValVersion{fmt.Sprintf("%05d", i), "foo", 1, 0})
				}
				createAndOpen(db, l0, 0)
				require.Equal(t, 1, len(db.Ranges([]byte("a"), 10000)))
			})
		})
	})

	t.Run("in-memory", func(t *testing.T) {
		t.Run("small table", func(t *testing.T) {
			runBadgerTest(t, &inMemoryOpt, func(t *testing.T, db *DB) {
				writer := db.newWriteBatch(false)
				for i := 0; i < 10; i++ {
					require.NoError(t, writer.Set([]byte(fmt.Sprintf("%05d", i)), []byte("foo")))
				}
				require.NoError(t, writer.Flush())
				require.Equal(t, 2, len(db.Ranges(nil, 10000)))
			})
		})
		t.Run("large table", func(t *testing.T) {
			runBadgerTest(t, &inMemoryOpt, func(t *testing.T, db *DB) {
				writer := db.newWriteBatch(false)
				for i := 0; i < 100000; i++ {
					require.NoError(t, writer.Set([]byte(fmt.Sprintf("%05d", i)), []byte("foo")))
				}
				require.NoError(t, writer.Flush())
				require.Equal(t, 11, len(db.Ranges(nil, 10000)))
			})
		})
		t.Run("prefix", func(t *testing.T) {
			runBadgerTest(t, &inMemoryOpt, func(t *testing.T, db *DB) {
				writer := db.newWriteBatch(false)
				for i := 0; i < 10000; i++ {
					require.NoError(t, writer.Set([]byte(fmt.Sprintf("%05d", i)), []byte("foo")))
				}
				require.NoError(t, writer.Flush())
				require.Equal(t, 1, len(db.Ranges([]byte("a"), 10000)))
			})
		})
	})
}

func TestSameLevel(t *testing.T) {
	opt := DefaultOptions("")
	opt.NumCompactors = 0
	opt.NumVersionsToKeep = math.MaxInt32
	opt.managedTxns = true
	opt.LmaxCompaction = true
	runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
		l6 := []keyValVersion{
			{"A", "bar", 4, bitDiscardEarlierVersions}, {"A", "bar", 3, 0},
			{"A", "bar", 2, 0}, {"Afoo", "baz", 2, 0},
		}
		l61 := []keyValVersion{
			{"B", "bar", 4, bitDiscardEarlierVersions}, {"B", "bar", 3, 0},
			{"B", "bar", 2, 0}, {"Bfoo", "baz", 2, 0},
		}
		l62 := []keyValVersion{
			{"C", "bar", 4, bitDiscardEarlierVersions}, {"C", "bar", 3, 0},
			{"C", "bar", 2, 0}, {"Cfoo", "baz", 2, 0},
		}
		createAndOpen(db, l6, 6)
		createAndOpen(db, l61, 6)
		createAndOpen(db, l62, 6)
		require.NoError(t, db.lc.validate())

		getAllAndCheck(t, db, []keyValVersion{
			{"A", "bar", 4, bitDiscardEarlierVersions}, {"A", "bar", 3, 0},
			{"A", "bar", 2, 0}, {"Afoo", "baz", 2, 0},
			{"B", "bar", 4, bitDiscardEarlierVersions}, {"B", "bar", 3, 0},
			{"B", "bar", 2, 0}, {"Bfoo", "baz", 2, 0},
			{"C", "bar", 4, bitDiscardEarlierVersions}, {"C", "bar", 3, 0},
			{"C", "bar", 2, 0}, {"Cfoo", "baz", 2, 0},
		})

		cdef := compactDef{
			thisLevel: db.lc.levels[6],
			nextLevel: db.lc.levels[6],
			top:       []*table.Table{db.lc.levels[6].tables[0]},
			bot:       db.lc.levels[6].tables[1:],
			t:         db.lc.levelTargets(),
		}
		cdef.t.baseLevel = 1
		// Set dicardTs to 3. foo2 and foo1 should be dropped.
		db.SetDiscardTs(3)
		require.NoError(t, db.lc.runCompactDef(-1, 6, cdef))
		getAllAndCheck(t, db, []keyValVersion{
			{"A", "bar", 4, bitDiscardEarlierVersions}, {"A", "bar", 3, 0},
			{"A", "bar", 2, 0}, {"Afoo", "baz", 2, 0},
			{"B", "bar", 4, bitDiscardEarlierVersions}, {"B", "bar", 3, 0},
			{"B", "bar", 2, 0}, {"Bfoo", "baz", 2, 0},
			{"C", "bar", 4, bitDiscardEarlierVersions}, {"C", "bar", 3, 0},
			{"C", "bar", 2, 0}, {"Cfoo", "baz", 2, 0},
		})

		require.NoError(t, db.lc.validate())
		// Set dicardTs to 7.
		db.SetDiscardTs(7)
		cdef = compactDef{
			thisLevel: db.lc.levels[6],
			nextLevel: db.lc.levels[6],
			top:       []*table.Table{db.lc.levels[6].tables[0]},
			bot:       db.lc.levels[6].tables[1:],
			t:         db.lc.levelTargets(),
		}
		cdef.t.baseLevel = 1
		require.NoError(t, db.lc.runCompactDef(-1, 6, cdef))
		getAllAndCheck(t, db, []keyValVersion{
			{"A", "bar", 4, bitDiscardEarlierVersions}, {"Afoo", "baz", 2, 0},
			{"B", "bar", 4, bitDiscardEarlierVersions}, {"Bfoo", "baz", 2, 0},
			{"C", "bar", 4, bitDiscardEarlierVersions}, {"Cfoo", "baz", 2, 0}})
		require.NoError(t, db.lc.validate())
	})
}

func TestTableContainsPrefix(t *testing.T) {
	opts := table.Options{
		BlockSize:          4 * 1024,
		BloomFalsePositive: 0.01,
	}

	buildTable := func(keys []string) *table.Table {
		filename := fmt.Sprintf("%s%s%d.sst", os.TempDir(), string(os.PathSeparator), rand.Uint32())
		b := table.NewTableBuilder(opts)
		defer b.Close()

		v := []byte("value")
		sort.Slice(keys, func(i, j int) bool {
			return keys[i] < keys[j]
		})
		for _, k := range keys {
			b.Add(y.KeyWithTs([]byte(k), 1), y.ValueStruct{Value: v}, 0)
			b.Add(y.KeyWithTs([]byte(k), 2), y.ValueStruct{Value: v}, 0)
		}
		tbl, err := table.CreateTable(filename, b)
		require.NoError(t, err)
		return tbl
	}

	tbl := buildTable([]string{"key1", "key3", "key31", "key32", "key4"})
	defer func() { require.NoError(t, tbl.DecrRef()) }()

	require.True(t, containsPrefix(tbl, []byte("key")))
	require.True(t, containsPrefix(tbl, []byte("key1")))
	require.True(t, containsPrefix(tbl, []byte("key3")))
	require.True(t, containsPrefix(tbl, []byte("key32")))
	require.True(t, containsPrefix(tbl, []byte("key4")))

	require.False(t, containsPrefix(tbl, []byte("key0")))
	require.False(t, containsPrefix(tbl, []byte("key2")))
	require.False(t, containsPrefix(tbl, []byte("key323")))
	require.False(t, containsPrefix(tbl, []byte("key5")))
}

// Test that if a compaction fails during fill tables process, its tables are  cleaned up and we are able
// to do compaction on them again.
func TestFillTableCleanup(t *testing.T) {
	opt := DefaultOptions("")
	opt.managedTxns = true
	// Stop normal compactions from happening. They can take up the entire space causing the test to fail.
	opt.NumCompactors = 0
	runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
		opts := table.Options{
			BlockSize:          4 * 1024,
			BloomFalsePositive: 0.01,
		}
		buildTable := func(prefix byte) *table.Table {
			filename := table.NewFilename(db.lc.reserveFileID(), db.opt.Dir)
			b := table.NewTableBuilder(opts)
			defer b.Close()
			key := make([]byte, 100)
			val := make([]byte, 500)
			rand.Read(val)

			copy(key, []byte{prefix})
			count := uint64(40000)
			for i := count; i > 0; i-- {
				var meta byte
				if i == 0 {
					meta = bitDiscardEarlierVersions
				}
				b.AddStaleKey(y.KeyWithTs(key, i), y.ValueStruct{Meta: meta, Value: val}, 0)
			}
			tbl, err := table.CreateTable(filename, b)
			require.NoError(t, err)
			return tbl
		}

		buildLevel := func(level int, num_tab int) {
			lh := db.lc.levels[level]
			for i := byte(1); i < byte(num_tab); i++ {
				tab := buildTable(i)
				require.NoError(t, db.manifest.addChanges([]*pb.ManifestChange{
					newCreateChange(tab.ID(), level, 0, tab.CompressionType()),
				}))
				tab.CreatedAt = time.Now().Add(-10 * time.Hour)
				// Add table to the given level.
				lh.addTable(tab)
			}
			require.NoError(t, db.lc.validate())

			require.NotZero(t, lh.getTotalStaleSize())
		}

		level := 6
		buildLevel(level-1, 2)
		buildLevel(level, 2)

		db.SetDiscardTs(1 << 30)
		// Modify the target file size so that we can compact all tables at once.
		tt := db.lc.levelTargets()
		tt.fileSz[6] = 1 << 30
		prio := compactionPriority{level: 6, t: tt}

		cd := compactDef{
			compactorId: 0,
			span:        nil,
			p:           prio,
			t:           prio.t,
			thisLevel:   db.lc.levels[level-1],
			nextLevel:   db.lc.levels[level],
		}

		// Fill tables passes first.
		require.Equal(t, db.lc.fillTables(&cd), true)
		// Make sure that running compaction again fails, as the tables are being compacted.
		require.Equal(t, db.lc.fillTables(&cd), false)

		// Reset, to remove compaction being happening
		db.lc.cstatus.delete(cd)
		// Test that compaction should be able to run again on these tables.
		require.Equal(t, db.lc.fillTables(&cd), true)
	})
}

func TestStaleDataCleanup(t *testing.T) {
	opt := DefaultOptions("")
	opt.managedTxns = true
	opt.LmaxCompaction = true
	runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
		opts := table.Options{
			BlockSize:          4 * 1024,
			BloomFalsePositive: 0.01,
		}
		buildStaleTable := func(prefix byte) *table.Table {
			filename := table.NewFilename(db.lc.reserveFileID(), db.opt.Dir)
			b := table.NewTableBuilder(opts)
			defer b.Close()
			key := make([]byte, 100)
			val := make([]byte, 500)
			rand.Read(val)

			copy(key, []byte{prefix})
			count := uint64(40000)
			for i := count; i > 0; i-- {
				var meta byte
				if i == 0 {
					meta = bitDiscardEarlierVersions
				}
				b.AddStaleKey(y.KeyWithTs(key, i), y.ValueStruct{Meta: meta, Value: val}, 0)
			}
			tbl, err := table.CreateTable(filename, b)
			require.NoError(t, err)
			return tbl
		}

		level := 6
		lh := db.lc.levels[level]
		for i := byte(1); i < 5; i++ {
			tab := buildStaleTable(i)
			require.NoError(t, db.manifest.addChanges([]*pb.ManifestChange{
				newCreateChange(tab.ID(), level, 0, tab.CompressionType()),
			}))
			tab.CreatedAt = time.Now().Add(-10 * time.Hour)
			// Add table to the given level.
			lh.addTable(tab)
		}
		require.NoError(t, db.lc.validate())

		require.NotZero(t, lh.getTotalStaleSize())

		db.SetDiscardTs(1 << 30)
		// Modify the target file size so that we can compact all tables at once.
		tt := db.lc.levelTargets()
		tt.fileSz[6] = 1 << 30
		prio := compactionPriority{level: 6, t: tt}
		require.NoError(t, db.lc.doCompact(-1, prio))

		require.Zero(t, lh.getTotalStaleSize())

	})
}
