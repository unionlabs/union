## Unreleased

* [BUGFIX] histograms: Fix possible data race when appending exemplars vs metrics gather. #1623

## 1.20.3 / 2024-09-05

* [BUGFIX] histograms: Fix possible data race when appending exemplars. #1608

## 1.20.2 / 2024-08-23

* [BUGFIX] promhttp: Unset Content-Encoding header when data is uncompressed. #1596

## 1.20.1 / 2024-08-20

* [BUGFIX] process-collector: Fixed unregistered descriptor error when using process collector with `PedanticRegistry` on linux machines. #1587

## 1.20.0 / 2024-08-14

* [CHANGE] :warning: go-collector: Remove `go_memstat_lookups_total` metric which was always 0; Go runtime stopped sharing pointer lookup statistics. #1577
* [FEATURE] :warning: go-collector: Add 3 default metrics: `go_gc_gogc_percent`, `go_gc_gomemlimit_bytes` and `go_sched_gomaxprocs_threads` as those are recommended by the Go team. #1559
* [FEATURE] go-collector: Add more information to all metrics' HELP e.g. the exact `runtime/metrics` sourcing each metric (if relevant). #1568 #1578
* [FEATURE] testutil: Add CollectAndFormat method. #1503
* [FEATURE] histograms: Add support for exemplars in native histograms. #1471
* [FEATURE] promhttp: Add experimental support for `zstd` on scrape, controlled by the request `Accept-Encoding` header. #1496
* [FEATURE] api/v1: Add `WithLimit` parameter to all API methods that supports it. #1544
* [FEATURE] prometheus: Add support for created timestamps in constant histograms and constant summaries. #1537
* [FEATURE] process-collector: Add network usage metrics: `process_network_receive_bytes_total` and `process_network_transmit_bytes_total`. #1555
* [FEATURE] promlint: Add duplicated metric lint rule. #1472
* [BUGFIX] promlint: Relax metric type in name linter rule. #1455
* [BUGFIX] promhttp: Make sure server instrumentation wrapping supports new and future extra responseWriter methods. #1480
* [BUGFIX] testutil: Functions using compareMetricFamilies are now failing if filtered metricNames are not in the input. #1424

## 1.19.0 / 2024-02-27

