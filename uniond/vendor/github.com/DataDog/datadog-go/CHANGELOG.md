## Changes

[//]: # (comment: Don't forget to update statsd/telemetry.go:clientVersionTelemetryTag when releasing a new version)

# 4.8.3 / 2021-10-27

* [BUGFIX] Fix `Event.Check` method: text is no longer required. See [#237][].

# 4.8.2 / 2021-09-06

* [BETA][BUGFIX] Fix race condition in aggregation where two sample could overwrite each other when sampled for the first time. See [#225][]

# 4.8.1 / 2021-07-09

* [BUGFIX] Prevent telemetry from using the client global namespace. See [#205][]
* [BETA][BUGFIX] Fix timings having a different precision with and without extended aggregation. See [#204][]

# 4.8.0 / 2021-06-14

* [BETA][IMPROVEMENT] Reduce aggregation default window to 2s to reduce sampling aliasing. See [#199][]
* [IMPROVEMENT] Automatically add a "\n" after each metric so the agent can determine if a metric is truncated. Per source EOL detection was made available in agent 7.28 with the `dogstatsd_eol_required` setting. See [#198][]

# 4.7.0 / 2021-05-05

* [BETA] Increase the number of workers in the aggregator when using channelMode with extended aggregation to have
  similar performance than channelMode without aggregation. See [#195][].

# 4.6.1 / 2021-04-30

* [BETA BUGFIX] Fix telemetry with extended aggregation and channelMode. See [#194][].

# 4.6.0 / 2021-04-16

* [BETA] Support sample rate and channel mode for extended aggregation (ie: histograms, distributions and timings). See [#187][].

# 4.5.1 / 2021-03-31

* [BUGFIX] Fix support of UDS and named pipe for DD_AGENT_HOST environment variable. See [#192][].

# 4.5.0 / 2021-03-15

* [IMPROVEMENT] Increase UDS default timeout from 1ms to 100ms. See [#186][].
* [IMPROVEMENT] Defer connection establishment to first write for Windows Named Pipe. See [#190][].

# 4.4.0 / 2021-02-10

* [BETA BUGFIX] Fix multi-metric aggregation when packing different metrics in the same packet. See [#181][].
* [FEATURE] Add support for Windows Named Pipes (Windows only). See [#182][] and [#185][].

# 4.3.1 / 2021-01-28

* [BUGFIX] Fix race condition when using sample rate (introduce in 4.3.0). See [#179][].

# 4.3.0 / 2021-01-20

* [BETA] Adding client side aggregation for distributions, histograms and timings. See [#176][].
* [IMPROVEMENT] Use a worker-specific random source to remove lock contention. See [#178][]. Thanks to [@matthewdale][].
* [IMPROVEMENT] Update devMode telemetry naming and taging to ease graphing in Datadog. See [#175][].

# 4.2.0 / 2020-11-02

* [UDS] Use better payload size defaults for UDS connections. See [#171][].

# 4.1.0 / 2020-10-23

[BETA BUGFIX] Ignore sampling rate when client side aggregation is enabled (for Gauge, Count and Set). See [#170][].
[FEATURE] Adding a new option `WithDevMode()`, to send more telemetry metrics to ease troubleshooting issues. See [#169][].


# 4.0.1 / 2020-10-07

### Notes

* [BUGFIX] Fix incomplete manual flush of the sender when the client isn't stopped. See [#163][].

# 4.0.0 / 2020-08-21

### Notes

* [FEATURE] Add new option `WithTelemetryAddr`, to send the telemetry data to a different endpoint. See [#157][].
* [BUGFIX] Fix race condition in the flush mechanism of the aggregator. See [#166][]. Thanks to [@cyx][].

### Breaking changes

- Dropping support for EOL versions of Golang 1.11 and lower.

# 3.7.2 / 2020-06-16

### Notes

* [BUGFIX] Fix panic on 32bits and ARM when using the telemetry. See [#156][].
* [BETA BUGFIX] Fix typo in method name to configure the aggregation window interval. See [#154][].

# 3.7.1 / 2020-05-01

### Notes

* [BUGFIX] Fix panic when calling CloneWithExtraOptions with a nil client. See [#148][].

# 3.7.0 / 2020-04-29

### Notes

* [FEATURE] Add new function to clone a Client, so library can inherit and extend options from the main application. See [#147][].
* [IMPROVEMENT] Auto append a '.' when needed to namespace. See [#145][]. Thanks to [@kamatama41][].
* [IMPROVEMENT] Add the client global tags to the telemetry tags. See [#143][]. Thanks to [@chrisleavoy][].

# 3.6.0 / 2020-04-21

### Notes

* [IMPROVEMENT] Reduce lock contention by sharding worker by metric name. See [#108][].
* [FEATURE] Adding a "channel mode" to send metrics to the client, disable by default. See [#134][].
* [BUGFIX] Fix metrics not being flushed when the client is closed. See [#144][].
* [BETA] Adding client side aggregation for Gauge, Count and Set. See [#139][].

# 3.5.0 / 2020-03-17

### Notes

* [IMPROVEMENT] Add support for `DD_ENV`, `DD_SERVICE`, and `DD_VERSION` to set global tags for `env`, `service` and `version`/ See [#137][]

# 3.4.1 / 2020-03-10

### Notes

* [BUGFIX] Fix possible deadlock when closing the client. See [#135][]. Thanks to [@danp60][].

# 3.4.0 / 2020-01-15

### Notes

* [IMPROVEMENT] Improve tags for the telemetry. See [#118][].
* [IMPROVEMENT] Add option to disable the telemetry. See [#117][].
* [IMPROVEMENT] Add metrics, event and service check count to the telemetry. See [#118][].

# 3.3.1 / 2019-12-13

### Notes

* [BUGFIX] Fix Unix domain socket path extraction. See [#113][].
* [BUGFIX] Fix an issue with custom writers leading to metric drops. See [#106][].
* [BUGFIX] Fix an error check in uds.Write leading to unneeded re-connections. See [#115][].

# 3.3.0 / 2019-12-02

### Notes

* [BUGFIX] Close the stop channel when closing a statsd client to avoid leaking. See [#107][].

# 3.2.0 / 2019-10-28

### Notes

* [IMPROVEMENT] Add all `Client` public methods to the `ClientInterface` and `NoOpClient`. See [#100][]. Thanks [@skaji][].

# 3.1.0 / 2019-10-24

### Notes

* [FEATURE] Add a noop client. See [#92][]. Thanks [@goodspark][].

# 3.0.0 / 2019-10-18

### Notes

* [FEATURE] Add a way to configure the maximum size of a single payload (was always 1432, the optimal size for local UDP). See [#91][].
* [IMPROVEMENT] Various performance improvements. See [#91][].
* [OTHER] The client now pre-allocates 4MB of memory to queue up metrics. This can be controlled using the [WithBufferPoolSize](https://godoc.org/github.com/DataDog/datadog-go/statsd#WithBufferPoolSize) option.

### Breaking changes

- Sending a metric over UDS won't return an error if we fail to forward the datagram to the agent. We took this decision for two main reasons:
  - This made the UDS client blocking by default which is not desirable
  - This design was flawed if you used a buffer as only the call that actually sent the buffer would return an error
- The `Buffered` option has been removed as the client can only be buffered. If for some reason you need to have only one dogstatsd message per payload you can still use the `WithMaxMessagesPerPayload` option set to 1.
- The `AsyncUDS` option has been removed as the networking layer is now running in a separate Goroutine.

# 2.3.0 / 2019-10-15

### Notes

 * [IMPROVEMENT] Use an error constant for "nil client" errors. See [#90][]. Thanks [@asf-stripe][].

# 2.2.0 / 2019-04-11

### Notes

 * [FEATURE] UDS: non-blocking implementation. See [#81][].
 * [FEATURE] Support configuration from standard environment variables. See [#78][].
 * [FEATURE] Configuration at client creation. See [#82][].
 * [IMPROVEMENT] UDS: change Mutex to RWMutex for fast already-connected path. See [#84][]. Thanks [@KJTsanaktsidis][].
 * [IMPROVEMENT] Return error when using on nil client. See [#65][]. Thanks [@Aceeri][].
 * [IMPROVEMENT] Reduce `Client.format` allocations. See [#53][]. Thanks [@vcabbage][].
 * [BUGFIX] UDS: add lock to writer for concurrency safety. See [#62][].
 * [DOCUMENTATION] Document new options, non-blocking client, etc. See [#85][].
 * [TESTING] Adding go 1.10 and go 1.11 to CI. See [#75][]. Thanks [@thedevsaddam][].

# 2.1.0 / 2018-03-30

### Notes

 * [IMPROVEMENT] Protect client methods from nil client. See [#52][], thanks [@ods94065][].

# 2.0.0 / 2018-01-29

### Details

Version `2.0.0` contains breaking changes and beta features, please refer to the
_Notes_ section below for details.

### Notes

 * [BREAKING] `statsdWriter` now implements io.Writer interface. See [#46][].
 * [BUGFIX] Flush buffer on close. See [#47][].
 * [BETA] Add support for global distributions. See [#45][].
 * [FEATURE] Add support for Unix Domain Sockets. See [#37][].
 * [FEATURE] Export `eventAlertType` and `eventPriority`. See [#42][], thanks [@thomas91310][].
 * [FEATURE] Export `Flush` method. See [#40][], thanks [@colega][].
 * [BUGFIX] Prevent panics when closing the `udsWriter`. See [#43][], thanks [@jacek-adamek][].
 * [IMPROVEMENT] Fix issues reported by Golint. See [#39][], thanks [@tariq1890][].
 * [IMPROVEMENT] Improve message building speed by using less `fmt.Sprintf`s. See [#32][], thanks [@corsc][].

# 1.1.0 / 2017-04-27

### Notes

* [FEATURE] Export serviceCheckStatus allowing interfaces to statsd.Client. See [#19][] (Thanks [@Jasrags][])
* [FEATURE] Client.sendMsg(). Check payload length for all messages. See [#25][] (Thanks [@theckman][])
* [BUGFIX] Remove new lines from tags. See [#21][] (Thanks [@sjung-stripe][])
* [BUGFIX] Do not panic on Client.Event when `nil`. See [#28][]
* [DOCUMENTATION] Update `decr` documentation to match implementation. See [#30][] (Thanks [@kcollasarundell][])

# 1.0.0 / 2016-08-22

### Details

We hadn't been properly versioning this project. We will begin to do so with this
`1.0.0` release. We had some contributions in the past and would like to thank the
contributors [@aviau][], [@sschepens][], [@jovanbrakus][],  [@abtris][], [@tummychow][], [@gphat][], [@diasjorge][],
[@victortrac][], [@seiffert][] and [@w-vi][], in no particular order, for their work.

Below, for reference, the latest improvements made in 07/2016 - 08/2016

### Notes

* [FEATURE] Implemented support for service checks. See [#17][] and [#5][]. (Thanks [@jovanbrakus][] and [@diasjorge][]).
* [FEATURE] Add Incr, Decr, Timing and more docs.. See [#15][]. (Thanks [@gphat][])
* [BUGFIX] Do not append to shared slice. See [#16][]. (Thanks [@tummychow][])

<!--- The following link definition list is generated by PimpMyChangelog --->
[#5]: https://github.com/DataDog/datadog-go/issues/5
[#15]: https://github.com/DataDog/datadog-go/issues/15
[#16]: https://github.com/DataDog/datadog-go/issues/16
[#17]: https://github.com/DataDog/datadog-go/issues/17
[#19]: https://github.com/DataDog/datadog-go/issues/19
[#21]: https://github.com/DataDog/datadog-go/issues/21
[#25]: https://github.com/DataDog/datadog-go/issues/25
[#28]: https://github.com/DataDog/datadog-go/issues/28
[#30]: https://github.com/DataDog/datadog-go/issues/30
[#32]: https://github.com/DataDog/datadog-go/issues/32
[#37]: https://github.com/DataDog/datadog-go/issues/37
[#39]: https://github.com/DataDog/datadog-go/issues/39
[#40]: https://github.com/DataDog/datadog-go/issues/40
[#42]: https://github.com/DataDog/datadog-go/issues/42
[#43]: https://github.com/DataDog/datadog-go/issues/43
[#45]: https://github.com/DataDog/datadog-go/issues/45
[#46]: https://github.com/DataDog/datadog-go/issues/46
[#47]: https://github.com/DataDog/datadog-go/issues/47
[#52]: https://github.com/DataDog/datadog-go/issues/52
[#53]: https://github.com/DataDog/datadog-go/issues/53
[#62]: https://github.com/DataDog/datadog-go/issues/62
[#65]: https://github.com/DataDog/datadog-go/issues/65
[#75]: https://github.com/DataDog/datadog-go/issues/75
[#78]: https://github.com/DataDog/datadog-go/issues/78
[#81]: https://github.com/DataDog/datadog-go/issues/81
[#82]: https://github.com/DataDog/datadog-go/issues/82
[#84]: https://github.com/DataDog/datadog-go/issues/84
[#85]: https://github.com/DataDog/datadog-go/issues/85
[#90]: https://github.com/DataDog/datadog-go/issues/90
[#91]: https://github.com/DataDog/datadog-go/issues/91
[#92]: https://github.com/DataDog/datadog-go/issues/92
[#100]: https://github.com/DataDog/datadog-go/issues/100
[#106]: https://github.com/DataDog/datadog-go/issues/106
[#107]: https://github.com/DataDog/datadog-go/issues/107
[#113]: https://github.com/DataDog/datadog-go/issues/113
[#117]: https://github.com/DataDog/datadog-go/issues/117
[#118]: https://github.com/DataDog/datadog-go/issues/118
[#115]: https://github.com/DataDog/datadog-go/issues/115
[#135]: https://github.com/DataDog/datadog-go/issues/135
[#137]: https://github.com/DataDog/datadog-go/issues/137
[#108]: https://github.com/DataDog/datadog-go/pull/108
[#134]: https://github.com/DataDog/datadog-go/pull/134
[#139]: https://github.com/DataDog/datadog-go/pull/139
[#143]: https://github.com/DataDog/datadog-go/pull/143
[#144]: https://github.com/DataDog/datadog-go/pull/144
[#145]: https://github.com/DataDog/datadog-go/pull/145
[#147]: https://github.com/DataDog/datadog-go/pull/147
[#148]: https://github.com/DataDog/datadog-go/pull/148
[#154]: https://github.com/DataDog/datadog-go/pull/154
[#156]: https://github.com/DataDog/datadog-go/pull/156
[#157]: https://github.com/DataDog/datadog-go/pull/157
[#163]: https://github.com/DataDog/datadog-go/pull/163
[#169]: https://github.com/DataDog/datadog-go/pull/169
[#170]: https://github.com/DataDog/datadog-go/pull/170
[#171]: https://github.com/DataDog/datadog-go/pull/171
[#175]: https://github.com/DataDog/datadog-go/pull/175
[#176]: https://github.com/DataDog/datadog-go/pull/176
[#178]: https://github.com/DataDog/datadog-go/pull/178
[#179]: https://github.com/DataDog/datadog-go/pull/179
[#181]: https://github.com/DataDog/datadog-go/pull/181
[#182]: https://github.com/DataDog/datadog-go/pull/182
[#185]: https://github.com/DataDog/datadog-go/pull/185
[#186]: https://github.com/DataDog/datadog-go/pull/186
[#187]: https://github.com/DataDog/datadog-go/pull/187
[#190]: https://github.com/DataDog/datadog-go/pull/190
[#192]: https://github.com/DataDog/datadog-go/pull/192
[#194]: https://github.com/DataDog/datadog-go/pull/194
[#195]: https://github.com/DataDog/datadog-go/pull/195
[#198]: https://github.com/DataDog/datadog-go/pull/198
[#199]: https://github.com/DataDog/datadog-go/pull/199
[#204]: https://github.com/DataDog/datadog-go/pull/204
[#205]: https://github.com/DataDog/datadog-go/pull/205
[#225]: https://github.com/DataDog/datadog-go/pull/225
[#237]: https://github.com/DataDog/datadog-go/pull/237
[@Aceeri]: https://github.com/Aceeri
[@Jasrags]: https://github.com/Jasrags
[@KJTsanaktsidis]: https://github.com/KJTsanaktsidis
[@abtris]: https://github.com/abtris
[@aviau]: https://github.com/aviau
[@colega]: https://github.com/colega
[@corsc]: https://github.com/corsc
[@diasjorge]: https://github.com/diasjorge
[@gphat]: https://github.com/gphat
[@jacek-adamek]: https://github.com/jacek-adamek
[@jovanbrakus]: https://github.com/jovanbrakus
[@kcollasarundell]: https://github.com/kcollasarundell
[@ods94065]: https://github.com/ods94065
[@seiffert]: https://github.com/seiffert
[@sjung-stripe]: https://github.com/sjung-stripe
[@sschepens]: https://github.com/sschepens
[@tariq1890]: https://github.com/tariq1890
[@theckman]: https://github.com/theckman
[@thedevsaddam]: https://github.com/thedevsaddam
[@thomas91310]: https://github.com/thomas91310
[@tummychow]: https://github.com/tummychow
[@vcabbage]: https://github.com/vcabbage
[@victortrac]: https://github.com/victortrac
[@w-vi]: https://github.com/w-vi
[@asf-stripe]: https://github.com/asf-stripe
[@goodspark]: https://github.com/goodspark
[@skaji]: https://github.com/skaji
[@danp60]: https://github.com/danp60
[@kamatama41]: https://github.com/kamatama41
[@chrisleavoy]: https://github.com/chrisleavoy
[@cyx]: https://github.com/cyx
[@matthewdale]: https://github.com/matthewdale
