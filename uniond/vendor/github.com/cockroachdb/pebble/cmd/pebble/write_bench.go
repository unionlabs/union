// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/cockroachdb/pebble/internal/ackseq"
	"github.com/cockroachdb/pebble/internal/randvar"
	"github.com/cockroachdb/pebble/internal/rate"
	"github.com/spf13/cobra"
)

// The following constants match the values that Cockroach uses in Admission
// Control at the time of writing.
// See: https://github.com/cockroachdb/cockroach/blob/cb5d5108a7705eac7be82bc7f0f8b6f4dc825b96/pkg/util/admission/granter.go#L1212-L1229
const (
	defaultL0FileLimit     = 1000
	defaultL0SubLevelLimit = 20
)

var writeBenchConfig struct {
	batch              *randvar.Flag
	keys               string
	values             *randvar.BytesFlag
	concurrency        int
	rateStart          int
	incBase            int
	testPeriod         time.Duration
	cooloffPeriod      time.Duration
	targetL0Files      int
	targetL0SubLevels  int
	maxRateDipFraction float64
	debug              bool
}

var writeBenchCmd = &cobra.Command{
	Use:   "write <dir>",
	Short: "Run YCSB F to find an a sustainable write throughput",
	Long: `
Run YCSB F (100% writes) at varying levels of sustained write load (ops/sec) to
determine an optimal value of write throughput.

The benchmark works by maintaining a fixed amount of write load on the DB for a
fixed amount of time. If the database can handle the sustained load - determined
by a heuristic that takes into account the number of files in L0 sub-levels, the
number of L0 sub-levels, and whether the DB has encountered a write stall (i.e.
measured load on the DB drops to zero) - the load is increased on the DB.

Load increases exponentially from an initial load. If the DB fails the heuristic
at the given write load, the load on the DB is paused for a period of time (the
cool-off period) before returning to the last value at which the DB could handle
the load. The exponent is then reset and the process repeats from this new
initial value. This allows the benchmark to converge on and oscillate around the
optimal write load.

The values of load at which the DB passes and fails the heuristic are maintained
over the duration of the benchmark. On completion of the benchmark, an "optimal"
value is computed. The optimal value is computed as the value that minimizes the
mis-classification of the recorded "passes" and "fails"". This can be visualized
as a point on the x-axis that separates the passes and fails into the left and
right half-planes, minimizing the number of fails that fall to the left of this
point (i.e. mis-classified fails) and the number of passes that fall to the
right (i.e. mis-classified passes).

The resultant "optimal sustained write load" value provides an estimate of the
write load that the DB can sustain without failing the target heuristic.

A typical invocation of the benchmark is as follows:

  pebble bench write [PATH] --wipe -c 1024 -d 8h --rate-start 30000 --debug
`,
	Args: cobra.ExactArgs(1),
	RunE: runWriteBenchmark,
}

func init() {
	initWriteBench(writeBenchCmd)
}

func initWriteBench(cmd *cobra.Command) {
	// Default values for custom flags.
	writeBenchConfig.batch = randvar.NewFlag("1")
	writeBenchConfig.values = randvar.NewBytesFlag("1000")

	cmd.Flags().Var(
		writeBenchConfig.batch, "batch",
		"batch size distribution [{zipf,uniform}:]min[-max]")
	cmd.Flags().StringVar(
		&writeBenchConfig.keys, "keys", "zipf", "latest, uniform, or zipf")
	cmd.Flags().Var(
		writeBenchConfig.values, "values",
		"value size distribution [{zipf,uniform}:]min[-max][/<target-compression>]")
	cmd.Flags().IntVarP(
		&writeBenchConfig.concurrency, "concurrency", "c",
		1, "number of concurrent workers")
	cmd.Flags().IntVar(
		&writeBenchConfig.rateStart, "rate-start",
		1000, "starting write load (ops/sec)")
	cmd.Flags().IntVar(
		&writeBenchConfig.incBase, "rate-inc-base",
		100, "increment / decrement base")
	cmd.Flags().DurationVar(
		&writeBenchConfig.testPeriod, "test-period",
		60*time.Second, "time to run at a given write load")
	cmd.Flags().DurationVar(
		&writeBenchConfig.cooloffPeriod, "cooloff-period",
		30*time.Second, "time to pause write load after a failure")
	cmd.Flags().IntVar(
		&writeBenchConfig.targetL0Files, "l0-files",
		defaultL0FileLimit, "target L0 file count")
	cmd.Flags().IntVar(
		&writeBenchConfig.targetL0SubLevels, "l0-sublevels",
		defaultL0SubLevelLimit, "target L0 sublevel count")
	cmd.Flags().BoolVarP(
		&wipe, "wipe", "w", false, "wipe the database before starting")
	cmd.Flags().Float64Var(
		&writeBenchConfig.maxRateDipFraction, "max-rate-dip-fraction", 0.1,
		"fraction at which to mark a test-run as failed if the actual rate dips below (relative to the desired rate)")
	cmd.Flags().BoolVar(
		&writeBenchConfig.debug, "debug", false, "print benchmark debug information")
}

