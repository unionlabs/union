package main

import (
	"bytes"
	"fmt"
	"log"
	"os"
	"path"
	"sync"
	"sync/atomic"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/spf13/cobra"
)

var fsBenchCmd = &cobra.Command{
	Use:   "fs <dir>",
	Short: "Run file system benchmarks.",
	Long: `
Run file system benchmarks. Each benchmark is predefined and can be
run using the command "bench fs <dir> --bench-name <benchmark>".
Each possible <benchmark> which can be run is defined in the code.
Benchmarks may require the specification of a --duration or
--max-ops flag, to prevent the benchmark from running forever
or running out of memory.

The --num-times flag can be used to run the entire benchmark, more than
once. If the flag isn't provided, then the benchmark is only run once.
`,
	Args: cobra.ExactArgs(1),
	RunE: runFsBench,
}

const writeBatchSize = 1 << 10

var fsConfig struct {
	// An upper limit on the number of ops which can be run.
	maxOps int

	// Benchmark to run.
	benchname string

	// Number of times each benchmark should be run.
	numTimes int

	fs vfs.FS

	precomputedWriteBatch []byte
}

func init() {
	fsBenchCmd.Flags().IntVar(
		&fsConfig.maxOps, "max-ops", 0,
		"Maximum number of times the operation which is being benchmarked should be run.",
	)

	fsBenchCmd.Flags().StringVar(
		&fsConfig.benchname, "bench-name", "", "The benchmark to run.")
	fsBenchCmd.MarkFlagRequired("bench-name")

	fsBenchCmd.Flags().IntVar(
		&fsConfig.numTimes, "num-times", 1,
		"Number of times each benchmark should be run.")

	// Add subcommand to list
	fsBenchCmd.AddCommand(listFsBench)

	// Just use the default vfs implementation for now.
	fsConfig.fs = vfs.Default

	fsConfig.precomputedWriteBatch = bytes.Repeat([]byte("a"), writeBatchSize)
}

// State relevant to a benchmark.
type fsBench struct {
	// A short name for the benchmark.
	name string

	// A one line description for the benchmark.
	description string

	// numOps is the total number of ops which
	// have been run for the benchmark. This is used
	// to make sure that we don't benchmark the operation
	// more than max-ops times.
	numOps int

	// directory under which the benchmark is run.
	dir     vfs.File
	dirName string

	// Stats associated with the benchmark.
	reg *histogramRegistry

	// The operation which we're benchmarking. This
	// will be called over and over again.
	// Returns false if run should no longer be called.
	run func(*namedHistogram) bool

	// Stop the benchmark from executing any further.
	// Stop is safe to call concurrently with run.
	stop func()

	// A cleanup func which must be called after
	// the benchmark has finished running.
	// Clean should be only called after making sure
	// that the run function is no longer executing.
	clean func()
}

// createFile can be used to create an empty file.
// Invariant: File shouldn't already exist.
func createFile(filepath string) vfs.File {
	fh, err := fsConfig.fs.Create(filepath)
	if err != nil {
		log.Fatalln(err)
	}
	return fh
}

// Invariant: file with filepath should exist.
func deleteFile(filepath string) {
	err := fsConfig.fs.Remove(filepath)
	if err != nil {
		log.Fatalln(err)
	}
}

// Write size bytes to the file in batches.
func writeToFile(fh vfs.File, size int64) {
	for size > 0 {
		var toWrite []byte
		if size >= writeBatchSize {
			toWrite = fsConfig.precomputedWriteBatch
		} else {
			toWrite = fsConfig.precomputedWriteBatch[:size]
		}
		written, err := fh.Write(toWrite)
		if err != nil {
			log.Fatalln(err)
		}
		if written != len(toWrite) {
			log.Fatalf("Couldn't write %d bytes to file\n", size)
		}
		size -= int64(len(toWrite))
	}
}

func syncFile(fh vfs.File) {
	err := fh.Sync()
	if err != nil {
		log.Fatalln(err)
	}
}

func closeFile(fh vfs.File) {
	err := fh.Close()
	if err != nil {
		log.Fatalln(err)
	}
}

