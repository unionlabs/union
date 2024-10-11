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

//go:build go1.17
// +build go1.17

package collectors

import (
	"encoding/json"
	"log"
	"net/http"
	"regexp"
	"sort"
	"testing"

	"github.com/google/go-cmp/cmp"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

var baseMetrics = []string{
	"go_gc_duration_seconds",
	"go_goroutines",
	"go_info",
	"go_memstats_last_gc_time_seconds",
	"go_threads",
}

var memstatMetrics = []string{
	"go_memstats_alloc_bytes",
	"go_memstats_alloc_bytes_total",
	"go_memstats_buck_hash_sys_bytes",
	"go_memstats_frees_total",
	"go_memstats_gc_sys_bytes",
	"go_memstats_heap_alloc_bytes",
	"go_memstats_heap_idle_bytes",
	"go_memstats_heap_inuse_bytes",
	"go_memstats_heap_objects",
	"go_memstats_heap_released_bytes",
	"go_memstats_heap_sys_bytes",
	"go_memstats_mallocs_total",
	"go_memstats_mcache_inuse_bytes",
	"go_memstats_mcache_sys_bytes",
	"go_memstats_mspan_inuse_bytes",
	"go_memstats_mspan_sys_bytes",
	"go_memstats_next_gc_bytes",
	"go_memstats_other_sys_bytes",
	"go_memstats_stack_inuse_bytes",
	"go_memstats_stack_sys_bytes",
	"go_memstats_sys_bytes",
}

func TestGoCollectorMarshalling(t *testing.T) {
	reg := prometheus.NewPedanticRegistry()
	reg.MustRegister(NewGoCollector(
		WithGoCollectorRuntimeMetrics(GoRuntimeMetricsRule{
			Matcher: regexp.MustCompile("/.*"),
		}),
	))
	result, err := reg.Gather()
	if err != nil {
		t.Fatal(err)
	}

	if _, err := json.Marshal(result); err != nil {
		t.Errorf("json marshalling should not fail, %v", err)
	}
}

func TestWithGoCollectorDefault(t *testing.T) {
	reg := prometheus.NewPedanticRegistry()
	reg.MustRegister(NewGoCollector())
	result, err := reg.Gather()
	if err != nil {
		t.Fatal(err)
	}

	got := []string{}
	for _, r := range result {
		got = append(got, r.GetName())
	}

	expected := append(withBaseMetrics(memstatMetrics), defaultRuntimeMetrics...)
	sort.Strings(expected)
	if diff := cmp.Diff(got, expected); diff != "" {
		t.Errorf("[IMPORTANT, those are default metrics, can't change in 1.x] missmatch (-want +got):\n%s", diff)
	}
}

func TestWithGoCollectorMemStatsMetricsDisabled(t *testing.T) {
	reg := prometheus.NewPedanticRegistry()
	reg.MustRegister(NewGoCollector(
		WithGoCollectorMemStatsMetricsDisabled(),
	))
	result, err := reg.Gather()
	if err != nil {
		t.Fatal(err)
	}

	got := []string{}
	for _, r := range result {
		got = append(got, r.GetName())
	}

	if diff := cmp.Diff(got, withBaseMetrics(defaultRuntimeMetrics)); diff != "" {
		t.Errorf("missmatch (-want +got):\n%s", diff)
	}
}

func TestGoCollectorAllowList(t *testing.T) {
	for _, test := range []struct {
		name     string
		rules    []GoRuntimeMetricsRule
		expected []string
	}{
		{
			name:     "Without any rules",
			rules:    nil,
			expected: withBaseMetrics(defaultRuntimeMetrics),
		},
		{
			name:     "allow all",
			rules:    []GoRuntimeMetricsRule{MetricsAll},
			expected: withAllMetrics(),
		},
		{
			name:     "allow GC",
			rules:    []GoRuntimeMetricsRule{MetricsGC},
			expected: withDefaultRuntimeMetrics(withGCMetrics(), true, false),
		},
		{
			name:     "allow Memory",
			rules:    []GoRuntimeMetricsRule{MetricsMemory},
			expected: withDefaultRuntimeMetrics(withMemoryMetrics(), false, false),
		},
		{
			name:     "allow Scheduler",
			rules:    []GoRuntimeMetricsRule{MetricsScheduler},
			expected: withDefaultRuntimeMetrics(withSchedulerMetrics(), false, true),
		},
		{
			name:     "allow debug",
			rules:    []GoRuntimeMetricsRule{MetricsDebug},
			expected: withDefaultRuntimeMetrics(withDebugMetrics(), false, false),
		},
	} {
		t.Run(test.name, func(t *testing.T) {
			reg := prometheus.NewPedanticRegistry()
			reg.MustRegister(NewGoCollector(
				WithGoCollectorMemStatsMetricsDisabled(),
				WithGoCollectorRuntimeMetrics(test.rules...),
			))
			result, err := reg.Gather()
			if err != nil {
				t.Fatal(err)
			}

			got := []string{}
			for _, r := range result {
				got = append(got, r.GetName())
			}

			if diff := cmp.Diff(got, test.expected); diff != "" {
				t.Errorf("missmatch (-want +got):\n%s", diff)
			}
		})
	}
}

func withBaseMetrics(metricNames []string) []string {
	metricNames = append(metricNames, baseMetrics...)
	sort.Strings(metricNames)
	return metricNames
}

func TestGoCollectorDenyList(t *testing.T) {
	for _, test := range []struct {
		name     string
		matchers []*regexp.Regexp
		expected []string
	}{
		{
			name:     "Without any matchers",
			matchers: nil,
			expected: withBaseMetrics(defaultRuntimeMetrics),
		},
		{
			name:     "deny all",
			matchers: []*regexp.Regexp{regexp.MustCompile("/.*")},
			expected: baseMetrics,
		},
		{
			name: "deny gc and scheduler latency",
			matchers: []*regexp.Regexp{
				regexp.MustCompile("^/gc/.*"),
				regexp.MustCompile("^/sched/latencies:.*"),
			},
			expected: withDefaultRuntimeMetrics(baseMetrics, true, false),
		},
		{
			name: "deny gc and scheduler",
			matchers: []*regexp.Regexp{
				regexp.MustCompile("^/gc/.*"),
				regexp.MustCompile("^/sched/.*"),
			},
			expected: baseMetrics,
		},
	} {
		t.Run(test.name, func(t *testing.T) {
			reg := prometheus.NewPedanticRegistry()
			reg.MustRegister(NewGoCollector(
				WithGoCollectorMemStatsMetricsDisabled(),
				WithoutGoCollectorRuntimeMetrics(test.matchers...),
			))
			result, err := reg.Gather()
			if err != nil {
				t.Fatal(err)
			}

			got := []string{}
			for _, r := range result {
				got = append(got, r.GetName())
			}

			if diff := cmp.Diff(got, test.expected); diff != "" {
				t.Errorf("missmatch (-want +got):\n%s", diff)
			}
		})
	}
}

func ExampleGoCollector() {
	reg := prometheus.NewPedanticRegistry()

	// Register the GoCollector with the default options. Only the base metrics, default runtime metrics and memstats are enabled.
	reg.MustRegister(NewGoCollector())

	http.Handle("/metrics", promhttp.HandlerFor(reg, promhttp.HandlerOpts{}))
	log.Fatal(http.ListenAndServe(":8080", nil))
}

func ExampleGoCollector_WithAdvancedGoMetrics() {
	reg := prometheus.NewPedanticRegistry()

	// Enable Go metrics with pre-defined rules. Or your custom rules.
	reg.MustRegister(
		NewGoCollector(
			WithGoCollectorMemStatsMetricsDisabled(),
			WithGoCollectorRuntimeMetrics(
				MetricsScheduler,
				MetricsMemory,
				GoRuntimeMetricsRule{
					Matcher: regexp.MustCompile("^/mycustomrule.*"),
				},
			),
			WithoutGoCollectorRuntimeMetrics(regexp.MustCompile("^/gc/.*")),
		))

	http.Handle("/metrics", promhttp.HandlerFor(reg, promhttp.HandlerOpts{}))
	log.Fatal(http.ListenAndServe(":8080", nil))
}

func ExampleGoCollector_DefaultRegister() {
	// Unregister the default GoCollector.
	prometheus.Unregister(NewGoCollector())

	// Register the default GoCollector with a custom config.
	prometheus.MustRegister(NewGoCollector(WithGoCollectorRuntimeMetrics(
		MetricsScheduler,
		MetricsGC,
		GoRuntimeMetricsRule{
			Matcher: regexp.MustCompile("^/mycustomrule.*"),
		},
	),
	))

	http.Handle("/metrics", promhttp.Handler())
	log.Fatal(http.ListenAndServe(":8080", nil))
}
