*Apr 26, 2023*

This release bumps the supported version of RocksDB, which requires cometbft-db
RocksDB users to update their builds (and hence requires a "major" release, but
does not introduce any other breaking changes). Special thanks to @yihuang for
this update!

While the minimum supported version of the Go compiler was bumped to 1.19, no
1.19-specific code changes were introduced and this should, therefore, still be
able to be compiled with earlier versions of Go. It is, however, recommended to
upgrade to the latest version(s) of Go ASAP.
