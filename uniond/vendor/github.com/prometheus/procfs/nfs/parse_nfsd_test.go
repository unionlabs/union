// Copyright 2018 The Prometheus Authors
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

package nfs_test

import (
	"reflect"
	"strings"
	"testing"

	"github.com/prometheus/procfs/nfs"
)

func TestNewNFSdServerRPCStats(t *testing.T) {
	tests := []struct {
		name    string
		content string
		stats   *nfs.ServerRPCStats
		invalid bool
	}{
		{
			name:    "invalid file",
			content: "invalid",
			invalid: true,
		}, {
			name: "good file, proc4ops 72",
			content: `rc 0 6 18622
fh 0 0 0 0 0
io 157286400 0
th 8 0 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000
ra 32 0 0 0 0 0 0 0 0 0 0 0
net 18628 0 18628 6
rpc 18628 0 0 0 0
proc2 18 2 69 0 0 4410 0 0 0 0 0 0 0 0 0 0 0 99 2
proc3 22 2 112 0 2719 111 0 0 0 0 0 0 0 0 0 0 0 27 216 0 2 1 0
proc4 2 2 10853
proc4ops 72 0 0 0 1098 2 0 0 0 0 8179 5896 0 0 0 0 5900 0 0 2 0 2 0 9609 0 2 150 1272 0 0 0 1236 0 0 0 0 3 3 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
wdeleg_getattr 16
`,
			stats: &nfs.ServerRPCStats{
				ReplyCache: nfs.ReplyCache{
					Hits:    0,
					Misses:  6,
					NoCache: 18622,
				},
				FileHandles: nfs.FileHandles{
					Stale:        0,
					TotalLookups: 0,
					AnonLookups:  0,
					DirNoCache:   0,
					NoDirNoCache: 0,
				},
				InputOutput: nfs.InputOutput{
					Read:  157286400,
					Write: 0,
				},
				Threads: nfs.Threads{
					Threads: 8,
					FullCnt: 0,
				},
				ReadAheadCache: nfs.ReadAheadCache{
					CacheSize:      32,
					CacheHistogram: []uint64{0, 0, 0, 0, 0, 0, 0, 0, 0, 0},
					NotFound:       0,
				},
				Network: nfs.Network{
					NetCount:   18628,
					UDPCount:   0,
					TCPCount:   18628,
					TCPConnect: 6,
				},
				ServerRPC: nfs.ServerRPC{
					RPCCount: 18628,
					BadCnt:   0,
					BadFmt:   0,
					BadAuth:  0,
					BadcInt:  0,
				},
				V2Stats: nfs.V2Stats{
					Null:     2,
					GetAttr:  69,
					SetAttr:  0,
					Root:     0,
					Lookup:   4410,
					ReadLink: 0,
					Read:     0,
					WrCache:  0,
					Write:    0,
					Create:   0,
					Remove:   0,
					Rename:   0,
					Link:     0,
					SymLink:  0,
					MkDir:    0,
					RmDir:    0,
					ReadDir:  99,
					FsStat:   2,
				},
				V3Stats: nfs.V3Stats{
					Null:        2,
					GetAttr:     112,
					SetAttr:     0,
					Lookup:      2719,
					Access:      111,
					ReadLink:    0,
					Read:        0,
					Write:       0,
					Create:      0,
					MkDir:       0,
					SymLink:     0,
					MkNod:       0,
					Remove:      0,
					RmDir:       0,
					Rename:      0,
					Link:        0,
					ReadDir:     27,
					ReadDirPlus: 216,
					FsStat:      0,
					FsInfo:      2,
					PathConf:    1,
					Commit:      0,
				},
				ServerV4Stats: nfs.ServerV4Stats{
					Null:     2,
					Compound: 10853,
				},
				V4Ops: nfs.V4Ops{
					Op0Unused:          0,
					Op1Unused:          0,
					Op2Future:          0,
					Access:             1098,
					Close:              2,
					Commit:             0,
					Create:             0,
					DelegPurge:         0,
					DelegReturn:        0,
					GetAttr:            8179,
					GetFH:              5896,
					Link:               0,
					Lock:               0,
					Lockt:              0,
					Locku:              0,
					Lookup:             5900,
					LookupRoot:         0,
					Nverify:            0,
					Open:               2,
					OpenAttr:           0,
					OpenConfirm:        2,
					OpenDgrd:           0,
					PutFH:              9609,
					PutPubFH:           0,
					PutRootFH:          2,
					Read:               150,
					ReadDir:            1272,
					ReadLink:           0,
					Remove:             0,
					Rename:             0,
					Renew:              1236,
					RestoreFH:          0,
					SaveFH:             0,
					SecInfo:            0,
					SetAttr:            0,
					SetClientID:        3,
					SetClientIDConfirm: 3,
					Verify:             0,
					Write:              0,
					RelLockOwner:       0,
				},
				WdelegGetattr: 16,
			},
		}, {
			name: "good file, proc4ops 40",
			content: `rc 0 25020854 19157796
fh 276 0 0 0 0
io 899844043 2470085989
th 1024 0 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000
ra 2048 4250593 118232 55926 31504 20253 13815 9875 7028 5546 3991 171551
net 44179842 1 44179026 3092
rpc 44177753 0 0 0 0
proc2 18 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
proc3 22 747 7259760 1383711 1570520 3464939 8436 4688207 21668847 1173194 6457 2127 172 213538 1253 556401 14950 1101 56245 90790 742 367 1989658
proc4 2 0 0
proc4ops 40 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
wdeleg_getattr 9`,

			stats: &nfs.ServerRPCStats{
				ReplyCache: nfs.ReplyCache{
					Hits:    0,
					Misses:  25020854,
					NoCache: 19157796,
				},
				FileHandles: nfs.FileHandles{
					Stale:        276,
					TotalLookups: 0,
					AnonLookups:  0,
					DirNoCache:   0,
					NoDirNoCache: 0,
				},
				InputOutput: nfs.InputOutput{
					Read:  899844043,
					Write: 2470085989,
				},
				Threads: nfs.Threads{
					Threads: 1024,
					FullCnt: 0,
				},
				ReadAheadCache: nfs.ReadAheadCache{
					CacheSize:      2048,
					CacheHistogram: []uint64{4250593, 118232, 55926, 31504, 20253, 13815, 9875, 7028, 5546, 3991},
					NotFound:       171551,
				},
				Network: nfs.Network{
					NetCount:   44179842,
					UDPCount:   1,
					TCPCount:   44179026,
					TCPConnect: 3092,
				},
				ServerRPC: nfs.ServerRPC{
					RPCCount: 44177753,
					BadCnt:   0,
					BadFmt:   0,
					BadAuth:  0,
					BadcInt:  0,
				},
				V2Stats: nfs.V2Stats{
					Null:     0,
					GetAttr:  0,
					SetAttr:  0,
					Root:     0,
					Lookup:   0,
					ReadLink: 0,
					Read:     0,
					WrCache:  0,
					Write:    0,
					Create:   0,
					Remove:   0,
					Rename:   0,
					Link:     0,
					SymLink:  0,
					MkDir:    0,
					RmDir:    0,
					ReadDir:  0,
					FsStat:   0,
				},
				V3Stats: nfs.V3Stats{
					Null:        747,
					GetAttr:     7259760,
					SetAttr:     1383711,
					Lookup:      1570520,
					Access:      3464939,
					ReadLink:    8436,
					Read:        4688207,
					Write:       21668847,
					Create:      1173194,
					MkDir:       6457,
					SymLink:     2127,
					MkNod:       172,
					Remove:      213538,
					RmDir:       1253,
					Rename:      556401,
					Link:        14950,
					ReadDir:     1101,
					ReadDirPlus: 56245,
					FsStat:      90790,
					FsInfo:      742,
					PathConf:    367,
					Commit:      1989658,
				},
				ServerV4Stats: nfs.ServerV4Stats{
					Null:     0,
					Compound: 0,
				},
				V4Ops: nfs.V4Ops{
					Op0Unused:          0,
					Op1Unused:          0,
					Op2Future:          0,
					Access:             0,
					Close:              0,
					Commit:             0,
					Create:             0,
					DelegPurge:         0,
					DelegReturn:        0,
					GetAttr:            0,
					GetFH:              0,
					Link:               0,
					Lock:               0,
					Lockt:              0,
					Locku:              0,
					Lookup:             0,
					LookupRoot:         0,
					Nverify:            0,
					Open:               0,
					OpenAttr:           0,
					OpenConfirm:        0,
					OpenDgrd:           0,
					PutFH:              0,
					PutPubFH:           0,
					PutRootFH:          0,
					Read:               0,
					ReadDir:            0,
					ReadLink:           0,
					Remove:             0,
					Rename:             0,
					Renew:              0,
					RestoreFH:          0,
					SaveFH:             0,
					SecInfo:            0,
					SetAttr:            0,
					SetClientID:        0,
					SetClientIDConfirm: 0,
					Verify:             0,
					Write:              0,
					RelLockOwner:       0,
				},
				WdelegGetattr: 9,
			},
		},
		{
			name: "good file, proc4ops 59",
			content: `rc 0 268 742119
fh 0 0 0 0 0
io 2476981939 0
th 8 0 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000
ra 32 30104 0 0 0 0 0 0 0 0 0 71174
net 742701 314 742393 10103960
rpc 742406 310 310 0 0
proc2 18 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
proc3 22 105 71158 0 175642 184711 17103 101277 0 0 0 0 0 0 0 0 0 0 8916 102 202 0 0
proc4 2 101 182991
proc4ops 59 0 0 0 18112 8341 0 0 0 3239 71595 11834 0 0 0 0 107097 0 0 8344 0 5100 0 181968 0 235 5735 4406 0 0 0 652 8342 8344 0 0 134 134 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
wdeleg_getattr 10`,
			stats: &nfs.ServerRPCStats{
				ReplyCache: nfs.ReplyCache{
					Hits:    0,
					Misses:  268,
					NoCache: 742119,
				},
				FileHandles: nfs.FileHandles{
					Stale:        0,
					TotalLookups: 0,
					AnonLookups:  0,
					DirNoCache:   0,
					NoDirNoCache: 0,
				},
				InputOutput: nfs.InputOutput{
					Read:  2476981939,
					Write: 0,
				},
				Threads: nfs.Threads{
					Threads: 8,
					FullCnt: 0,
				},
				ReadAheadCache: nfs.ReadAheadCache{
					CacheSize:      32,
					CacheHistogram: []uint64{30104, 0, 0, 0, 0, 0, 0, 0, 0, 0},
					NotFound:       71174,
				},
				Network: nfs.Network{
					NetCount:   742701,
					UDPCount:   314,
					TCPCount:   742393,
					TCPConnect: 10103960,
				},
				ServerRPC: nfs.ServerRPC{
					RPCCount: 742406,
					BadCnt:   310,
					BadFmt:   310,
					BadAuth:  0,
					BadcInt:  0,
				},
				V2Stats: nfs.V2Stats{
					Null:     0,
					GetAttr:  0,
					SetAttr:  0,
					Root:     0,
					Lookup:   0,
					ReadLink: 0,
					Read:     0,
					WrCache:  0,
					Write:    0,
					Create:   0,
					Remove:   0,
					Rename:   0,
					Link:     0,
					SymLink:  0,
					MkDir:    0,
					RmDir:    0,
					ReadDir:  0,
					FsStat:   0,
				},
				V3Stats: nfs.V3Stats{
					Null:        105,
					GetAttr:     71158,
					SetAttr:     0,
					Lookup:      175642,
					Access:      184711,
					ReadLink:    17103,
					Read:        101277,
					Write:       0,
					Create:      0,
					MkDir:       0,
					SymLink:     0,
					MkNod:       0,
					Remove:      0,
					RmDir:       0,
					Rename:      0,
					Link:        0,
					ReadDir:     0,
					ReadDirPlus: 8916,
					FsStat:      102,
					FsInfo:      202,
					PathConf:    0,
					Commit:      0,
				},
				ServerV4Stats: nfs.ServerV4Stats{
					Null:     101,
					Compound: 182991,
				},
				V4Ops: nfs.V4Ops{
					Op0Unused:          0,
					Op1Unused:          0,
					Op2Future:          0,
					Access:             18112,
					Close:              8341,
					Commit:             0,
					Create:             0,
					DelegPurge:         0,
					DelegReturn:        3239,
					GetAttr:            71595,
					GetFH:              11834,
					Link:               0,
					Lock:               0,
					Lockt:              0,
					Locku:              0,
					Lookup:             107097,
					LookupRoot:         0,
					Nverify:            0,
					Open:               8344,
					OpenAttr:           0,
					OpenConfirm:        5100,
					OpenDgrd:           0,
					PutFH:              181968,
					PutPubFH:           0,
					PutRootFH:          235,
					Read:               5735,
					ReadDir:            4406,
					ReadLink:           0,
					Remove:             0,
					Rename:             0,
					Renew:              652,
					RestoreFH:          8342,
					SaveFH:             8344,
					SecInfo:            0,
					SetAttr:            0,
					SetClientID:        134,
					SetClientIDConfirm: 134,
					Verify:             0,
					Write:              0,
					RelLockOwner:       0,
				},
				WdelegGetattr: 10,
			},
		}, {
			name: "good file, proc4ops 39",
			content: `rc 0 25020854 19157796
fh 276 0 0 0 0
io 899844043 2470085989
th 1024 0 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000
ra 2048 4250593 118232 55926 31504 20253 13815 9875 7028 5546 3991 171551
net 44179842 1 44179026 3092
rpc 44177753 0 0 0 0
proc2 18 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
proc3 22 747 7259760 1383711 1570520 3464939 8436 4688207 21668847 1173194 6457 2127 172 213538 1253 556401 14950 1101 56245 90790 742 367 1989658
proc4 2 0 0
proc4ops 39 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 39
wdeleg_getattr 765432`,
			stats: &nfs.ServerRPCStats{
				ReplyCache: nfs.ReplyCache{
					Hits:    0,
					Misses:  25020854,
					NoCache: 19157796,
				},
				FileHandles: nfs.FileHandles{
					Stale:        276,
					TotalLookups: 0,
					AnonLookups:  0,
					DirNoCache:   0,
					NoDirNoCache: 0,
				},
				InputOutput: nfs.InputOutput{
					Read:  899844043,
					Write: 2470085989,
				},
				Threads: nfs.Threads{
					Threads: 1024,
					FullCnt: 0,
				},
				ReadAheadCache: nfs.ReadAheadCache{
					CacheSize:      2048,
					CacheHistogram: []uint64{4250593, 118232, 55926, 31504, 20253, 13815, 9875, 7028, 5546, 3991},
					NotFound:       171551,
				},
				Network: nfs.Network{
					NetCount:   44179842,
					UDPCount:   1,
					TCPCount:   44179026,
					TCPConnect: 3092,
				},
				ServerRPC: nfs.ServerRPC{
					RPCCount: 44177753,
					BadCnt:   0,
					BadFmt:   0,
					BadAuth:  0,
					BadcInt:  0,
				},
				V2Stats: nfs.V2Stats{
					Null:     0,
					GetAttr:  0,
					SetAttr:  0,
					Root:     0,
					Lookup:   0,
					ReadLink: 0,
					Read:     0,
					WrCache:  0,
					Write:    0,
					Create:   0,
					Remove:   0,
					Rename:   0,
					Link:     0,
					SymLink:  0,
					MkDir:    0,
					RmDir:    0,
					ReadDir:  0,
					FsStat:   0,
				},
				V3Stats: nfs.V3Stats{
					Null:        747,
					GetAttr:     7259760,
					SetAttr:     1383711,
					Lookup:      1570520,
					Access:      3464939,
					ReadLink:    8436,
					Read:        4688207,
					Write:       21668847,
					Create:      1173194,
					MkDir:       6457,
					SymLink:     2127,
					MkNod:       172,
					Remove:      213538,
					RmDir:       1253,
					Rename:      556401,
					Link:        14950,
					ReadDir:     1101,
					ReadDirPlus: 56245,
					FsStat:      90790,
					FsInfo:      742,
					PathConf:    367,
					Commit:      1989658,
				},
				ServerV4Stats: nfs.ServerV4Stats{
					Null:     0,
					Compound: 0,
				},
				V4Ops: nfs.V4Ops{
					Op0Unused:          0,
					Op1Unused:          0,
					Op2Future:          0,
					Access:             0,
					Close:              0,
					Commit:             0,
					Create:             0,
					DelegPurge:         0,
					DelegReturn:        0,
					GetAttr:            0,
					GetFH:              0,
					Link:               0,
					Lock:               0,
					Lockt:              0,
					Locku:              0,
					Lookup:             0,
					LookupRoot:         0,
					Nverify:            0,
					Open:               0,
					OpenAttr:           0,
					OpenConfirm:        0,
					OpenDgrd:           0,
					PutFH:              0,
					PutPubFH:           0,
					PutRootFH:          0,
					Read:               0,
					ReadDir:            0,
					ReadLink:           0,
					Remove:             0,
					Rename:             0,
					Renew:              0,
					RestoreFH:          0,
					SaveFH:             0,
					SecInfo:            0,
					SetAttr:            0,
					SetClientID:        0,
					SetClientIDConfirm: 0,
					Verify:             0,
					Write:              39,
					RelLockOwner:       0,
				},
				WdelegGetattr: 765432,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			stats, err := nfs.ParseServerRPCStats(strings.NewReader(tt.content))

			if tt.invalid && err == nil {
				t.Fatal("expected an error, but none occurred")
			}
			if !tt.invalid && err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			if want, have := tt.stats, stats; !reflect.DeepEqual(want, have) {
				t.Fatalf("unexpected NFS stats:\nwant:\n%v\nhave:\n%v", want, have)
			}
		})
	}
}
