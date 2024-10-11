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

//go:build go1.21 && !go1.22
// +build go1.21,!go1.22

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
		"go_gc_gogc_percent",
		"go_gc_gomemlimit_bytes",
		"go_gc_heap_allocs_by_size_bytes",
		"go_gc_heap_allocs_bytes_total",
		"go_gc_heap_allocs_objects_total",
		"go_gc_heap_frees_by_size_bytes",
		"go_gc_heap_frees_bytes_total",
		"go_gc_heap_frees_objects_total",
		"go_gc_heap_goal_bytes",
		"go_gc_heap_live_bytes",
		"go_gc_heap_objects_objects",
		"go_gc_heap_tiny_allocs_objects_total",
		"go_gc_limiter_last_enabled_gc_cycle",
		"go_gc_pauses_seconds",
		"go_gc_scan_globals_bytes",
		"go_gc_scan_heap_bytes",
		"go_gc_scan_stack_bytes",
		"go_gc_scan_total_bytes",
		"go_gc_stack_starting_size_bytes",
		"go_godebug_non_default_behavior_execerrdot_events_total",
		"go_godebug_non_default_behavior_gocachehash_events_total",
		"go_godebug_non_default_behavior_gocachetest_events_total",
		"go_godebug_non_default_behavior_gocacheverify_events_total",
		"go_godebug_non_default_behavior_http2client_events_total",
		"go_godebug_non_default_behavior_http2server_events_total",
		"go_godebug_non_default_behavior_installgoroot_events_total",
		"go_godebug_non_default_behavior_jstmpllitinterp_events_total",
		"go_godebug_non_default_behavior_multipartmaxheaders_events_total",
		"go_godebug_non_default_behavior_multipartmaxparts_events_total",
		"go_godebug_non_default_behavior_multipathtcp_events_total",
		"go_godebug_non_default_behavior_netedns0_events_total",
		"go_godebug_non_default_behavior_panicnil_events_total",
		"go_godebug_non_default_behavior_randautoseed_events_total",
		"go_godebug_non_default_behavior_tarinsecurepath_events_total",
		"go_godebug_non_default_behavior_tlsmaxrsasize_events_total",
		"go_godebug_non_default_behavior_x509sha1_events_total",
		"go_godebug_non_default_behavior_x509usefallbackroots_events_total",
		"go_godebug_non_default_behavior_zipinsecurepath_events_total",
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
		"go_gc_gogc_percent",
		"go_gc_gomemlimit_bytes",
		"go_gc_heap_allocs_by_size_bytes",
		"go_gc_heap_allocs_bytes_total",
		"go_gc_heap_allocs_objects_total",
		"go_gc_heap_frees_by_size_bytes",
		"go_gc_heap_frees_bytes_total",
		"go_gc_heap_frees_objects_total",
		"go_gc_heap_goal_bytes",
		"go_gc_heap_live_bytes",
		"go_gc_heap_objects_objects",
		"go_gc_heap_tiny_allocs_objects_total",
		"go_gc_limiter_last_enabled_gc_cycle",
		"go_gc_pauses_seconds",
		"go_gc_scan_globals_bytes",
		"go_gc_scan_heap_bytes",
		"go_gc_scan_stack_bytes",
		"go_gc_scan_total_bytes",
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
	return withBaseMetrics([]string{
		"go_godebug_non_default_behavior_execerrdot_events_total",
		"go_godebug_non_default_behavior_gocachehash_events_total",
		"go_godebug_non_default_behavior_gocachetest_events_total",
		"go_godebug_non_default_behavior_gocacheverify_events_total",
		"go_godebug_non_default_behavior_http2client_events_total",
		"go_godebug_non_default_behavior_http2server_events_total",
		"go_godebug_non_default_behavior_installgoroot_events_total",
		"go_godebug_non_default_behavior_jstmpllitinterp_events_total",
		"go_godebug_non_default_behavior_multipartmaxheaders_events_total",
		"go_godebug_non_default_behavior_multipartmaxparts_events_total",
		"go_godebug_non_default_behavior_multipathtcp_events_total",
		"go_godebug_non_default_behavior_netedns0_events_total",
		"go_godebug_non_default_behavior_panicnil_events_total",
		"go_godebug_non_default_behavior_randautoseed_events_total",
		"go_godebug_non_default_behavior_tarinsecurepath_events_total",
		"go_godebug_non_default_behavior_tlsmaxrsasize_events_total",
		"go_godebug_non_default_behavior_x509sha1_events_total",
		"go_godebug_non_default_behavior_x509usefallbackroots_events_total",
		"go_godebug_non_default_behavior_zipinsecurepath_events_total",
	})
}

var defaultRuntimeMetrics = []string{
	"go_gc_gogc_percent",
	"go_gc_gomemlimit_bytes",
	"go_sched_gomaxprocs_threads",
}

func withDefaultRuntimeMetrics(metricNames []string, withoutGC, withoutSched bool) []string {
	if withoutGC && withoutSched {
		// If both flags are true, return the metricNames as is.
		return metricNames
	} else if withoutGC && !withoutSched {
		// If only withoutGC is true, include "go_sched_gomaxprocs_threads" only.
		metricNames = append(metricNames, []string{"go_sched_gomaxprocs_threads"}...)
	} else if withoutSched && !withoutGC {
		// If only withoutSched is true, exclude "go_sched_gomaxprocs_threads".
		metricNames = append(metricNames, []string{"go_gc_gogc_percent", "go_gc_gomemlimit_bytes"}...)
	} else {
		// If neither flag is true, use the default metrics.
		metricNames = append(metricNames, defaultRuntimeMetrics...)
	}
	// sorting is required
	sort.Strings(metricNames)
	return metricNames
}
