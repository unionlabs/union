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
	"encoding/hex"
	"io/ioutil"
	"math/rand"
	"sync/atomic"
	"time"

	"github.com/dgraph-io/badger/v2/table"
	"github.com/dgraph-io/badger/v2/y"
	"github.com/pkg/errors"
)

func (s *levelsController) validate() error {
	for _, l := range s.levels {
		if err := l.validate(); err != nil {
			return errors.Wrap(err, "Levels Controller")
		}
	}
	return nil
}

// Check does some sanity check on one level of data or in-memory index.
func (s *levelHandler) validate() error {
	if s.level == 0 {
		return nil
	}

	s.RLock()
	defer s.RUnlock()
	numTables := len(s.tables)
	for j := 1; j < numTables; j++ {
		if j >= len(s.tables) {
			return errors.Errorf("Level %d, j=%d numTables=%d", s.level, j, numTables)
		}

		if y.CompareKeys(s.tables[j-1].Biggest(), s.tables[j].Smallest()) >= 0 {
			return errors.Errorf(
				"Inter: Biggest(j-1) \n%s\n vs Smallest(j): \n%s\n: level=%d j=%d numTables=%d",
				hex.Dump(s.tables[j-1].Biggest()), hex.Dump(s.tables[j].Smallest()),
				s.level, j, numTables)
		}

		if y.CompareKeys(s.tables[j].Smallest(), s.tables[j].Biggest()) > 0 {
			return errors.Errorf(
				"Intra: \n%s\n vs \n%s\n: level=%d j=%d numTables=%d",
				hex.Dump(s.tables[j].Smallest()), hex.Dump(s.tables[j].Biggest()), s.level, j, numTables)
		}
	}
	return nil
}

// func (s *KV) debugPrintMore() { s.lc.debugPrintMore() }

// // debugPrintMore shows key ranges of each level.
// func (s *levelsController) debugPrintMore() {
// 	s.Lock()
// 	defer s.Unlock()
// 	for i := 0; i < s.kv.opt.MaxLevels; i++ {
// 		s.levels[i].debugPrintMore()
// 	}
// }

// func (s *levelHandler) debugPrintMore() {
// 	s.RLock()
// 	defer s.RUnlock()
// 	s.elog.Printf("Level %d:", s.level)
// 	for _, t := range s.tables {
// 		y.Printf(" [%s, %s]", t.Smallest(), t.Biggest())
// 	}
// 	y.Printf("\n")
// }

// reserveFileID reserves a unique file id.
func (s *levelsController) reserveFileID() uint64 {
	id := atomic.AddUint64(&s.nextFileID, 1)
	return id - 1
}

func getIDMap(dir string) map[uint64]struct{} {
	fileInfos, err := ioutil.ReadDir(dir)
	y.Check(err)
	idMap := make(map[uint64]struct{})
	for _, info := range fileInfos {
		if info.IsDir() {
			continue
		}
		fileID, ok := table.ParseFileID(info.Name())
		if !ok {
			continue
		}
		idMap[fileID] = struct{}{}
	}
	return idMap
}

func init() {
	rand.Seed(time.Now().UnixNano())
}
