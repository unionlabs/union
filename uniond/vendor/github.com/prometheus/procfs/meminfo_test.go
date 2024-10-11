// Copyright 2019 The Prometheus Authors
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
	"reflect"
	"testing"
)

func TestMeminfo(t *testing.T) {
	expected := Meminfo{
		MemTotal:          newuint64(15666184),
		MemFree:           newuint64(440324),
		Buffers:           newuint64(1020128),
		Cached:            newuint64(12007640),
		SwapCached:        newuint64(0),
		Active:            newuint64(6761276),
		Inactive:          newuint64(6532708),
		ActiveAnon:        newuint64(267256),
		InactiveAnon:      newuint64(268),
		ActiveFile:        newuint64(6494020),
		InactiveFile:      newuint64(6532440),
		Unevictable:       newuint64(0),
		Mlocked:           newuint64(0),
		SwapTotal:         newuint64(0),
		SwapFree:          newuint64(0),
		Dirty:             newuint64(768),
		Writeback:         newuint64(0),
		AnonPages:         newuint64(266216),
		Mapped:            newuint64(44204),
		Shmem:             newuint64(1308),
		Slab:              newuint64(1807264),
		SReclaimable:      newuint64(1738124),
		SUnreclaim:        newuint64(69140),
		KernelStack:       newuint64(1616),
		PageTables:        newuint64(5288),
		NFSUnstable:       newuint64(0),
		Bounce:            newuint64(0),
		WritebackTmp:      newuint64(0),
		CommitLimit:       newuint64(7833092),
		CommittedAS:       newuint64(530844),
		VmallocTotal:      newuint64(34359738367),
		VmallocUsed:       newuint64(36596),
		VmallocChunk:      newuint64(34359637840),
		Percpu:            newuint64(26176),
		HardwareCorrupted: newuint64(0),
		AnonHugePages:     newuint64(12288),
		HugePagesTotal:    newuint64(0),
		HugePagesFree:     newuint64(0),
		HugePagesRsvd:     newuint64(0),
		HugePagesSurp:     newuint64(0),
		Hugepagesize:      newuint64(2048),
		DirectMap4k:       newuint64(91136),
		DirectMap2M:       newuint64(16039936),

		MemTotalBytes:          newuint64(16042172416),
		MemFreeBytes:           newuint64(450891776),
		BuffersBytes:           newuint64(1044611072),
		CachedBytes:            newuint64(12295823360),
		SwapCachedBytes:        newuint64(0),
		ActiveBytes:            newuint64(6923546624),
		InactiveBytes:          newuint64(6689492992),
		ActiveAnonBytes:        newuint64(273670144),
		InactiveAnonBytes:      newuint64(274432),
		ActiveFileBytes:        newuint64(6649876480),
		InactiveFileBytes:      newuint64(6689218560),
		UnevictableBytes:       newuint64(0),
		MlockedBytes:           newuint64(0),
		SwapTotalBytes:         newuint64(0),
		SwapFreeBytes:          newuint64(0),
		DirtyBytes:             newuint64(786432),
		WritebackBytes:         newuint64(0),
		AnonPagesBytes:         newuint64(272605184),
		MappedBytes:            newuint64(45264896),
		ShmemBytes:             newuint64(1339392),
		SlabBytes:              newuint64(1850638336),
		SReclaimableBytes:      newuint64(1779838976),
		SUnreclaimBytes:        newuint64(70799360),
		KernelStackBytes:       newuint64(1654784),
		PageTablesBytes:        newuint64(5414912),
		NFSUnstableBytes:       newuint64(0),
		BounceBytes:            newuint64(0),
		WritebackTmpBytes:      newuint64(0),
		CommitLimitBytes:       newuint64(8021086208),
		CommittedASBytes:       newuint64(543584256),
		VmallocTotalBytes:      newuint64(35184372087808),
		VmallocUsedBytes:       newuint64(37474304),
		VmallocChunkBytes:      newuint64(35184269148160),
		PercpuBytes:            newuint64(26804224),
		HardwareCorruptedBytes: newuint64(0),
		AnonHugePagesBytes:     newuint64(12582912),
		HugepagesizeBytes:      newuint64(2097152),
		DirectMap4kBytes:       newuint64(93323264),
		DirectMap2MBytes:       newuint64(16424894464),
	}

	have, err := getProcFixtures(t).Meminfo()
	if err != nil {
		t.Fatal(err)
	}

	if !reflect.DeepEqual(have, expected) {
		t.Logf("have: %+v", have)
		t.Logf("expected: %+v", expected)
		t.Errorf("structs are not equal")
	}
}
