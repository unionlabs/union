/*
 *
 * Copyright 2020 gRPC authors.
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

package load

import (
	"fmt"
	"sort"
	"sync"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
)

var (
	dropCategories = []string{"drop_for_real", "drop_for_fun"}
	localities     = []string{"locality-A", "locality-B"}
	errTest        = fmt.Errorf("test error")
)

// rpcData wraps the rpc counts and load data to be pushed to the store.
type rpcData struct {
	start, success, failure int
	serverData              map[string]float64 // Will be reported with successful RPCs.
}

// TestDrops spawns a bunch of goroutines which report drop data. After the
// goroutines have exited, the test dumps the stats from the Store and makes
// sure they are as expected.
func TestDrops(t *testing.T) {
	var (
		drops = map[string]int{
			dropCategories[0]: 30,
			dropCategories[1]: 40,
			"":                10,
		}
		wantStoreData = &Data{
			TotalDrops: 80,
			Drops: map[string]uint64{
				dropCategories[0]: 30,
				dropCategories[1]: 40,
			},
		}
	)

	ls := perClusterStore{}
	var wg sync.WaitGroup
	for category, count := range drops {
		for i := 0; i < count; i++ {
			wg.Add(1)
			go func(c string) {
				ls.CallDropped(c)
				wg.Done()
			}(category)
		}
	}
	wg.Wait()

	gotStoreData := ls.stats()
	if diff := cmp.Diff(wantStoreData, gotStoreData, cmpopts.EquateEmpty(), cmpopts.IgnoreFields(Data{}, "ReportInterval")); diff != "" {
		t.Errorf("store.stats() returned unexpected diff (-want +got):\n%s", diff)
	}
}

// TestLocalityStats spawns a bunch of goroutines which report rpc and load
// data. After the goroutines have exited, the test dumps the stats from the
// Store and makes sure they are as expected.
func TestLocalityStats(t *testing.T) {
	var (
		localityData = map[string]rpcData{
			localities[0]: {
				start:      40,
				success:    20,
				failure:    10,
				serverData: map[string]float64{"net": 1, "disk": 2, "cpu": 3, "mem": 4},
			},
			localities[1]: {
				start:      80,
				success:    40,
				failure:    20,
				serverData: map[string]float64{"net": 1, "disk": 2, "cpu": 3, "mem": 4},
			},
		}
		wantStoreData = &Data{
			LocalityStats: map[string]LocalityData{
				localities[0]: {
					RequestStats: RequestData{
						Succeeded:  20,
						Errored:    10,
						InProgress: 10,
						Issued:     40,
					},
					LoadStats: map[string]ServerLoadData{
						"net":  {Count: 20, Sum: 20},
						"disk": {Count: 20, Sum: 40},
						"cpu":  {Count: 20, Sum: 60},
						"mem":  {Count: 20, Sum: 80},
					},
				},
				localities[1]: {
					RequestStats: RequestData{
						Succeeded:  40,
						Errored:    20,
						InProgress: 20,
						Issued:     80,
					},
					LoadStats: map[string]ServerLoadData{
						"net":  {Count: 40, Sum: 40},
						"disk": {Count: 40, Sum: 80},
						"cpu":  {Count: 40, Sum: 120},
						"mem":  {Count: 40, Sum: 160},
					},
				},
			},
		}
	)

	ls := perClusterStore{}
	var wg sync.WaitGroup
	for locality, data := range localityData {
		wg.Add(data.start)
		for i := 0; i < data.start; i++ {
			go func(l string) {
				ls.CallStarted(l)
				wg.Done()
			}(locality)
		}
		// The calls to callStarted() need to happen before the other calls are
		// made. Hence the wait here.
		wg.Wait()

		wg.Add(data.success)
		for i := 0; i < data.success; i++ {
			go func(l string, serverData map[string]float64) {
				ls.CallFinished(l, nil)
				for n, d := range serverData {
					ls.CallServerLoad(l, n, d)
				}
				wg.Done()
			}(locality, data.serverData)
		}
		wg.Add(data.failure)
		for i := 0; i < data.failure; i++ {
			go func(l string) {
				ls.CallFinished(l, errTest)
				wg.Done()
			}(locality)
		}
		wg.Wait()
	}

	gotStoreData := ls.stats()
	if diff := cmp.Diff(wantStoreData, gotStoreData, cmpopts.EquateEmpty(), cmpopts.IgnoreFields(Data{}, "ReportInterval")); diff != "" {
		t.Errorf("store.stats() returned unexpected diff (-want +got):\n%s", diff)
	}
}

func TestResetAfterStats(t *testing.T) {
	// Push a bunch of drops, call stats and load stats, and leave inProgress to be non-zero.
	// Dump the stats. Verify expected
	// Push the same set of loads as before
	// Now dump and verify the newly expected ones.
	var (
		drops = map[string]int{
			dropCategories[0]: 30,
			dropCategories[1]: 40,
		}
		localityData = map[string]rpcData{
			localities[0]: {
				start:      40,
				success:    20,
				failure:    10,
				serverData: map[string]float64{"net": 1, "disk": 2, "cpu": 3, "mem": 4},
			},
			localities[1]: {
				start:      80,
				success:    40,
				failure:    20,
				serverData: map[string]float64{"net": 1, "disk": 2, "cpu": 3, "mem": 4},
			},
		}
		wantStoreData = &Data{
			TotalDrops: 70,
			Drops: map[string]uint64{
				dropCategories[0]: 30,
				dropCategories[1]: 40,
			},
			LocalityStats: map[string]LocalityData{
				localities[0]: {
					RequestStats: RequestData{
						Succeeded:  20,
						Errored:    10,
						InProgress: 10,
						Issued:     40,
					},

					LoadStats: map[string]ServerLoadData{
						"net":  {Count: 20, Sum: 20},
						"disk": {Count: 20, Sum: 40},
						"cpu":  {Count: 20, Sum: 60},
						"mem":  {Count: 20, Sum: 80},
					},
				},
				localities[1]: {
					RequestStats: RequestData{
						Succeeded:  40,
						Errored:    20,
						InProgress: 20,
						Issued:     80,
					},

					LoadStats: map[string]ServerLoadData{
						"net":  {Count: 40, Sum: 40},
						"disk": {Count: 40, Sum: 80},
						"cpu":  {Count: 40, Sum: 120},
						"mem":  {Count: 40, Sum: 160},
					},
				},
			},
		}
	)

	reportLoad := func(ls *perClusterStore) {
		for category, count := range drops {
			for i := 0; i < count; i++ {
				ls.CallDropped(category)
			}
		}
		for locality, data := range localityData {
			for i := 0; i < data.start; i++ {
				ls.CallStarted(locality)
			}
			for i := 0; i < data.success; i++ {
				ls.CallFinished(locality, nil)
				for n, d := range data.serverData {
					ls.CallServerLoad(locality, n, d)
				}
			}
			for i := 0; i < data.failure; i++ {
				ls.CallFinished(locality, errTest)
			}
		}
	}

	ls := perClusterStore{}
	reportLoad(&ls)
	gotStoreData := ls.stats()
	if diff := cmp.Diff(wantStoreData, gotStoreData, cmpopts.EquateEmpty(), cmpopts.IgnoreFields(Data{}, "ReportInterval")); diff != "" {
		t.Errorf("store.stats() returned unexpected diff (-want +got):\n%s", diff)
	}

	// The above call to stats() should have reset all load reports except the
	// inProgress rpc count. We are now going to push the same load data into
	// the store. So, we should expect to see twice the count for inProgress.
	for _, l := range localities {
		ls := wantStoreData.LocalityStats[l]
		ls.RequestStats.InProgress *= 2
		wantStoreData.LocalityStats[l] = ls
	}
	reportLoad(&ls)
	gotStoreData = ls.stats()
	if diff := cmp.Diff(wantStoreData, gotStoreData, cmpopts.EquateEmpty(), cmpopts.IgnoreFields(Data{}, "ReportInterval")); diff != "" {
		t.Errorf("store.stats() returned unexpected diff (-want +got):\n%s", diff)
	}
}

var sortDataSlice = cmp.Transformer("SortDataSlice", func(in []*Data) []*Data {
	out := append([]*Data(nil), in...) // Copy input to avoid mutating it
	sort.Slice(out,
		func(i, j int) bool {
			if out[i].Cluster < out[j].Cluster {
				return true
			}
			if out[i].Cluster == out[j].Cluster {
				return out[i].Service < out[j].Service
			}
			return false
		},
	)
	return out
})

// Test all load are returned for the given clusters, and all clusters are
// reported if no cluster is specified.
func TestStoreStats(t *testing.T) {
	var (
		testClusters = []string{"c0", "c1", "c2"}
		testServices = []string{"s0", "s1"}
		testLocality = "test-locality"
	)

	store := NewStore()
	for _, c := range testClusters {
		for _, s := range testServices {
			store.PerCluster(c, s).CallStarted(testLocality)
			store.PerCluster(c, s).CallServerLoad(testLocality, "abc", 123)
			store.PerCluster(c, s).CallDropped("dropped")
			store.PerCluster(c, s).CallFinished(testLocality, nil)
		}
	}

	wantC0 := []*Data{
		{
			Cluster: "c0", Service: "s0",
			TotalDrops: 1, Drops: map[string]uint64{"dropped": 1},
			LocalityStats: map[string]LocalityData{
				"test-locality": {
					RequestStats: RequestData{Succeeded: 1, Issued: 1},
					LoadStats:    map[string]ServerLoadData{"abc": {Count: 1, Sum: 123}},
				},
			},
		},
		{
			Cluster: "c0", Service: "s1",
			TotalDrops: 1, Drops: map[string]uint64{"dropped": 1},
			LocalityStats: map[string]LocalityData{
				"test-locality": {
					RequestStats: RequestData{Succeeded: 1, Issued: 1},
					LoadStats:    map[string]ServerLoadData{"abc": {Count: 1, Sum: 123}},
				},
			},
		},
	}
	// Call Stats with just "c0", this should return data for "c0", and not
	// touch data for other clusters.
	gotC0 := store.Stats([]string{"c0"})
	if diff := cmp.Diff(wantC0, gotC0, cmpopts.EquateEmpty(), cmpopts.IgnoreFields(Data{}, "ReportInterval"), sortDataSlice); diff != "" {
		t.Errorf("store.stats() returned unexpected diff (-want +got):\n%s", diff)
	}

	wantOther := []*Data{
		{
			Cluster: "c1", Service: "s0",
			TotalDrops: 1, Drops: map[string]uint64{"dropped": 1},
			LocalityStats: map[string]LocalityData{
				"test-locality": {
					RequestStats: RequestData{Succeeded: 1, Issued: 1},
					LoadStats:    map[string]ServerLoadData{"abc": {Count: 1, Sum: 123}},
				},
			},
		},
		{
			Cluster: "c1", Service: "s1",
			TotalDrops: 1, Drops: map[string]uint64{"dropped": 1},
			LocalityStats: map[string]LocalityData{
				"test-locality": {
					RequestStats: RequestData{Succeeded: 1, Issued: 1},
					LoadStats:    map[string]ServerLoadData{"abc": {Count: 1, Sum: 123}},
				},
			},
		},
		{
			Cluster: "c2", Service: "s0",
			TotalDrops: 1, Drops: map[string]uint64{"dropped": 1},
			LocalityStats: map[string]LocalityData{
				"test-locality": {
					RequestStats: RequestData{Succeeded: 1, Issued: 1},
					LoadStats:    map[string]ServerLoadData{"abc": {Count: 1, Sum: 123}},
				},
			},
		},
		{
			Cluster: "c2", Service: "s1",
			TotalDrops: 1, Drops: map[string]uint64{"dropped": 1},
			LocalityStats: map[string]LocalityData{
				"test-locality": {
					RequestStats: RequestData{Succeeded: 1, Issued: 1},
					LoadStats:    map[string]ServerLoadData{"abc": {Count: 1, Sum: 123}},
				},
			},
		},
	}
	// Call Stats with empty slice, this should return data for all the
	// remaining clusters, and not include c0 (because c0 data was cleared).
	gotOther := store.Stats(nil)
	if diff := cmp.Diff(wantOther, gotOther, cmpopts.EquateEmpty(), cmpopts.IgnoreFields(Data{}, "ReportInterval"), sortDataSlice); diff != "" {
		t.Errorf("store.stats() returned unexpected diff (-want +got):\n%s", diff)
	}
}

// Test the cases that if a cluster doesn't have load to report, its data is not
// appended to the slice returned by Stats().
func TestStoreStatsEmptyDataNotReported(t *testing.T) {
	var (
		testServices = []string{"s0", "s1"}
		testLocality = "test-locality"
	)

	store := NewStore()
	// "c0"'s RPCs all finish with success.
	for _, s := range testServices {
		store.PerCluster("c0", s).CallStarted(testLocality)
		store.PerCluster("c0", s).CallFinished(testLocality, nil)
	}
	// "c1"'s RPCs never finish (always inprocess).
	for _, s := range testServices {
		store.PerCluster("c1", s).CallStarted(testLocality)
	}

	want0 := []*Data{
		{
			Cluster: "c0", Service: "s0",
			LocalityStats: map[string]LocalityData{
				"test-locality": {RequestStats: RequestData{Succeeded: 1, Issued: 1}},
			},
		},
		{
			Cluster: "c0", Service: "s1",
			LocalityStats: map[string]LocalityData{
				"test-locality": {RequestStats: RequestData{Succeeded: 1, Issued: 1}},
			},
		},
		{
			Cluster: "c1", Service: "s0",
			LocalityStats: map[string]LocalityData{
				"test-locality": {RequestStats: RequestData{InProgress: 1, Issued: 1}},
			},
		},
		{
			Cluster: "c1", Service: "s1",
			LocalityStats: map[string]LocalityData{
				"test-locality": {RequestStats: RequestData{InProgress: 1, Issued: 1}},
			},
		},
	}
	// Call Stats with empty slice, this should return data for all the
	// clusters.
	got0 := store.Stats(nil)
	if diff := cmp.Diff(want0, got0, cmpopts.EquateEmpty(), cmpopts.IgnoreFields(Data{}, "ReportInterval"), sortDataSlice); diff != "" {
		t.Errorf("store.stats() returned unexpected diff (-want +got):\n%s", diff)
	}

	want1 := []*Data{
		{
			Cluster: "c1", Service: "s0",
			LocalityStats: map[string]LocalityData{
				"test-locality": {RequestStats: RequestData{InProgress: 1}},
			},
		},
		{
			Cluster: "c1", Service: "s1",
			LocalityStats: map[string]LocalityData{
				"test-locality": {RequestStats: RequestData{InProgress: 1}},
			},
		},
	}
	// Call Stats with empty slice again, this should return data only for "c1",
	// because "c0" data was cleared, but "c1" has in-progress RPCs.
	got1 := store.Stats(nil)
	if diff := cmp.Diff(want1, got1, cmpopts.EquateEmpty(), cmpopts.IgnoreFields(Data{}, "ReportInterval"), sortDataSlice); diff != "" {
		t.Errorf("store.stats() returned unexpected diff (-want +got):\n%s", diff)
	}
}