func getDiskUsage(filepath string) {
	_, err := fsConfig.fs.GetDiskUsage(filepath)
	if err != nil {
		log.Fatalln(err)
	}
}

func openDir(filepath string) vfs.File {
	fh, err := fsConfig.fs.OpenDir(filepath)
	if err != nil {
		log.Fatalln(err)
	}
	return fh
}

func mkDir(filepath string) {
	err := fsConfig.fs.MkdirAll(filepath, 0755)
	if err != nil {
		log.Fatalln(err)
	}
}

func removeAllFiles(filepath string) {
	err := fsConfig.fs.RemoveAll(filepath)
	if err != nil {
		log.Fatalln(err)
	}
}

// fileSize is in bytes.
func createBench(benchName string, benchDescription string) fsBenchmark {
	createBench := func(dirpath string) *fsBench {
		bench := &fsBench{}
		mkDir(dirpath)
		fh := openDir(dirpath)

		bench.dir = fh
		bench.dirName = dirpath
		bench.reg = newHistogramRegistry()
		bench.numOps = 0
		bench.name = benchName
		bench.description = benchDescription

		// setup the operation to benchmark, and the cleanup functions.
		pref := "temp_"
		var numFiles int
		var done atomic.Bool

		bench.run = func(hist *namedHistogram) bool {
			if done.Load() {
				return false
			}

			start := time.Now()
			fh := createFile(path.Join(dirpath, fmt.Sprintf("%s%d", pref, numFiles)))
			syncFile(bench.dir)
			hist.Record(time.Since(start))

			closeFile(fh)
			numFiles++
			return true
		}

		bench.stop = func() {
			done.Store(true)
		}

		bench.clean = func() {
			removeAllFiles(dirpath)
			closeFile(bench.dir)
		}

		return bench
	}

	return fsBenchmark{
		createBench,
		benchName,
		benchDescription,
	}
}

// This benchmark prepopulates a directory with some files of a given size. Then, it creates and deletes
// a file of some size, while measuring only the performance of the delete.
func deleteBench(
	benchName string, benchDescription string, preNumFiles int, preFileSize int64, fileSize int64,
) fsBenchmark {

	createBench := func(dirpath string) *fsBench {
		bench := &fsBench{}
		mkDir(dirpath)
		fh := openDir(dirpath)

		bench.dir = fh
		bench.dirName = dirpath
		bench.reg = newHistogramRegistry()
		bench.numOps = 0
		bench.name = benchName
		bench.description = benchDescription

		// prepopulate the directory
		prePref := "pre_temp_"
		for i := 0; i < preNumFiles; i++ {
			fh := createFile(path.Join(dirpath, fmt.Sprintf("%s%d", prePref, i)))
			if preFileSize > 0 {
				writeToFile(fh, preFileSize)
				syncFile(fh)
			}
			closeFile(fh)
		}
		syncFile(bench.dir)

		var done atomic.Bool
		bench.run = func(hist *namedHistogram) bool {
			if done.Load() {
				return false
			}

			filename := "newfile"
			fh := createFile(path.Join(dirpath, filename))
			writeToFile(fh, fileSize)
			syncFile(fh)

			start := time.Now()
			deleteFile(path.Join(dirpath, filename))
			hist.Record(time.Since(start))

			return true
		}

		bench.stop = func() {
			done.Store(true)
		}

		bench.clean = func() {
			removeAllFiles(dirpath)
			closeFile(bench.dir)
		}

		return bench
	}

	return fsBenchmark{
		createBench,
		benchName,
		benchDescription,
	}
}

