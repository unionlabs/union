/*
 * Copyright 2017 Dgraph Labs, Inc. and Contributors
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
	"bytes"
	"fmt"
	"log"
	"math"
	"sync"

	"golang.org/x/net/trace"

	"github.com/dgraph-io/badger/v2/table"
	"github.com/dgraph-io/badger/v2/y"
)

type keyRange struct {
	left  []byte
	right []byte
	inf   bool
}

var infRange = keyRange{inf: true}

func (r keyRange) String() string {
	return fmt.Sprintf("[left=%x, right=%x, inf=%v]", r.left, r.right, r.inf)
}

func (r keyRange) equals(dst keyRange) bool {
	return bytes.Equal(r.left, dst.left) &&
		bytes.Equal(r.right, dst.right) &&
		r.inf == dst.inf
}

func (r keyRange) overlapsWith(dst keyRange) bool {
	if r.inf || dst.inf {
		return true
	}

	// If my left is greater than dst right, we have no overlap.
	if y.CompareKeys(r.left, dst.right) > 0 {
		return false
	}
	// If my right is less than dst left, we have no overlap.
	if y.CompareKeys(r.right, dst.left) < 0 {
		return false
	}
	// We have overlap.
	return true
}

func getKeyRange(tables ...*table.Table) keyRange {
	if len(tables) == 0 {
		return keyRange{}
	}
	smallest := tables[0].Smallest()
	biggest := tables[0].Biggest()
	for i := 1; i < len(tables); i++ {
		if y.CompareKeys(tables[i].Smallest(), smallest) < 0 {
			smallest = tables[i].Smallest()
		}
		if y.CompareKeys(tables[i].Biggest(), biggest) > 0 {
			biggest = tables[i].Biggest()
		}
	}

	// We pick all the versions of the smallest and the biggest key. Note that version zero would
	// be the rightmost key, considering versions are default sorted in descending order.
	return keyRange{
		left:  y.KeyWithTs(y.ParseKey(smallest), math.MaxUint64),
		right: y.KeyWithTs(y.ParseKey(biggest), 0),
	}
}

type levelCompactStatus struct {
	ranges  []keyRange
	delSize int64
}

func (lcs *levelCompactStatus) debug() string {
	var b bytes.Buffer
	for _, r := range lcs.ranges {
		b.WriteString(r.String())
	}
	return b.String()
}

func (lcs *levelCompactStatus) overlapsWith(dst keyRange) bool {
	for _, r := range lcs.ranges {
		if r.overlapsWith(dst) {
			return true
		}
	}
	return false
}

func (lcs *levelCompactStatus) remove(dst keyRange) bool {
	final := lcs.ranges[:0]
	var found bool
	for _, r := range lcs.ranges {
		if !r.equals(dst) {
			final = append(final, r)
		} else {
			found = true
		}
	}
	lcs.ranges = final
	return found
}

type compactStatus struct {
	sync.RWMutex
	levels []*levelCompactStatus
}

func (cs *compactStatus) toLog(tr trace.Trace) {
	cs.RLock()
	defer cs.RUnlock()

	tr.LazyPrintf("Compaction status:")
	for i, l := range cs.levels {
		if l.debug() == "" {
			continue
		}
		tr.LazyPrintf("[%d] %s", i, l.debug())
	}
}

func (cs *compactStatus) overlapsWith(level int, this keyRange) bool {
	cs.RLock()
	defer cs.RUnlock()

	thisLevel := cs.levels[level]
	return thisLevel.overlapsWith(this)
}

func (cs *compactStatus) delSize(l int) int64 {
	cs.RLock()
	defer cs.RUnlock()
	return cs.levels[l].delSize
}

type thisAndNextLevelRLocked struct{}

// compareAndAdd will check whether we can run this compactDef. That it doesn't overlap with any
// other running compaction. If it can be run, it would store this run in the compactStatus state.
func (cs *compactStatus) compareAndAdd(_ thisAndNextLevelRLocked, cd compactDef) bool {
	cs.Lock()
	defer cs.Unlock()

	level := cd.thisLevel.level

	y.AssertTruef(level < len(cs.levels)-1, "Got level %d. Max levels: %d", level, len(cs.levels))
	thisLevel := cs.levels[level]
	nextLevel := cs.levels[level+1]

	if thisLevel.overlapsWith(cd.thisRange) {
		return false
	}
	if nextLevel.overlapsWith(cd.nextRange) {
		return false
	}
	// Check whether this level really needs compaction or not. Otherwise, we'll end up
	// running parallel compactions for the same level.
	// Update: We should not be checking size here. Compaction priority already did the size checks.
	// Here we should just be executing the wish of others.

	thisLevel.ranges = append(thisLevel.ranges, cd.thisRange)
	nextLevel.ranges = append(nextLevel.ranges, cd.nextRange)
	thisLevel.delSize += cd.thisSize
	return true
}

func (cs *compactStatus) delete(cd compactDef) {
	cs.Lock()
	defer cs.Unlock()

	level := cd.thisLevel.level
	y.AssertTruef(level < len(cs.levels)-1, "Got level %d. Max levels: %d", level, len(cs.levels))

	thisLevel := cs.levels[level]
	nextLevel := cs.levels[level+1]

	thisLevel.delSize -= cd.thisSize
	found := thisLevel.remove(cd.thisRange)
	found = nextLevel.remove(cd.nextRange) && found

	if !found {
		this := cd.thisRange
		next := cd.nextRange
		fmt.Printf("Looking for: [%q, %q, %v] in this level.\n", this.left, this.right, this.inf)
		fmt.Printf("This Level:\n%s\n", thisLevel.debug())
		fmt.Println()
		fmt.Printf("Looking for: [%q, %q, %v] in next level.\n", next.left, next.right, next.inf)
		fmt.Printf("Next Level:\n%s\n", nextLevel.debug())
		log.Fatal("keyRange not found")
	}
}