// writeBenchResult contains the results of a test run at a given rate. The
// independent variable is the rate (in ops/sec) and the dependent variable is
// whether the test passed or failed. Additional metadata associated with the
// test run is also captured.
type writeBenchResult struct {
	name     string
	rate     int           // The rate at which the test is currently running.
	passed   bool          // Was the test successful at this rate.
	elapsed  time.Duration // The total elapsed time of the test.
	bytes    uint64        // The size of the LSM.
	levels   int           // The number of levels occupied in the LSM.
	writeAmp float64       // The write amplification.
}

// String implements fmt.Stringer, printing a raw benchmark line. These lines
// are used when performing analysis on a given benchmark run.
func (r writeBenchResult) String() string {
	return fmt.Sprintf("BenchmarkRaw%s %d ops/sec %v pass %s elapsed %d bytes %d levels %.2f writeAmp",
		r.name,
		r.rate,
		r.passed,
		r.elapsed,
		r.bytes,
		r.levels,
		r.writeAmp,
	)
}

func runWriteBenchmark(_ *cobra.Command, args []string) error {
	const workload = "F" // 100% inserts.
	var (
		writers      []*pauseWriter
		writersWg    *sync.WaitGroup // Tracks completion of all pauseWriters.
		cooloff      bool            // Is cool-off enabled.
		streak       int             // The number of successive passes.
		clockStart   time.Time       // Start time for current load.
		cooloffStart time.Time       // When cool-off was enabled.
		stack        []int           // Stack of passing load values.
		pass, fail   []int           // Values of load that pass and fail, respectively.
		rateAcc      float64         // Accumulator of measured rates for a single test run.
	)

	desiredRate := writeBenchConfig.rateStart
	incBase := writeBenchConfig.incBase
	weights, err := ycsbParseWorkload(workload)

	if err != nil {
		return err
	}

	keyDist, err := ycsbParseKeyDist(writeBenchConfig.keys)
	if err != nil {
		return err
	}
	batchDist := writeBenchConfig.batch
	valueDist := writeBenchConfig.values

	// Construct a new YCSB F benchmark with the configured values.
	y := newYcsb(weights, keyDist, batchDist, nil /* scans */, valueDist)
	y.keyNum = ackseq.New(0)

	setLimit := func(l int) {
		perWriterRate := float64(l) / float64(len(writers))
		for _, w := range writers {
			w.setRate(perWriterRate)
		}
	}

	// Function closure to run on test-run failure.
	onTestFail := func(r writeBenchResult, cancel func()) {
		fail = append(fail, desiredRate)

		// Emit a benchmark raw datapoint.
		fmt.Println(r)

		// We failed at the current load, we have two options:

		// a) No room to backtrack. We're done.
		if len(stack) == 0 {
			debugPrint("no room to backtrack; exiting ...\n")
			cancel()
			writersWg.Wait()
			return
		}

		// b) We still have room to backtrack. Reduce the load to the
		// last known passing value.
		desiredRate, stack = stack[len(stack)-1], stack[:len(stack)-1]
		setLimit(desiredRate)

		// Enter the cool-off period.
		cooloff = true
		var wg sync.WaitGroup
		for _, w := range writers {
			// With a large number of writers, pausing synchronously can
			// take a material amount of time. Instead, pause the
			// writers in parallel in the background, and wait for all
			// to complete before continuing.
			wg.Add(1)
			go func(writer *pauseWriter) {
				writer.pause()
				wg.Done()
			}(w)
		}
		wg.Wait()

		// Reset the counters and clocks.
		streak = 0
		rateAcc = 0
		cooloffStart = time.Now()
		clockStart = time.Now()
		debugPrint("Fail. Pausing writers for cool-off period.\n")
		debugPrint(fmt.Sprintf("new rate=%d\npasses=%v\nfails=%v\nstack=%v\n",
			desiredRate, pass, fail, stack))
	}

	// Function closure to run on test-run success.
	onTestSuccess := func(r writeBenchResult) {
		streak++
		pass = append(pass, desiredRate)
		stack = append(stack, desiredRate)

		// Emit a benchmark raw datapoint.
		r.passed = true
		fmt.Println(r)

		// Increase the rate.
		desiredRate = desiredRate + incBase*(1<<(streak-1))
		setLimit(desiredRate)

		// Restart the test.
		rateAcc = 0
		clockStart = time.Now()

		debugPrint(fmt.Sprintf("Pass.\nnew rate=%d\npasses=%v\nfails=%v\nstreak=%d\nstack=%v\n",
			desiredRate, pass, fail, streak, stack))
	}

	name := fmt.Sprintf("write/values=%s", writeBenchConfig.values)
	ctx, cancel := context.WithCancel(context.Background())
	runTest(args[0], test{
		init: func(db DB, wg *sync.WaitGroup) {
			y.db = db
			writersWg = wg

			// Spawn the writers.
			for i := 0; i < writeBenchConfig.concurrency; i++ {
				writer := newPauseWriter(y, float64(desiredRate))
				writers = append(writers, writer)
				writersWg.Add(1)
				go writer.run(ctx, wg)
			}
			setLimit(desiredRate)

			// Start the clock on the current load.
			clockStart = time.Now()
		},
		tick: func(elapsed time.Duration, i int) {
			m := y.db.Metrics()
			if i%20 == 0 {
				if writeBenchConfig.debug && i > 0 {
					fmt.Printf("%s\n", m)
				}
				fmt.Println("___elapsed___clock___rate(desired)___rate(actual)___L0files___L0levels___levels______lsmBytes___writeAmp")
			}

			// Print the current stats.
			l0Files := m.Levels[0].NumFiles
			l0Sublevels := m.Levels[0].Sublevels
			nLevels := 0
			for _, l := range m.Levels {
				if l.BytesIn > 0 {
					nLevels++
				}
			}
			lsmBytes := m.DiskSpaceUsage()
			total := m.Total()
			writeAmp := (&total).WriteAmp()

			var currRate float64
			var stalled bool
			y.reg.Tick(func(tick histogramTick) {
				h := tick.Hist
				currRate = float64(h.TotalCount()) / tick.Elapsed.Seconds()
				stalled = !cooloff && currRate == 0
			})
			rateAcc += currRate

			// The heuristic by which the DB can sustain a given write load is
			// determined by whether the DB, for the configured window of time:
			// 1) did not encounter a write stall (i.e. write load fell to
			//    zero),
			// 2) number of files in L0 was at or below the target, and
			// 3) number of L0 sub-levels is at or below the target.
			failed := stalled ||
				int(l0Files) > writeBenchConfig.targetL0Files ||
				int(l0Sublevels) > writeBenchConfig.targetL0SubLevels

			// Print the result for this tick.
			fmt.Printf("%10s %7s %15d %14.1f %9d %10d %8d %13d %10.2f\n",
				time.Duration(elapsed.Seconds()+0.5)*time.Second,
				time.Duration(time.Since(clockStart).Seconds()+0.5)*time.Second,
				desiredRate,
				currRate,
				l0Files,
				l0Sublevels,
				nLevels,
				lsmBytes,
				writeAmp,
			)

			// If we're in cool-off mode, allow it to complete before resuming
			// writing.
			if cooloff {
				if time.Since(cooloffStart) < writeBenchConfig.cooloffPeriod {
					return
				}
				debugPrint("ending cool-off")

				// Else, resume writing.
				cooloff = false
				for _, w := range writers {
					w.unpause()
				}
				clockStart = time.Now()

				return
			}

			r := writeBenchResult{
				name:     name,
				rate:     desiredRate,
				elapsed:  time.Duration(elapsed.Seconds()+0.5) * time.Second,
				bytes:    lsmBytes,
				levels:   nLevels,
				writeAmp: writeAmp,
			}

			if failed {
				onTestFail(r, cancel)
				return
			}

			// Else, the DB could handle the current load. We only increase
			// after a fixed amount of time at this load as elapsed.
			testElapsed := time.Since(clockStart)
			if testElapsed < writeBenchConfig.testPeriod {
				// This test-run still has time on the clock.
				return
			}

			// This test-run has completed.

			// If the average rate over the test is less than the desired rate,
			// we mark this test-run as a failure. This handles cases where we
			// encounter a bottleneck that limits write throughput but
			// incorrectly mark the test as passed.
			diff := 1 - rateAcc/(float64(desiredRate)*testElapsed.Seconds())
			if diff > writeBenchConfig.maxRateDipFraction {
				if writeBenchConfig.debug {
					debugPrint(fmt.Sprintf(
						"difference in rates (%.2f) exceeded threshold (%.2f); marking test as failed\n",
						diff, writeBenchConfig.maxRateDipFraction,
					))
				}
				onTestFail(r, cancel)
				return
			}

			// Mark this test-run as passed.
			onTestSuccess(r)
		},
		done: func(elapsed time.Duration) {
			// Print final analysis.
			var total int64
			y.reg.Tick(func(tick histogramTick) {
				total = tick.Cumulative.TotalCount()
			})
			fmt.Println("___elapsed___ops(total)")
			fmt.Printf("%10s %12d\n", elapsed.Truncate(time.Second), total)
		},
	})

	return nil
}

