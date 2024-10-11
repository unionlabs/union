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
	"testing"
)

func TestSlabInfo(t *testing.T) {
	slabs, err := getProcFixtures(t).SlabInfo()
	if err != nil {
		t.Fatal(err)
	}
	if numSlabs := len(slabs.Slabs); numSlabs != 300 {
		t.Errorf("expected 300 slabs, got %v", numSlabs)
	}
	if name := slabs.Slabs[0].Name; name != "pid_3" {
		t.Errorf("Expected slab name to be 'pid_3', got %v", name)
	}
	if objActive := slabs.Slabs[0].ObjActive; objActive != 375 {
		t.Errorf("Expected slab objects active to be 375, got %v", objActive)
	}
	if objNum := slabs.Slabs[0].ObjNum; objNum != 532 {
		t.Errorf("Expected slab objects number to be 532, got %v", objNum)
	}
	if objSize := slabs.Slabs[0].ObjSize; objSize != 576 {
		t.Errorf("Expected slab objects Size to be 576, got %+v", objSize)
	}
	if objPerSlab := slabs.Slabs[0].ObjPerSlab; objPerSlab != 28 {
		t.Errorf("Expected slab objects per slab to be 28, got %v", objPerSlab)
	}
	if pagesPerSlab := slabs.Slabs[0].PagesPerSlab; pagesPerSlab != 4 {
		t.Errorf("Expected pages per slab to be 4, got %v", pagesPerSlab)
	}
	if limit := slabs.Slabs[0].Limit; limit != 0 {
		t.Errorf("Expected limit to be 0, got %v", limit)
	}
	if batch := slabs.Slabs[0].Batch; batch != 0 {
		t.Errorf("Expected batch to be 0, got %v", batch)
	}
	if sharedFactor := slabs.Slabs[0].SharedFactor; sharedFactor != 0 {
		t.Errorf("Expected shared factor to be 0, got %v", sharedFactor)
	}
	if slabActive := slabs.Slabs[0].SlabActive; slabActive != 19 {
		t.Errorf("Expected slab active to be 19, got %v", slabActive)
	}
	if slabNum := slabs.Slabs[0].SlabNum; slabNum != 19 {
		t.Errorf("Expected slab num to be 19, got %v", slabNum)
	}
	if sharedAvail := slabs.Slabs[0].SharedAvail; sharedAvail != 0 {
		t.Errorf("Expected shared available to be 0, got %v", sharedAvail)
	}
}
