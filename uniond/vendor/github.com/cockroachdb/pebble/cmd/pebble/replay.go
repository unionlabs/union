// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"bytes"
	"context"
	"flag"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"syscall"
	"time"
	"unicode"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/bloom"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/cache"
	"github.com/cockroachdb/pebble/replay"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/spf13/cobra"
)

func initReplayCmd() *cobra.Command {
	c := replayConfig{
		pacer:            pacerFlag{Pacer: replay.PaceByFixedReadAmp(10)},
		runDir:           "",
		count:            1,
		streamLogs:       false,
		ignoreCheckpoint: false,
	}
	cmd := &cobra.Command{
		Use:   "replay <workload>",
		Short: "run the provided captured write workload",
		Args:  cobra.ExactArgs(1),
		RunE:  c.runE,
	}
	cmd.Flags().IntVar(
		&c.count, "count", 1, "the number of times to replay the workload")
	cmd.Flags().StringVar(
		&c.name, "name", "", "the name of the workload being replayed")
	cmd.Flags().VarPF(
		&c.pacer, "pacer", "p", "the pacer to use: unpaced, reference-ramp, or fixed-ramp=N")
	cmd.Flags().Uint64Var(
		&c.maxWritesMB, "max-writes", 0, "the maximum volume of writes (MB) to apply, with 0 denoting unlimited")
	cmd.Flags().StringVar(
		&c.optionsString, "options", "", "Pebble options to override, in the OPTIONS ini format but with any whitespace as field delimiters instead of newlines")
	cmd.Flags().StringVar(
		&c.runDir, "run-dir", c.runDir, "the directory to use for the replay data directory; defaults to a random dir in pwd")
	cmd.Flags().Int64Var(
		&c.maxCacheSize, "max-cache-size", c.maxCacheSize, "the max size of the block cache")
	cmd.Flags().BoolVar(
		&c.streamLogs, "stream-logs", c.streamLogs, "stream the Pebble logs to stdout during replay")
	cmd.Flags().BoolVar(
		&c.ignoreCheckpoint, "ignore-checkpoint", c.ignoreCheckpoint, "ignore the workload's initial checkpoint")
	cmd.Flags().StringVar(
		&c.checkpointDir, "checkpoint-dir", c.checkpointDir, "path to the checkpoint to use if not <WORKLOAD_DIR>/checkpoint")
	return cmd
}

type replayConfig struct {
	name             string
	pacer            pacerFlag
	runDir           string
	count            int
	maxWritesMB      uint64
	streamLogs       bool
	checkpointDir    string
	ignoreCheckpoint bool
	optionsString    string
	maxCacheSize     int64

	cleanUpFuncs []func() error
}

func (c *replayConfig) args() (args []string) {
	if c.name != "" {
		args = append(args, "--name", c.name)
	}
	if c.pacer.spec != "" {
		args = append(args, "--pacer", c.pacer.spec)
	}
	if c.runDir != "" {
		args = append(args, "--run-dir", c.runDir)
	}
	if c.count != 0 {
		args = append(args, "--count", fmt.Sprint(c.count))
	}
	if c.maxWritesMB != 0 {
		args = append(args, "--max-writes", fmt.Sprint(c.maxWritesMB))
	}
	if c.maxCacheSize != 0 {
		args = append(args, "--max-cache-size", fmt.Sprint(c.maxCacheSize))
	}
	if c.streamLogs {
		args = append(args, "--stream-logs")
	}
	if c.checkpointDir != "" {
		args = append(args, "--checkpoint-dir", c.checkpointDir)
	}
	if c.ignoreCheckpoint {
		args = append(args, "--ignore-checkpoint")
	}
	if c.optionsString != "" {
		args = append(args, "--options", c.optionsString)
	}
	return args
}

func (c *replayConfig) runE(cmd *cobra.Command, args []string) error {
	if c.ignoreCheckpoint && c.checkpointDir != "" {
		return errors.Newf("cannot provide both --checkpoint-dir and --ignore-checkpoint")
	}
	stdout := cmd.OutOrStdout()

	workloadPath := args[0]
	if err := c.runOnce(stdout, workloadPath); err != nil {
		return err
	}
	c.count--

	// If necessary, run it again. We run again replacing our existing process
	// with the next run so that we're truly starting over. This helps avoid the
	// possibility of state within the Go runtime, the fragmentation of the
	// heap, or global state within Pebble from interfering with the
	// independence of individual runs. Previously we called runOnce multiple
	// times without exec-ing, but we observed less variance between runs from
	// within the same process.
	if c.count > 0 {
		fmt.Printf("%d runs remaining.", c.count)
		executable, err := os.Executable()
		if err != nil {
			return err
		}
		execArgs := append(append([]string{executable, "bench", "replay"}, c.args()...), workloadPath)
		syscall.Exec(executable, execArgs, os.Environ())
	}
	return nil
}

