// Copyright 2022 The Prometheus Authors
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

//go:build go1.20 && !go1.21
// +build go1.20,!go1.21

package collectors

import "sort"

func withAllMetrics() []string {
	return withBaseMetrics([]string{
		"go_cgo_go_to_c_calls_calls_total",
		"go_cpu_classes_gc_mark_assist_cpu_seconds_total",
		"go_cpu_classes_gc_mark_dedicated_cpu_seconds_total",
		"go_cpu_classes_gc_mark_idle_cpu_seconds_total",
		"go_cpu_classes_gc_pause_cpu_seconds_total",
		"go_cpu_classes_gc_total_cpu_seconds_total",
		"go_cpu_classes_idle_cpu_seconds_total",
		"go_cpu_classes_scavenge_assist_cpu_seconds_total",
		"go_cpu_classes_scavenge_background_cpu_seconds_total",
		"go_cpu_classes_scavenge_total_cpu_seconds_total",
		"go_cpu_classes_total_cpu_seconds_total",
		"go_cpu_classes_user_cpu_seconds_total",
		"go_gc_cycles_automatic_gc_cycles_total",
		"go_gc_cycles_forced_gc_cycles_total",
		"go_gc_cycles_total_gc_cycles_total",
		"go_gc_heap_allocs_by_size_bytes",
		"go_gc_heap_allocs_bytes_total",
		"go_gc_heap_allocs_objects_total",
		"go_gc_heap_frees_by_size_bytes",
		"go_gc_heap_frees_bytes_total",
		"go_gc_heap_frees_objects_total",
		"go_gc_heap_goal_bytes",
		"go_gc_heap_objects_objects",
		"go_gc_heap_tiny_allocs_objects_total",
		"go_gc_limiter_last_enabled_gc_cycle",
		"go_gc_pauses_seconds",
		"go_gc_stack_starting_size_bytes",
		"go_memory_classes_heap_free_bytes",
		"go_memory_classes_heap_objects_bytes",
		"go_memory_classes_heap_released_bytes",
		"go_memory_classes_heap_stacks_bytes",
		"go_memory_classes_heap_unused_bytes",
		"go_memory_classes_metadata_mcache_free_bytes",
		"go_memory_classes_metadata_mcache_inuse_bytes",
		"go_memory_classes_metadata_mspan_free_bytes",
		"go_memory_classes_metadata_mspan_inuse_bytes",
		"go_memory_classes_metadata_other_bytes",
		"go_memory_classes_os_stacks_bytes",
		"go_memory_classes_other_bytes",
		"go_memory_classes_profiling_buckets_bytes",
		"go_memory_classes_total_bytes",
		"go_sched_gomaxprocs_threads",
		"go_sched_goroutines_goroutines",
		"go_sched_latencies_seconds",
		"go_sync_mutex_wait_total_seconds_total",
	})
}

func withGCMetrics() []string {
	return withBaseMetrics([]string{
		"go_gc_cycles_automatic_gc_cycles_total",
		"go_gc_cycles_forced_gc_cycles_total",
		"go_gc_cycles_total_gc_cycles_total",
		"go_gc_heap_allocs_by_size_bytes",
		"go_gc_heap_allocs_bytes_total",
		"go_gc_heap_allocs_objects_total",
		"go_gc_heap_frees_by_size_bytes",
		"go_gc_heap_frees_bytes_total",
		"go_gc_heap_frees_objects_total",
		"go_gc_heap_goal_bytes",
		"go_gc_heap_objects_objects",
		"go_gc_heap_tiny_allocs_objects_total",
		"go_gc_limiter_last_enabled_gc_cycle",
		"go_gc_pauses_seconds",
		"go_gc_stack_starting_size_bytes",
	})
}

func withMemoryMetrics() []string {
	return withBaseMetrics([]string{
		"go_memory_classes_heap_free_bytes",
		"go_memory_classes_heap_objects_bytes",
		"go_memory_classes_heap_released_bytes",
		"go_memory_classes_heap_stacks_bytes",
		"go_memory_classes_heap_unused_bytes",
		"go_memory_classes_metadata_mcache_free_bytes",
		"go_memory_classes_metadata_mcache_inuse_bytes",
		"go_memory_classes_metadata_mspan_free_bytes",
		"go_memory_classes_metadata_mspan_inuse_bytes",
		"go_memory_classes_metadata_other_bytes",
		"go_memory_classes_os_stacks_bytes",
		"go_memory_classes_other_bytes",
		"go_memory_classes_profiling_buckets_bytes",
		"go_memory_classes_total_bytes",
	})
}

func withSchedulerMetrics() []string {
	return withBaseMetrics([]string{
		"go_sched_gomaxprocs_threads",
		"go_sched_goroutines_goroutines",
		"go_sched_latencies_seconds",
	})
}

func withDebugMetrics() []string {
	return withBaseMetrics([]string{})
}

var defaultRuntimeMetrics = []string{
	"go_sched_gomaxprocs_threads",
}

func withDefaultRuntimeMetrics(metricNames []string, withoutGC, withoutSched bool) []string {
	// If withoutSched is true, exclude "go_sched_gomaxprocs_threads".
	if withoutSched {
		return metricNames
	}
	metricNames = append(metricNames, defaultRuntimeMetrics...)
	// sorting is required
	sort.Strings(metricNames)
	return metricNames
}