// This benchmark creates some files in a directory, and then measures the performance
// of the vfs.Remove function.
// fileSize is in bytes.
func deleteUniformBench(
	benchName string, benchDescription string, numFiles int, fileSize int64,
) fsBenchmark {
	createBench := func(dirpath string) *fsBench {
		bench := &fsBench{}
		mkDir(dirpath)
		fh := openDir(dirpath)

		bench.dir = fh
		bench.dirName = dirpath
		bench.reg = newHistogramRegistry()
		bench.numOps = 0
		bench.name = benchName
		bench.description = benchDescription

		// setup the operation to benchmark, and the cleaup functions.
		pref := "temp_"
		for i := 0; i < numFiles; i++ {
			fh := createFile(path.Join(dirpath, fmt.Sprintf("%s%d", pref, i)))
			if fileSize > 0 {
				writeToFile(fh, fileSize)
				syncFile(fh)
			}
			closeFile(fh)
		}
		syncFile(bench.dir)

		var done atomic.Bool
		bench.run = func(hist *namedHistogram) bool {
			if done.Load() {
				return false
			}

			if numFiles == 0 {
				return false
			}

			start := time.Now()
			deleteFile(path.Join(dirpath, fmt.Sprintf("%s%d", pref, numFiles-1)))
			hist.Record(time.Since(start))

			numFiles--
			return true
		}

		bench.stop = func() {
			done.Store(true)
		}

		bench.clean = func() {
			removeAll(dirpath)
			closeFile(bench.dir)
		}

		return bench
	}

	return fsBenchmark{
		createBench,
		benchName,
		benchDescription,
	}
}

// Tests the performance of syncing data to disk.
// Only measures the sync performance.
// The writes will be synced after every writeSize bytes have been written.
func writeSyncBench(
	benchName string, benchDescription string, maxFileSize int64, writeSize int64,
) fsBenchmark {

	if writeSize > maxFileSize {
		log.Fatalln("File write threshold is greater than max file size.")
	}

	createBench := func(dirpath string) *fsBench {
		bench := &fsBench{}
		mkDir(dirpath)
		fh := openDir(dirpath)

		bench.dir = fh
		bench.dirName = dirpath
		bench.reg = newHistogramRegistry()
		bench.numOps = 0
		bench.name = benchName
		bench.description = benchDescription

		pref := "temp_"
		var benchData struct {
			done         atomic.Bool
			fh           vfs.File
			fileNum      int
			bytesWritten int64
		}
		benchData.fh = createFile(path.Join(dirpath, fmt.Sprintf("%s%d", pref, benchData.fileNum)))

		bench.run = func(hist *namedHistogram) bool {
			if benchData.done.Load() {
				return false
			}

			if benchData.bytesWritten+writeSize > maxFileSize {
				closeFile(benchData.fh)
				benchData.fileNum++
				benchData.bytesWritten = 0
				benchData.fh = createFile(path.Join(dirpath, fmt.Sprintf("%s%d", pref, benchData.fileNum)))
			}

			benchData.bytesWritten += writeSize
			writeToFile(benchData.fh, writeSize)

			start := time.Now()
			syncFile(benchData.fh)
			hist.Record(time.Since(start))

			return true
		}

		bench.stop = func() {
			benchData.done.Store(true)
		}

		bench.clean = func() {
			closeFile(benchData.fh)
			removeAllFiles(dirpath)
			closeFile(bench.dir)
		}

		return bench
	}

	return fsBenchmark{
		createBench,
		benchName,
		benchDescription,
	}
}