func (c *replayConfig) runOnce(stdout io.Writer, workloadPath string) error {
	defer c.cleanUp()
	if c.name == "" {
		c.name = vfs.Default.PathBase(workloadPath)
	}

	r := &replay.Runner{
		RunDir:       c.runDir,
		WorkloadFS:   vfs.Default,
		WorkloadPath: workloadPath,
		Pacer:        c.pacer,
		Opts:         &pebble.Options{},
	}
	if c.maxWritesMB > 0 {
		r.MaxWriteBytes = c.maxWritesMB * (1 << 20)
	}
	if err := c.initRunDir(r); err != nil {
		return err
	}
	if err := c.initOptions(r); err != nil {
		return err
	}
	if verbose {
		fmt.Fprintln(stdout, "Options:")
		fmt.Fprintln(stdout, r.Opts.String())
	}

	// Begin the workload. Run does not block.
	ctx := context.Background()
	if err := r.Run(ctx); err != nil {
		return errors.Wrapf(err, "starting workload")
	}

	// Wait blocks until the workload is complete. Once Wait returns, all of the
	// workload's write operations have been replayed AND the database's
	// compactions have quiesced.
	m, err := r.Wait()
	if err != nil {
		return errors.Wrapf(err, "waiting for workload to complete")
	}
	if err := r.Close(); err != nil {
		return errors.Wrapf(err, "cleaning up")
	}
	fmt.Fprintln(stdout, "Workload complete.")
	if err := m.WriteBenchmarkString(c.name, stdout); err != nil {
		return err
	}
	for _, plot := range m.Plots(120 /* width */, 30 /* height */) {
		fmt.Fprintln(stdout, plot.Name)
		fmt.Fprintln(stdout, plot.Plot)
		fmt.Fprintln(stdout)
	}
	fmt.Fprintln(stdout, m.Final.String())
	return nil
}

func (c *replayConfig) initRunDir(r *replay.Runner) error {
	if r.RunDir == "" {
		// Default to replaying in a new directory within the current working
		// directory.
		wd, err := os.Getwd()
		if err != nil {
			return err
		}
		r.RunDir, err = os.MkdirTemp(wd, "replay-")
		if err != nil {
			return err
		}
		c.cleanUpFuncs = append(c.cleanUpFuncs, func() error {
			return os.RemoveAll(r.RunDir)
		})
	}
	if !c.ignoreCheckpoint {
		checkpointDir := c.getCheckpointDir(r)
		fmt.Printf("%s: Attempting to initialize with checkpoint %q.\n", time.Now().Format(time.RFC3339), checkpointDir)
		ok, err := vfs.Clone(
			r.WorkloadFS,
			vfs.Default,
			checkpointDir,
			filepath.Join(r.RunDir),
			vfs.CloneTryLink)
		if err != nil {
			return err
		}
		if !ok {
			return errors.Newf("no checkpoint %q exists; you may re-run with --ignore-checkpoint", checkpointDir)
		}
		fmt.Printf("%s: Run directory initialized with checkpoint %q.\n", time.Now().Format(time.RFC3339), checkpointDir)
	}
	return nil
}

func (c *replayConfig) initOptions(r *replay.Runner) error {
	// If using a workload checkpoint, load the Options from it.
	// TODO(jackson): Allow overriding the OPTIONS.
	if !c.ignoreCheckpoint {
		ls, err := r.WorkloadFS.List(c.getCheckpointDir(r))
		if err != nil {
			return err
		}
		sort.Strings(ls)
		var optionsFilepath string
		for _, l := range ls {
			path := r.WorkloadFS.PathJoin(r.WorkloadPath, "checkpoint", l)
			typ, _, ok := base.ParseFilename(r.WorkloadFS, path)
			if ok && typ == base.FileTypeOptions {
				optionsFilepath = path
			}
		}
		f, err := r.WorkloadFS.Open(optionsFilepath)
		if err != nil {
			return err
		}
		o, err := io.ReadAll(f)
		if err != nil {
			return err
		}
		if err := f.Close(); err != nil {
			return err
		}
		if err := r.Opts.Parse(string(o), c.parseHooks()); err != nil {
			return err
		}
	}
	if err := c.parseCustomOptions(c.optionsString, r.Opts); err != nil {
		return err
	}
	// TODO(jackson): If r.Opts.Comparer == nil, peek at the workload's
	// manifests and pull the comparer out of them.
	//
	// r.Opts.Comparer can only be nil at this point if ignoreCheckpoint is
	// set; otherwise we'll have already extracted the Comparer from the
	// checkpoint's OPTIONS file.

	if c.streamLogs {
		r.Opts.AddEventListener(pebble.MakeLoggingEventListener(pebble.DefaultLogger))
	}
	r.Opts.EnsureDefaults()
	return nil
}

func (c *replayConfig) getCheckpointDir(r *replay.Runner) string {
	if c.checkpointDir != "" {
		return c.checkpointDir
	}
	return r.WorkloadFS.PathJoin(r.WorkloadPath, `checkpoint`)
}

