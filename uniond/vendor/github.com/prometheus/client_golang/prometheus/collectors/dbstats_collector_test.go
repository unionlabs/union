// Copyright 2021 The Prometheus Authors
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

package collectors

import (
	"database/sql"
	"testing"

	"github.com/prometheus/client_golang/prometheus"
)

func TestDBStatsCollector(t *testing.T) {
	reg := prometheus.NewPedanticRegistry()
	{
		db := new(sql.DB)
		if err := reg.Register(NewDBStatsCollector(db, "db_A")); err != nil {
			t.Fatal(err)
		}
	}
	{
		db := new(sql.DB)
		if err := reg.Register(NewDBStatsCollector(db, "db_B")); err != nil {
			t.Fatal(err)
		}
	}

	mfs, err := reg.Gather()
	if err != nil {
		t.Fatal(err)
	}

	names := []string{
		"go_sql_max_open_connections",
		"go_sql_open_connections",
		"go_sql_in_use_connections",
		"go_sql_idle_connections",
		"go_sql_wait_count_total",
		"go_sql_wait_duration_seconds_total",
		"go_sql_max_idle_closed_total",
		"go_sql_max_lifetime_closed_total",
		"go_sql_max_idle_time_closed_total",
	}
	type result struct {
		found bool
	}
	results := make(map[string]result)
	for _, name := range names {
		results[name] = result{found: false}
	}
	for _, mf := range mfs {
		m := mf.GetMetric()
		if len(m) != 2 {
			t.Errorf("expected 2 metrics bug got %d", len(m))
		}
		labelA := m[0].GetLabel()[0]
		if name := labelA.GetName(); name != "db_name" {
			t.Errorf("expected to get label \"db_name\" but got %s", name)
		}
		if value := labelA.GetValue(); value != "db_A" {
			t.Errorf("expected to get value \"db_A\" but got %s", value)
		}
		labelB := m[1].GetLabel()[0]
		if name := labelB.GetName(); name != "db_name" {
			t.Errorf("expected to get label \"db_name\" but got %s", name)
		}
		if value := labelB.GetValue(); value != "db_B" {
			t.Errorf("expected to get value \"db_B\" but got %s", value)
		}

		for _, name := range names {
			if name == mf.GetName() {
				results[name] = result{found: true}
				break
			}
		}
	}

	for name, result := range results {
		if !result.found {
			t.Errorf("%s not found", name)
		}
	}
}