// Tests the peformance of calling the vfs.GetDiskUsage call on a directory,
// as the number of files/total size of files in the directory grows.
func diskUsageBench(
	benchName string, benchDescription string, maxFileSize int64, writeSize int64,
) fsBenchmark {

	if writeSize > maxFileSize {
		log.Fatalln("File write threshold is greater than max file size.")
	}

	createBench := func(dirpath string) *fsBench {
		bench := &fsBench{}
		mkDir(dirpath)
		fh := openDir(dirpath)

		bench.dir = fh
		bench.dirName = dirpath
		bench.reg = newHistogramRegistry()
		bench.numOps = 0
		bench.name = benchName
		bench.description = benchDescription

		pref := "temp_"
		var benchData struct {
			done         atomic.Bool
			fh           vfs.File
			fileNum      int
			bytesWritten int64
		}
		benchData.fh = createFile(path.Join(dirpath, fmt.Sprintf("%s%d", pref, benchData.fileNum)))

		bench.run = func(hist *namedHistogram) bool {
			if benchData.done.Load() {
				return false
			}

			if benchData.bytesWritten+writeSize > maxFileSize {
				closeFile(benchData.fh)
				benchData.fileNum++
				benchData.bytesWritten = 0
				benchData.fh = createFile(path.Join(dirpath, fmt.Sprintf("%s%d", pref, benchData.fileNum)))
			}

			benchData.bytesWritten += writeSize
			writeToFile(benchData.fh, writeSize)
			syncFile(benchData.fh)

			start := time.Now()
			getDiskUsage(dirpath)
			hist.Record(time.Since(start))

			return true
		}

		bench.stop = func() {
			benchData.done.Store(true)
		}

		bench.clean = func() {
			closeFile(benchData.fh)
			removeAllFiles(dirpath)
			closeFile(bench.dir)
		}

		return bench
	}

	return fsBenchmark{
		createBench,
		benchName,
		benchDescription,
	}
}

// A benchmark is a function which takes a directory
// as input and returns the fsBench struct which has
// all the information required to run the benchmark.
type fsBenchmark struct {
	createBench func(string) *fsBench
	name        string
	description string
}

// The various benchmarks which can be run.
var benchmarks = map[string]fsBenchmark{
	"create_empty": createBench("create_empty", "create empty file, sync par dir"),
	"delete_10k_2MiB": deleteUniformBench(
		"delete_10k_2MiB", "create 10k 2MiB size files, measure deletion times", 10_000, 2<<20,
	),
	"delete_100k_2MiB": deleteUniformBench(
		"delete_100k_2MiB", "create 100k 2MiB size files, measure deletion times", 100_000, 2<<20,
	),
	"delete_200k_2MiB": deleteUniformBench(
		"delete_200k_2MiB", "create 200k 2MiB size files, measure deletion times", 200_000, 2<<20,
	),
	"write_sync_1MiB": writeSyncBench(
		"write_sync_1MiB", "Write 1MiB to a file, then sync, while timing the sync.", 2<<30, 1<<20,
	),
	"write_sync_16MiB": writeSyncBench(
		"write_sync_16MiB", "Write 16MiB to a file, then sync, while timing the sync.", 2<<30, 16<<20,
	),
	"write_sync_128MiB": writeSyncBench(
		"write_sync_128MiB", "Write 128MiB to a file, then sync, while timing the sync.", 2<<30, 128<<20,
	),
	"disk_usage_128MB": diskUsageBench(
		"disk_usage_128MB",
		"Write 128MiB to a file, measure GetDiskUsage call. Create a new file, when file size is 1GB.",
		1<<30, 128<<20,
	),
	"disk_usage_many_files": diskUsageBench(
		"disk_usage_many_files",
		"Create new file, Write 128KiB to a file, measure GetDiskUsage call.",
		128<<10, 128<<10,
	),
	"delete_large_dir_256MiB": deleteBench(
		"delete_large_dir_256MiB", "Prepopulate directory with 100k 1MiB files, measure delete peformance of 256MiB files",
		1e5, 1<<20, 256<<20,
	),
	"delete_large_dir_2MiB": deleteBench(
		"delete_large_dir_2MiB", "Prepopulate directory with 100k 1MiB files, measure delete peformance of 2MiB files",
		1e5, 1<<20, 2<<20,
	),
	"delete_small_dir_2GiB": deleteBench(
		"delete_small_dir_2GiB", "Prepopulate directory with 1k 1MiB files, measure delete peformance of 2GiB files",
		1e3, 1<<20, 2<<30,
	),
	"delete_small_dir_256MiB": deleteBench(
		"delete_small_dir_256MiB", "Prepopulate directory with 1k 1MiB files, measure delete peformance of 256MiB files",
		1e3, 1<<20, 256<<20,
	),
	"delete_small_dir_2MiB": deleteBench(
		"delete_small_dir_2MiB", "Prepopulate directory with 1k 1MiB files, measure delete peformance of 2MiB files",
		1e3, 1<<20, 2<<20,
	),
}