The module `prometheus/common v0.48.0` introduced an incompatibility when used together with client_golang (See https://github.com/prometheus/client_golang/pull/1448 for more details). If your project uses client_golang and you want to use `prometheus/common v0.48.0` or higher, please update client_golang to v1.19.0.

* [CHANGE] Minimum required go version is now 1.20 (we also test client_golang against new 1.22 version). #1445 #1449
* [FEATURE] collectors: Add version collector. #1422 #1427

## 1.18.0 / 2023-12-22

* [FEATURE] promlint: Allow creation of custom metric validations. #1311
* [FEATURE] Go programs using client_golang can be built in wasip1 OS. #1350
* [BUGFIX] histograms: Add timer to reset ASAP after bucket limiting has happened. #1367
* [BUGFIX] testutil: Fix comparison of metrics with empty Help strings. #1378
* [ENHANCEMENT] Improved performance of `MetricVec.WithLabelValues(...)`. #1360

## 1.17.0 / 2023-09-27

* [CHANGE] Minimum required go version is now 1.19 (we also test client_golang against new 1.21 version). #1325
* [FEATURE] Add support for Created Timestamps in Counters, Summaries and Historams. #1313
* [ENHANCEMENT] Enable detection of a native histogram without observations. #1314

## 1.16.0 / 2023-06-15

* [BUGFIX] api: Switch to POST for LabelNames, Series, and QueryExemplars. #1252
* [BUGFIX] api: Fix undefined execution order in return statements. #1260
* [BUGFIX] native histograms: Fix bug in bucket key calculation. #1279
* [ENHANCEMENT] Reduce constrainLabels allocations for all metrics. #1272
* [ENHANCEMENT] promhttp: Add process start time header for scrape efficiency. #1278
* [ENHANCEMENT] promlint: Improve metricUnits runtime. #1286

## 1.15.1 / 2023-05-3

* [BUGFIX] Fixed promhttp.Instrument* handlers wrongly trying to attach exemplar to unsupported metrics (e.g. summary), \
causing panics. #1253

## 1.15.0 / 2023-04-13

* [BUGFIX] Fix issue with atomic variables on ppc64le. #1171
* [BUGFIX] Support for multiple samples within same metric. #1181
* [BUGFIX] Bump golang.org/x/text to v0.3.8 to mitigate CVE-2022-32149. #1187
* [ENHANCEMENT] Add exemplars and middleware examples. #1173
* [ENHANCEMENT] Add more context to "duplicate label names" error to enable debugging. #1177
* [ENHANCEMENT] Add constrained labels and constrained variant for all MetricVecs. #1151
* [ENHANCEMENT] Moved away from deprecated github.com/golang/protobuf package. #1183
* [ENHANCEMENT] Add possibility to dynamically get label values for http instrumentation. #1066
* [ENHANCEMENT] Add ability to Pusher to add custom headers. #1218
* [ENHANCEMENT] api: Extend and improve efficiency of json-iterator usage. #1225
* [ENHANCEMENT] Added (official) support for go 1.20. #1234
* [ENHANCEMENT] timer: Added support for exemplars. #1233
* [ENHANCEMENT] Filter expected metrics as well in CollectAndCompare. #1143
* [ENHANCEMENT] :warning: Only set start/end if time is not Zero. This breaks compatibility in experimental api package. If you strictly depend on empty time.Time as actual value, the behavior is now changed. #1238

## 1.14.0 / 2022-11-08

* [FEATURE] Add Support for Native Histograms. #1150
* [CHANGE] Extend `prometheus.Registry` to implement `prometheus.Collector` interface. #1103

## 1.13.1 / 2022-11-01

* [BUGFIX] Fix race condition with Exemplar in Counter. #1146
* [BUGFIX] Fix `CumulativeCount` value of `+Inf` bucket created from exemplar. #1148
* [BUGFIX] Fix double-counting bug in `promhttp.InstrumentRoundTripperCounter`. #1118

## 1.13.0 / 2022-08-05

* [CHANGE] Minimum required Go version is now 1.17 (we also test client_golang against new 1.19 version).
* [ENHANCEMENT] Added `prometheus.TransactionalGatherer` interface for `promhttp.Handler` use which allows using low allocation update techniques for custom collectors. #989
* [ENHANCEMENT] Added exemplar support to `prometheus.NewConstHistogram`. See [`ExampleNewConstHistogram_WithExemplar`](prometheus/examples_test.go#L602) example on how to use it. #986
* [ENHANCEMENT] `prometheus/push.Pusher` has now context aware methods that pass context to HTTP request. #1028
* [ENHANCEMENT] `prometheus/push.Pusher` has now `Error` method that retrieve last error. #1075
* [ENHANCEMENT] `testutil.GatherAndCompare` provides now readable diff on failed comparisons. #998
* [ENHANCEMENT] Query API now supports timeouts. #1014
* [ENHANCEMENT] New `MetricVec` method `DeletePartialMatch(labels Labels)` for deleting all metrics that match provided labels. #1013
* [ENHANCEMENT] `api.Config` now accepts passing custom `*http.Client`. #1025
* [BUGFIX] Raise exemplar labels limit from 64 to 128 bytes as specified in OpenMetrics spec. #1091
* [BUGFIX] Allow adding exemplar to +Inf bucket to const histograms. #1094
* [ENHANCEMENT] Most `promhttp.Instrument*` middlewares now supports adding exemplars to metrics. This allows hooking those to your tracing middleware that retrieves trace ID and put it in exemplar if present. #1055
* [ENHANCEMENT] Added `testutil.ScrapeAndCompare` method. #1043
* [BUGFIX] Fixed `GopherJS` build support. #897
* [ENHANCEMENT] :warning: Added way to specify what `runtime/metrics`  `collectors.NewGoCollector` should use. See [`ExampleGoCollector_WithAdvancedGoMetrics`](prometheus/collectors/go_collector_latest_test.go#L263). #1102

## 1.12.2 / 2022-05-13

* [CHANGE] Added `collectors.WithGoCollections` that allows to choose what collection of Go runtime metrics user wants: Equivalent of [`MemStats` structure](https://pkg.go.dev/runtime#MemStats) configured using `GoRuntimeMemStatsCollection`, new based on dedicated [runtime/metrics](https://pkg.go.dev/runtime/metrics) metrics represented by `GoRuntimeMetricsCollection` option, or both by specifying `GoRuntimeMemStatsCollection | GoRuntimeMetricsCollection` flag. #1031
* [CHANGE] :warning: Change in `collectors.NewGoCollector` metrics: Reverting addition of new ~80 runtime metrics by default. You can enable this back with `GoRuntimeMetricsCollection` option or `GoRuntimeMemStatsCollection | GoRuntimeMetricsCollection` for smooth transition.
* [BUGFIX] Fixed the bug that causes generated histogram metric names to end with `_total`. ⚠️ This changes 3 metric names in the new Go collector that was reverted from default in this release.
  * `go_gc_heap_allocs_by_size_bytes_total` -> `go_gc_heap_allocs_by_size_bytes`,
  * `go_gc_heap_frees_by_size_bytes_total` -> `go_gc_heap_allocs_by_size_bytes`
  * `go_gc_pauses_seconds_total` -> `go_gc_pauses_seconds`.
* [CHANCE] Removed `-Inf` buckets from new Go Collector histograms.

## 1.12.1 / 2022-01-29

* [BUGFIX] Make the Go 1.17 collector concurrency-safe #969
  * Use simpler locking in the Go 1.17 collector #975
* [BUGFIX] Reduce granularity of histogram buckets for Go 1.17 collector #974
* [ENHANCEMENT] API client: make HTTP reads more efficient #976

## 1.12.0 / 2022-01-19

* [CHANGE] example/random: Move flags and metrics into main() #935
* [FEATURE] API client: Support wal replay status api #944
* [FEATURE] Use the runtime/metrics package for the Go collector for 1.17+ #955
* [ENHANCEMENT] API client: Update /api/v1/status/tsdb to include headStats #925
* [ENHANCEMENT] promhttp: Check validity of method and code label values #962

## 1.11.0 / 2021-06-07

* [CHANGE] Add new collectors package. #862
* [CHANGE] `prometheus.NewExpvarCollector` is deprecated, use `collectors.NewExpvarCollector` instead. #862
* [CHANGE] `prometheus.NewGoCollector` is deprecated, use `collectors.NewGoCollector` instead. #862
* [CHANGE] `prometheus.NewBuildInfoCollector` is deprecated, use `collectors.NewBuildInfoCollector` instead. #862
* [FEATURE] Add new collector for database/sql#DBStats. #866
* [FEATURE] API client: Add exemplars API support. #861
* [ENHANCEMENT] API client: Add newer fields to Rules API. #855
* [ENHANCEMENT] API client: Add missing fields to Targets API. #856

## 1.10.0 / 2021-03-18

* [CHANGE] Minimum required Go version is now 1.13.
* [CHANGE] API client: Add matchers to `LabelNames` and `LabesValues`. #828
* [FEATURE] API client: Add buildinfo call. #841
* [BUGFIX] Fix build on riscv64. #833

## 1.9.0 / 2020-12-17

* [FEATURE] `NewPidFileFn` helper to create process collectors for processes whose PID is read from a file. #804
* [BUGFIX] promhttp: Prevent endless loop in `InstrumentHandler...` middlewares with invalid metric or label names. #823

## 1.8.0 / 2020-10-15

* [CHANGE] API client: Use `time.Time` rather than `string` for timestamps in `RuntimeinfoResult`. #777
* [FEATURE] Export `MetricVec` to facilitate implementation of vectors of custom `Metric` types. #803
* [FEATURE] API client: Support `/status/tsdb` endpoint. #773
* [ENHANCEMENT] API client: Enable GET fallback on status code 501. #802
* [ENHANCEMENT] Remove `Metric` references after reslicing to free up more memory. #784

## 1.7.1 / 2020-06-23

* [BUGFIX] API client: Actually propagate start/end parameters of `LabelNames` and `LabelValues`. #771

## 1.7.0 / 2020-06-17

* [CHANGE] API client: Add start/end parameters to `LabelNames` and `LabelValues`. #767
* [FEATURE] testutil: Add `GatherAndCount` and enable filtering in `CollectAndCount` #753
* [FEATURE] API client: Add support for `status` and `runtimeinfo` endpoints. #755
* [ENHANCEMENT] Wrapping `nil` with a `WrapRegistererWith...` function creates a no-op `Registerer`.  #764
* [ENHANCEMENT] promlint: Allow Kelvin as a base unit for cases like color temperature. #761
* [BUGFIX] push: Properly handle empty job and label values. #752

## 1.6.0 / 2020-04-28

* [FEATURE] testutil: Add lint checks for metrics, including a sub-package `promlint` to expose the linter engine for external usage. #739 #743
* [ENHANCEMENT] API client: Improve error messages. #731
* [BUGFIX] process collector: Fix `process_resident_memory_bytes` on 32bit MS Windows. #734

## 1.5.1 / 2020-03-14

* [BUGFIX] promhttp: Remove another superfluous `WriteHeader` call. #726

## 1.5.0 / 2020-03-03

* [FEATURE] promauto: Add a factory to allow automatic registration with a local registry. #713
* [FEATURE] promauto: Add `NewUntypedFunc`. #713
* [FEATURE] API client: Support new metadata endpoint. #718

## 1.4.1 / 2020-02-07

* [BUGFIX] Fix timestamp of exemplars in `CounterVec`. #710

## 1.4.0 / 2020-01-27

* [CHANGE] Go collector: Improve doc string for `go_gc_duration_seconds`. #702
* [FEATURE] Support a subset of OpenMetrics, including exemplars. Needs opt-in via `promhttp.HandlerOpts`. **EXPERIMENTAL** #706
* [FEATURE] Add `testutil.CollectAndCount`. #703

## 1.3.0 / 2019-12-21

* [FEATURE] Support tags in Graphite bridge. #668
* [BUGFIX] API client: Actually return Prometheus warnings. #699

## 1.2.1 / 2019-10-17

* [BUGFIX] Fix regression in the implementation of `Registerer.Unregister`. #663

## 1.2.0 / 2019-10-15

* [FEATURE] Support pushing to Pushgateway v0.10+. #652
* [ENHANCEMENT] Improve hashing to make a spurious `AlreadyRegisteredError` less likely to occur. #657
* [ENHANCEMENT] API client: Add godoc examples. #630
* [BUGFIX] promhttp: Correctly call WriteHeader in HTTP middleware. #634

## 1.1.0 / 2019-08-01

* [CHANGE] API client: Format time as UTC rather than RFC3339Nano. #617
* [CHANGE] API client: Add warnings to `LabelValues` and `LabelNames` calls. #609
* [FEATURE] Push: Support base64 encoding in grouping key. #624
* [FEATURE] Push: Add Delete method to Pusher. #613

## 1.0.0 / 2019-06-15

_This release removes all previously deprecated features, resulting in the breaking changes listed below. As this is v1.0.0, semantic versioning applies from now on, with the exception of the API client and parts marked explicitly as experimental._

* [CHANGE] Remove objectives from the default `Summary`. (Objectives have to be set explicitly in the `SummaryOpts`.) #600
* [CHANGE] Remove all HTTP related feature in the `prometheus` package. (Use the `promhttp` package instead.)  #600
* [CHANGE] Remove `push.FromGatherer`, `push.AddFromGatherer`, `push.Collectors`. (Use `push.New` instead.) #600
* [CHANGE] API client: Pass warnings through on non-error responses. #599
* [CHANGE] API client: Add warnings to `Series` call. #603
* [FEATURE] Make process collector work on Microsoft Windows. **EXPERIMENTAL** #596
* [FEATURE] API client: Add `/labels` call. #604
* [BUGFIX] Make `AlreadyRegisteredError` usable for wrapped registries. #607

## 0.9.4 / 2019-06-07

* [CHANGE] API client: Switch to alert values as strings. #585
* [FEATURE] Add a collector for Go module build information. #595
* [FEATURE] promhttp: Add an counter for internal errors during HTTP exposition. #594
* [FEATURE] API client: Support target metadata API. #590
* [FEATURE] API client: Support storage warnings. #562
* [ENHANCEMENT] API client: Improve performance handling JSON. #570
* [BUGFIX] Reduce test flakiness. #573

## 0.9.3 / 2019-05-16

* [CHANGE] Required Go version is now 1.9+. #561
* [FEATURE] API client: Add POST with get fallback for Query/QueryRange. #557
* [FEATURE] API client: Add alerts endpoint. #552
* [FEATURE] API client: Add rules endpoint. #508
* [FEATURE] push: Add option to pick metrics format. #540
* [ENHANCEMENT] Limit time the Go collector may take to collect memstats,
  returning results from the previous collection in case of a timeout. #568
* [ENHANCEMENT] Pusher now requires only a thin interface instead of a full
  `http.Client`, facilitating mocking and custom HTTP client implementation.
  #559
* [ENHANCEMENT] Memory usage improvement for histograms and summaries without
  objectives. #536
* [ENHANCEMENT] Summaries without objectives are now lock-free. #521
* [BUGFIX] promhttp: `InstrumentRoundTripperTrace` now takes into account a pre-set context. #582
* [BUGFIX] `TestCounterAddLarge` now works on all platforms. #567
* [BUGFIX] Fix `promhttp` examples. #535 #544
* [BUGFIX] API client: Wait for done before writing to shared response
  body. #532
* [BUGFIX] API client: Deal with discovered labels properly. #529

## 0.9.2 / 2018-12-06

* [FEATURE] Support for Go modules. #501
* [FEATURE] `Timer.ObserveDuration` returns observed duration. #509
* [ENHANCEMENT] Improved doc comments and error messages. #504
* [BUGFIX] Fix race condition during metrics gathering. #512
* [BUGFIX] Fix testutil metric comparison for Histograms and empty labels. #494
  #498

## 0.9.1 / 2018-11-03

* [FEATURE] Add `WriteToTextfile` function to facilitate the creation of
  *.prom files for the textfile collector of the node exporter. #489
* [ENHANCEMENT] More descriptive error messages for inconsistent label
  cardinality. #487
* [ENHANCEMENT] Exposition: Use a GZIP encoder pool to avoid allocations in
  high-frequency scrape scenarios. #366
* [ENHANCEMENT] Exposition: Streaming serving of metrics data while encoding.
  #482
* [ENHANCEMENT] API client: Add a way to return the body of a 5xx response.
  #479

## 0.9.0 / 2018-10-15

* [CHANGE] Go1.6 is no longer supported.
* [CHANGE] More refinements of the `Registry` consistency checks: Duplicated
  labels are now detected, but inconsistent label dimensions are now allowed.
  Collisions with the “magic” metric and label names in Summaries and
  Histograms are detected now. #108 #417 #471
* [CHANGE] Changed `ProcessCollector` constructor. #219
* [CHANGE] Changed Go counter `go_memstats_heap_released_bytes_total` to gauge
  `go_memstats_heap_released_bytes`. #229
* [CHANGE] Unexported `LabelPairSorter`. #453
* [CHANGE] Removed the `Untyped` metric from direct instrumentation. #340
* [CHANGE] Unexported `MetricVec`. #319
* [CHANGE] Removed deprecated `Set` method from `Counter` #247
* [CHANGE] Removed deprecated `RegisterOrGet` and `MustRegisterOrGet`. #247
* [CHANGE] API client: Introduced versioned packages.
* [FEATURE] A `Registerer` can be wrapped with prefixes and labels. #357
* [FEATURE] “Describe by collect” helper function. #239
* [FEATURE] Added package `testutil`. #58
* [FEATURE] Timestamp can be explicitly set for const metrics. #187
* [FEATURE] “Unchecked” collectors are possible now without cheating. #47
* [FEATURE] Pushing to the Pushgateway reworked in package `push` to support
  many new features. (The old functions are still usable but deprecated.) #372
  #341
* [FEATURE] Configurable connection limit for scrapes. #179
* [FEATURE] New HTTP middlewares to instrument `http.Handler` and
  `http.RoundTripper`. The old middlewares and the pre-instrumented `/metrics`
  handler are (strongly) deprecated. #316 #57 #101 #224
* [FEATURE] “Currying” for metric vectors. #320
* [FEATURE] A `Summary` can be created without quantiles. #118
* [FEATURE] Added a `Timer` helper type. #231
* [FEATURE] Added a Graphite bridge. #197
* [FEATURE] Help strings are now optional. #460
* [FEATURE] Added `process_virtual_memory_max_bytes` metric. #438 #440
* [FEATURE] Added `go_gc_cpu_fraction` and `go_threads` metrics. #281 #277
* [FEATURE] Added `promauto` package with auto-registering metrics. #385 #393
* [FEATURE] Add `SetToCurrentTime` method to `Gauge`. #259
* [FEATURE] API client: Add AlertManager, Status, and Target methods. #402
* [FEATURE] API client: Add admin methods. #398
* [FEATURE] API client: Support series API. #361
* [FEATURE] API client: Support querying label values.
* [ENHANCEMENT] Smarter creation of goroutines during scraping. Solves memory
  usage spikes in certain situations. #369
* [ENHANCEMENT] Counters are now faster if dealing with integers only. #367
* [ENHANCEMENT] Improved label validation. #274 #335
* [BUGFIX] Creating a const metric with an invalid `Desc` returns an error. #460
* [BUGFIX] Histogram observations don't race any longer with exposition. #275
* [BUGFIX] Fixed goroutine leaks. #236 #472
* [BUGFIX] Fixed an error message for exponential histogram buckets. #467
* [BUGFIX] Fixed data race writing to the metric map. #401
* [BUGFIX] API client: Decode JSON on a 4xx response but do not on 204
  responses. #476 #414

## 0.8.0 / 2016-08-17

* [CHANGE] Registry is doing more consistency checks. This might break
  existing setups that used to export inconsistent metrics.
* [CHANGE] Pushing to Pushgateway moved to package `push` and changed to allow
  arbitrary grouping.
* [CHANGE] Removed `SelfCollector`.
* [CHANGE] Removed `PanicOnCollectError` and `EnableCollectChecks` methods.
* [CHANGE] Moved packages to the prometheus/common repo: `text`, `model`,
  `extraction`.
* [CHANGE] Deprecated a number of functions.
* [FEATURE] Allow custom registries. Added `Registerer` and `Gatherer`
  interfaces.
* [FEATURE] Separated HTTP exposition, allowing custom HTTP handlers (package
  `promhttp`) and enabling the creation of other exposition mechanisms.
* [FEATURE] `MustRegister` is variadic now, allowing registration of many
  collectors in one call.
* [FEATURE] Added HTTP API v1 package.
* [ENHANCEMENT] Numerous documentation improvements.
* [ENHANCEMENT] Improved metric sorting.
* [ENHANCEMENT] Inlined fnv64a hashing for improved performance.
* [ENHANCEMENT] Several test improvements.
* [BUGFIX] Handle collisions in MetricVec.

## 0.7.0 / 2015-07-27

* [CHANGE] Rename ExporterLabelPrefix to ExportedLabelPrefix.
* [BUGFIX] Closed gaps in metric consistency check.
* [BUGFIX] Validate LabelName/LabelSet on JSON unmarshaling.
* [ENHANCEMENT] Document the possibility to create "empty" metrics in
  a metric vector.
* [ENHANCEMENT] Fix and clarify various doc comments and the README.md.
* [ENHANCEMENT] (Kind of) solve "The Proxy Problem" of http.InstrumentHandler.
* [ENHANCEMENT] Change responseWriterDelegator.written to int64.

## 0.6.0 / 2015-06-01

* [CHANGE] Rename process_goroutines to go_goroutines.
* [ENHANCEMENT] Validate label names during YAML decoding.
* [ENHANCEMENT] Add LabelName regular expression.
* [BUGFIX] Ensure alignment of struct members for 32-bit systems.

## 0.5.0 / 2015-05-06

* [BUGFIX] Removed a weakness in the fingerprinting aka signature code.
  This makes fingerprinting slower and more allocation-heavy, but the
  weakness was too severe to be tolerated.
* [CHANGE] As a result of the above, Metric.Fingerprint is now returning
  a different fingerprint. To keep the same fingerprint, the new method
  Metric.FastFingerprint was introduced, which will be used by the
  Prometheus server for storage purposes (implying that a collision
  detection has to be added, too).
* [ENHANCEMENT] The Metric.Equal and Metric.Before do not depend on
  fingerprinting anymore, removing the possibility of an undetected
  fingerprint collision.
* [FEATURE] The Go collector in the exposition library includes garbage
  collection stats.
* [FEATURE] The exposition library allows to create constant "throw-away"
  summaries and histograms.
* [CHANGE] A number of new reserved labels and prefixes.

## 0.4.0 / 2015-04-08

* [CHANGE] Return NaN when Summaries have no observations yet.
* [BUGFIX] Properly handle Summary decay upon Write().
* [BUGFIX] Fix the documentation link to the consumption library.
* [FEATURE] Allow the metric family injection hook to merge with existing
  metric families.
* [ENHANCEMENT] Removed cgo dependency and conditional compilation of procfs.
* [MAINTENANCE] Adjusted to changes in matttproud/golang_protobuf_extensions.

## 0.3.2 / 2015-03-11

* [BUGFIX] Fixed the receiver type of COWMetric.Set(). This method is
  only used by the Prometheus server internally.
* [CLEANUP] Added licenses of vendored code left out by godep.

## 0.3.1 / 2015-03-04

* [ENHANCEMENT] Switched fingerprinting functions from own free list to
  sync.Pool.
* [CHANGE] Makefile uses Go 1.4.2 now (only relevant for examples and tests).

## 0.3.0 / 2015-03-03

* [CHANGE] Changed the fingerprinting for metrics. THIS WILL INVALIDATE ALL
  PERSISTED FINGERPRINTS. IF YOU COMPILE THE PROMETHEUS SERVER WITH THIS
  VERSION, YOU HAVE TO WIPE THE PREVIOUSLY CREATED STORAGE.
* [CHANGE] LabelValuesToSignature removed. (Nobody had used it, and it was
  arguably broken.)
* [CHANGE] Vendored dependencies. Those are only used by the Makefile. If
  client_golang is used as a library, the vendoring will stay out of your way.
* [BUGFIX] Remove a weakness in the fingerprinting for metrics. (This made
  the fingerprinting change above necessary.)
* [FEATURE] Added new fingerprinting functions SignatureForLabels and
  SignatureWithoutLabels to be used by the Prometheus server. These functions
  require fewer allocations than the ones currently used by the server.

## 0.2.0 / 2015-02-23

* [FEATURE] Introduce new Histagram metric type.
* [CHANGE] Ignore process collector errors for now (better error handling
  pending).
* [CHANGE] Use clear error interface for process pidFn.
* [BUGFIX] Fix Go download links for several archs and OSes.
* [ENHANCEMENT] Massively improve Gauge and Counter performance.
* [ENHANCEMENT] Catch illegal label names for summaries in histograms.
* [ENHANCEMENT] Reduce allocations during fingerprinting.
* [ENHANCEMENT] Remove cgo dependency. procfs package will only be included if
  both cgo is available and the build is for an OS with procfs.
* [CLEANUP] Clean up code style issues.
* [CLEANUP] Mark slow test as such and exclude them from travis.
* [CLEANUP] Update protobuf library package name.
* [CLEANUP] Updated vendoring of beorn7/perks.

## 0.1.0 / 2015-02-02

* [CLEANUP] Introduced semantic versioning and changelog. From now on,
  changes will be reported in this file.
