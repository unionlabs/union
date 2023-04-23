# Performance

After some discussion with Jae on the usability, it seems performance is a big concern.  If every write takes around 1ms, that puts a serious upper limit on the speed of the consensus engine (especially since with the check/tx dichotomy, we need at least two writes (to cache, only one to disk) and likely two or more queries to handle any transaction).

As Jae notes: for CheckTx, a copy of IAVLTree doesn't need to be saved. During CheckTx it'll load inner nodes into the cache. The cache is shared w/ the AppendTx state IAVLTree, so during AppendTx we should save some time. There would only be 1 set of writes. Also, there's quite a bit of free time in between blocks as provided by Tendermint, during which CheckTx can run priming the cache, so hopefully this helps as well.

Jae: That said, I'm not sure exactly what the tx throughput would be during normal running times. I'm hoping that we can have e.g. 3 second blocks w/ say over a hundred txs per sec per block w/ 1 million items. That will get us through for some time, but that time is limited.

Ethan: I agree, and think this works now with goleveldb backing on most host machines.  For public chains, maybe it is desired to push 1000 tx every 3 sec to a block, with a db size of 1 billion items.  10x the throughput with 1000x the data.  That could be a long-term goal, and would scale to the cosmos and beyond.

## Plan

For any goal, we need some clear steps.

1) Cleanup code, and write some more benchmark cases to capture "realistic" usage
2) Run tests on various hardware to see the best performing backing stores
3) Do profiling on the best performance to see if there are any easy performance gains
4) (Possibly) Write another implementation of merkle.Tree to improve all the memory overhead, consider CPU cache, etc....
5) (Possibly) Write another backend datastore to persist the tree in a more efficient way

The rest of this document is the planned or completed actions for the above-listed steps.

## Cleanup

Done in branch `cleanup_deps`:
  * Fixed up dependeny management (tmlibs/db etc in glide/vendor)
  * Updated Makefile (test, bench, get_deps)
  * Fixed broken code - `looper.go` and one benchmark didn't run

Benchmarks should be parameterized on:
  1) storage implementation
  2) initial data size
  3) length of keys
  4) length of data
  5) block size (frequency of copy/hash...)
Thus, we would see the same benchmark run against memdb with 100K items, goleveldb with 100K, leveldb with 100K, memdb with 10K, goleveldb with 10K...

Scenarios to run after db is set up.
  * Pure query time (known/hits, vs. random/misses)
  * Write timing (known/updates, vs. random/inserts)
  * Delete timing (existing keys only)
  * TMSP Usage:
    * For each block size:
      * 2x copy "last commit" -> check and real
      * repeat for each tx:
        * (50% update + 50% insert?)
        * query + insert/update in check
        * query + insert/update in real
      * get hash
      * save real
      * real -> "last commit"


## Benchmarks

After writing the benchmarks, we can run them under various environments and store the results under benchmarks directory.  Some useful environments to test:

  * Dev machines
  * Digital ocean small/large machine
  * Various AWS setups

Please run the benchmark on more machines and add the result.  Just type: `make record` in the directory and wait a (long) while (with little other load on the machine).

This will require also a quick setup script to install go and run tests in these environments.  Maybe some scripts even. Also, this will produce a lot of files and we may have to graph them to see something useful...

But for starting, my laptop, and one digital ocean and one aws server should be sufficient. At least to find the winner, before profiling.


## Profiling

Once we figure out which current implementation looks fastest, let's profile it to make it even faster.  It is great to optimize the memdb code to really speed up the hashing and tree-building logic.  And then focus on the backend implementation to optimize the disk storage, which will be the next major pain point.

Some guides:

  * [Profiling benchmarks locally](https://medium.com/@hackintoshrao/daily-code-optimization-using-benchmarks-and-profiling-in-golang-gophercon-india-2016-talk-874c8b4dc3c5#.jmnd8w2qr)
  * [On optimizing memory](https://signalfx.com/blog/a-pattern-for-optimizing-go-2/)
  * [Profiling running programs](http://blog.ralch.com/tutorial/golang-performance-and-memory-analysis/)
  * [Dave Chenny's profiler pkg](https://github.com/pkg/profile)

Some ideas for speedups:

  * [Speedup SHA256 100x on ARM](https://blog.minio.io/accelerating-sha256-by-100x-in-golang-on-arm-1517225f5ff4#.pybt7bb3w)
  * [Faster SHA256 golang implementation](https://github.com/minio/sha256-simd)
  * [Data structure alignment](http://stackoverflow.com/questions/39063530/optimising-datastructure-word-alignment-padding-in-golang)
  * [Slice alignment](http://blog.chewxy.com/2016/07/25/on-the-memory-alignment-of-go-slice-values/)
  * [Tool to analyze your structs](https://github.com/dominikh/go-structlayout)

## Tree Re-implementation

If we want to copy lots of objects, it becomes better to think of using memcpy on large (eg. 4-16KB) buffers than copying individual structs.  We also could allocate arrays of structs and align them to remove a lot of memory management and gc overhead. That means going down to some C-level coding...

Some links for thought:

  * [Array representation of a binary tree](http://www.cse.hut.fi/en/research/SVG/TRAKLA2/tutorials/heap_tutorial/taulukkona.html)
  * [Memcpy buffer size timing](http://stackoverflow.com/questions/21038965/why-does-the-speed-of-memcpy-drop-dramatically-every-4kb)
  * [Calling memcpy from go](https://github.com/jsgilmore/shm/blob/master/memcpy.go)
  * [Unsafe docs](https://godoc.org/unsafe)
  * [...and how to use it](https://copyninja.info/blog/workaround-gotypesystems.html)
  * [Or maybe just plain copy...](https://godoc.org/builtin#copy)

## Backend implementation

Storing each link in the tree in leveldb treats each node as an isolated item.  Since we know some usage patterns (when a parent is hit, very likely one child will be hit), we could try to organize the memory and disk location of the nodes ourselves to make it more efficient.  Or course, this could be a long, slippery slope.

Inspired by the [Array representation](http://www.cse.hut.fi/en/research/SVG/TRAKLA2/tutorials/heap_tutorial/taulukkona.html) link above, we could consider other layouts for the nodes. For example, rather than store them alone, or the entire tree in one big array, the nodes could be placed in groups of 15 based on the parent (parent and 3 generations of children).  Then we have 4 levels before jumping to another location.  Maybe we just store this larger chunk as one leveldb location, or really try to do the mmap ourselves...

In any case, assuming around 100 bytes for one non-leaf node (3 sha hashes, plus prefix, plus other data), 15 nodes would be a little less than 2K, maybe even go one more level to 31 nodes and 3-4KB, where we could take best advantage of the memory/disk page size.

Some links for thought:

  * [Memory mapped files](https://github.com/edsrzf/mmap-go)