func runFsBench(_ *cobra.Command, args []string) error {
	benchmark, ok := benchmarks[fsConfig.benchname]
	if !ok {
		return errors.Errorf("trying to run an unknown benchmark: %s", fsConfig.benchname)
	}

	// Run the benchmark a comple of times.
	fmt.Printf("The benchmark will be run %d time(s).\n", fsConfig.numTimes)
	for i := 0; i < fsConfig.numTimes; i++ {
		fmt.Println("Starting benchmark:", i)
		benchStruct := benchmark.createBench(args[0])
		runTestWithoutDB(testWithoutDB{
			init: benchStruct.init,
			tick: benchStruct.tick,
			done: benchStruct.done,
		})
	}
	return nil
}

func (bench *fsBench) init(wg *sync.WaitGroup) {
	fmt.Println("Running benchmark:", bench.name)
	fmt.Println("Description:", bench.description)

	wg.Add(1)
	go bench.execute(wg)
}

func (bench *fsBench) execute(wg *sync.WaitGroup) {
	defer wg.Done()

	latencyHist := bench.reg.Register(bench.name)

	for {
		// run the op which we're benchmarking.
		bench.numOps++

		// The running function will determine exactly what to latency
		// it wants to measure.
		continueBench := bench.run(latencyHist)
		if !continueBench || (fsConfig.maxOps > 0 && bench.numOps >= fsConfig.maxOps) {
			break
		}
	}
}

func (bench *fsBench) tick(elapsed time.Duration, i int) {
	if i%20 == 0 {
		fmt.Println("____optype__elapsed__ops/sec(inst)___ops/sec(cum)__p50(ms)__p95(ms)__p99(ms)__pMax(ms)")
	}
	bench.reg.Tick(func(tick histogramTick) {
		h := tick.Hist

		fmt.Printf("%10s %8s %14.1f %14.1f %5.6f %5.6f %5.6f %5.6f\n",
			tick.Name[:10],
			time.Duration(elapsed.Seconds()+0.5)*time.Second,
			float64(h.TotalCount())/tick.Elapsed.Seconds(),
			float64(tick.Cumulative.TotalCount())/elapsed.Seconds(),
			time.Duration(h.ValueAtQuantile(50)).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(95)).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(99)).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(100)).Seconds()*1000,
		)
	})
}

func (bench *fsBench) done(wg *sync.WaitGroup, elapsed time.Duration) {
	// Do the cleanup.
	bench.stop()
	wg.Wait()
	defer bench.clean()

	fmt.Println("\n____optype__elapsed_____ops(total)___ops/sec(cum)__avg(ms)__p50(ms)__p95(ms)__p99(ms)__pMax(ms)")

	resultTick := histogramTick{}
	bench.reg.Tick(func(tick histogramTick) {
		h := tick.Cumulative
		if resultTick.Cumulative == nil {
			resultTick.Now = tick.Now
			resultTick.Cumulative = h
		} else {
			resultTick.Cumulative.Merge(h)
		}

		fmt.Printf("%10s %7.1fs %14d %14.1f %5.6f %5.6f %5.6f %5.6f %5.6f\n",
			tick.Name[:10], elapsed.Seconds(), h.TotalCount(),
			float64(h.TotalCount())/elapsed.Seconds(),
			time.Duration(h.Mean()).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(50)).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(95)).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(99)).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(100)).Seconds()*1000,
		)
	})
	fmt.Println()

	resultHist := resultTick.Cumulative

	fmt.Printf("Benchmarkfsbench/%s  %d %0.1f ops/sec\n\n",
		bench.name,
		resultHist.TotalCount(),
		float64(resultHist.TotalCount())/elapsed.Seconds(),
	)
}

func verbosef(fmtstr string, args ...interface{}) {
	if verbose {
		fmt.Printf(fmtstr, args...)
	}
}

func removeAll(dir string) {
	verbosef("Removing %q.\n", dir)
	if err := os.RemoveAll(dir); err != nil {
		log.Fatal(err)
	}
}