// debugPrint prints a debug line to stdout if debug logging is enabled via the
// --debug flag.
func debugPrint(s string) {
	if !writeBenchConfig.debug {
		return
	}
	fmt.Print("DEBUG: " + s)
}

// pauseWriter issues load against a pebble instance, and can be paused on
// demand to allow the DB to recover.
type pauseWriter struct {
	y        *ycsb
	limiter  *rate.Limiter
	pauseC   chan struct{}
	unpauseC chan struct{}
}

// newPauseWriter returns a new pauseWriter.
func newPauseWriter(y *ycsb, initialRate float64) *pauseWriter {
	// Set the burst rate for the limiter to the lowest sensible value to
	// prevent excessive bursting. Note that a burst of zero effectively
	// disables the rate limiter, as a wait time of +Inf is returned from all
	// calls, and `wait(l *rate.Limiter)` will not sleep in this case.
	const burst = 1
	return &pauseWriter{
		y:        y,
		limiter:  rate.NewLimiter(float64(initialRate), burst),
		pauseC:   make(chan struct{}),
		unpauseC: make(chan struct{}),
	}
}

// run starts the pauseWriter, issuing load against the DB.
func (w *pauseWriter) run(ctx context.Context, wg *sync.WaitGroup) {
	defer wg.Done()

	buf := &ycsbBuf{rng: randvar.NewRand()}
	hist := w.y.reg.Register("insert")
	for {
		select {
		case <-ctx.Done():
			return
		case <-w.pauseC:
			// Hold the goroutine here until we unpause.
			<-w.unpauseC
		default:
			wait(w.limiter)
			start := time.Now()
			w.y.insert(w.y.db, buf)
			hist.Record(time.Since(start))
		}
	}
}

// pause signals that the writer should pause after the current operation.
func (w *pauseWriter) pause() {
	w.pauseC <- struct{}{}
}

// unpause unpauses the writer.
func (w *pauseWriter) unpause() {
	w.unpauseC <- struct{}{}
}

// setRate sets the rate limit for this writer.
func (w *pauseWriter) setRate(r float64) {
	w.limiter.SetRate(r)
}