func (c *replayConfig) parseHooks() *pebble.ParseHooks {
	return &pebble.ParseHooks{
		NewCache: func(size int64) *cache.Cache {
			if c.maxCacheSize != 0 && size > c.maxCacheSize {
				size = c.maxCacheSize
			}
			return cache.New(size)
		},
		NewComparer: makeComparer,
		NewFilterPolicy: func(name string) (pebble.FilterPolicy, error) {
			switch name {
			case "none":
				return nil, nil
			case "rocksdb.BuiltinBloomFilter":
				return bloom.FilterPolicy(10), nil
			default:
				return nil, errors.Errorf("invalid filter policy name %q", name)
			}
		},
		NewMerger: makeMerger,
	}
}

// parseCustomOptions parses Pebble Options passed through a CLI flag.
// Ordinarily Pebble Options are specified through an INI file with newlines
// delimiting fields. That doesn't translate well to a CLI interface, so this
// function accepts fields are that delimited by any whitespace. This is the
// same format that CockroachDB accepts Pebble Options through the --store flag,
// and this code is copied from there.
func (c *replayConfig) parseCustomOptions(optsStr string, opts *pebble.Options) error {
	if optsStr == "" {
		return nil
	}
	// Pebble options are supplied in the Pebble OPTIONS ini-like
	// format, but allowing any whitespace to delimit lines. Convert
	// the options to a newline-delimited format. This isn't a trivial
	// character replacement because whitespace may appear within a
	// stanza, eg ["Level 0"].
	value := strings.TrimSpace(optsStr)
	var buf bytes.Buffer
	for len(value) > 0 {
		i := strings.IndexFunc(value, func(r rune) bool {
			return r == '[' || unicode.IsSpace(r)
		})
		switch {
		case i == -1:
			buf.WriteString(value)
			value = value[len(value):]
		case value[i] == '[':
			// If there's whitespace within [ ], we write it verbatim.
			j := i + strings.IndexRune(value[i:], ']')
			buf.WriteString(value[:j+1])
			value = value[j+1:]
		case unicode.IsSpace(rune(value[i])):
			// NB: This doesn't handle multibyte whitespace.
			buf.WriteString(value[:i])
			buf.WriteRune('\n')
			value = strings.TrimSpace(value[i+1:])
		}
	}
	return opts.Parse(buf.String(), c.parseHooks())
}

func (c *replayConfig) cleanUp() error {
	for _, f := range c.cleanUpFuncs {
		if err := f(); err != nil {
			return err
		}
	}
	return nil
}

func makeComparer(name string) (*pebble.Comparer, error) {
	switch name {
	case base.DefaultComparer.Name:
		return base.DefaultComparer, nil
	case "cockroach_comparator":
		return mvccComparer, nil
	default:
		return nil, errors.Newf("unrecognized comparer %q", name)
	}
}

func makeMerger(name string) (*pebble.Merger, error) {
	switch name {
	case base.DefaultMerger.Name:
		return base.DefaultMerger, nil
	case "cockroach_merge_operator":
		// We don't want to reimplement the cockroach merger. Instead we
		// implement this merger to return the newer of the two operands. This
		// doesn't exactly model cockroach's true use but should be good enough.
		// TODO(jackson): Consider lifting replay into a `cockroach debug`
		// command so we can use the true merger and comparer.
		merger := new(pebble.Merger)
		merger.Merge = func(key, value []byte) (pebble.ValueMerger, error) {
			return &overwriteValueMerger{value: append([]byte{}, value...)}, nil
		}
		merger.Name = name
		return merger, nil
	default:
		return nil, errors.Newf("unrecognized comparer %q", name)
	}
}

// pacerFlag provides a command line flag interface for specifying the pacer to
// use. It implements the flag.Value interface.
type pacerFlag struct {
	replay.Pacer
	spec string
}

var _ flag.Value = (*pacerFlag)(nil)

func (f *pacerFlag) String() string { return f.spec }
func (f *pacerFlag) Type() string   { return "pacer" }

// Set implements the Flag.Value interface.
func (f *pacerFlag) Set(spec string) error {
	f.spec = spec
	switch {
	case spec == "unpaced":
		f.Pacer = replay.Unpaced{}
	case spec == "reference-ramp":
		f.Pacer = replay.PaceByReferenceReadAmp{}
	case strings.HasPrefix(spec, "fixed-ramp="):
		rAmp, err := strconv.Atoi(strings.TrimPrefix(spec, "fixed-ramp="))
		if err != nil {
			return errors.Newf("unable to parse fixed r-amp: %s", err)
		}
		f.Pacer = replay.PaceByFixedReadAmp(rAmp)
	default:
		return errors.Newf("unrecognized pacer spec: %q", errors.Safe(spec))
	}
	return nil
}

type overwriteValueMerger struct {
	value []byte
}

func (o *overwriteValueMerger) MergeNewer(value []byte) error {
	o.value = append(o.value[:0], value...)
	return nil
}

func (o *overwriteValueMerger) MergeOlder(value []byte) error {
	return nil
}

func (o *overwriteValueMerger) Finish(includesBase bool) ([]byte, io.Closer, error) {
	return o.value, nil, nil
}
