// Copyright 2020 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package procfs

import (
	"bytes"
	"reflect"
	"testing"
)

func TestParseConntrackStat(t *testing.T) {
	var nfConntrackStat = []byte(`entries  searched found new invalid ignore delete delete_list insert insert_failed drop early_drop icmp_error  expect_new expect_create expect_delete search_restart
00000021  00000000 00000000 00000000 00000003 0000588a 00000000 00000000 00000000 00000000 00000000 00000000 00000000  00000000 00000000 00000000 00000000
00000021  00000000 00000000 00000000 00000002 000056a4 00000000 00000000 00000000 00000000 00000000 00000000 00000000  00000000 00000000 00000000 00000002
00000021  00000000 00000000 00000000 00000001 000058d4 00000000 00000000 00000000 00000000 00000000 00000000 00000000  00000000 00000000 00000000 00000001
00000021  00000000 00000000 00000000 0000002f 00005688 00000000 00000000 00000000 00000000 00000000 00000000 00000000  00000000 00000000 00000000 00000004
`)
	r := bytes.NewReader(nfConntrackStat)

	have, err := parseConntrackStat(r)
	if err != nil {
		t.Fatal(err)
	}

	want := []ConntrackStatEntry{
		ConntrackStatEntry{
			Entries:       33,
			Found:         0,
			Invalid:       3,
			Ignore:        22666,
			Insert:        0,
			InsertFailed:  0,
			Drop:          0,
			EarlyDrop:     0,
			SearchRestart: 0,
		},
		ConntrackStatEntry{
			Entries:       33,
			Found:         0,
			Invalid:       2,
			Ignore:        22180,
			Insert:        0,
			InsertFailed:  0,
			Drop:          0,
			EarlyDrop:     0,
			SearchRestart: 2,
		},
		ConntrackStatEntry{
			Entries:       33,
			Found:         0,
			Invalid:       1,
			Ignore:        22740,
			Insert:        0,
			InsertFailed:  0,
			Drop:          0,
			EarlyDrop:     0,
			SearchRestart: 1,
		},
		ConntrackStatEntry{
			Entries:       33,
			Found:         0,
			Invalid:       47,
			Ignore:        22152,
			Insert:        0,
			InsertFailed:  0,
			Drop:          0,
			EarlyDrop:     0,
			SearchRestart: 4,
		},
	}
	if !reflect.DeepEqual(want, have) {
		t.Errorf("want %v, have %v", want, have)
	}
}

func TestParseOldConntrackStat(t *testing.T) {
	var nfConntrackStat = []byte(`entries  searched found new invalid ignore delete delete_list insert insert_failed drop early_drop icmp_error  expect_new expect_create expect_delete
0000002b  0003159f 02e6786a 00142562 0001bf93 00e1a051 00142537 000b8fe0 000b900b 00000000 00000000 00000000 0001b46a  00000000 00000000 00000000
`)
	r := bytes.NewReader(nfConntrackStat)

	have, err := parseConntrackStat(r)
	if err != nil {
		t.Fatal(err)
	}

	want := []ConntrackStatEntry{
		ConntrackStatEntry{
			Entries:       43,
			Searched:      202143,
			Found:         48658538,
			New:           1320290,
			Invalid:       114579,
			Ignore:        14786641,
			Delete:        1320247,
			DeleteList:    757728,
			Insert:        757771,
			InsertFailed:  0,
			Drop:          0,
			EarlyDrop:     0,
			SearchRestart: 0,
		},
	}
	if !reflect.DeepEqual(want, have) {
		t.Errorf("want %v, have %v", want, have)
	}
}
