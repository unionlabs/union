// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	crand "crypto/rand"
	"fmt"
	"io"
	"math"
	"math/rand"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/humanize"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/private"
	"github.com/cockroachdb/pebble/internal/rangedel"
	"github.com/cockroachdb/pebble/internal/rangekey"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/objstorage/remote"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

type iterCmdOpts struct {
	verboseKey bool
	stats      *base.InternalIteratorStats
}

type iterCmdOpt func(*iterCmdOpts)

func iterCmdVerboseKey(opts *iterCmdOpts) { opts.verboseKey = true }

func iterCmdStats(stats *base.InternalIteratorStats) iterCmdOpt {
	return func(opts *iterCmdOpts) {
		opts.stats = stats
	}
}

func runGetCmd(t testing.TB, td *datadriven.TestData, d *DB) string {
	snap := Snapshot{
		db:     d,
		seqNum: InternalKeySeqNumMax,
	}
	td.MaybeScanArgs(t, "seq", &snap.seqNum)

	var buf bytes.Buffer
	for _, data := range strings.Split(td.Input, "\n") {
		v, closer, err := snap.Get([]byte(data))
		if err != nil {
			fmt.Fprintf(&buf, "%s: %s\n", data, err)
		} else {
			fmt.Fprintf(&buf, "%s:%s\n", data, v)
			closer.Close()
		}
	}
	return buf.String()
}

func runIterCmd(d *datadriven.TestData, iter *Iterator, closeIter bool) string {
	if closeIter {
		defer func() {
			if iter != nil {
				iter.Close()
			}
		}()
	}
	var b bytes.Buffer
	for _, line := range strings.Split(d.Input, "\n") {
		parts := strings.Fields(line)
		if len(parts) == 0 {
			continue
		}
		printValidityState := false
		var valid bool
		var validityState IterValidityState
		switch parts[0] {
		case "seek-ge":
			if len(parts) != 2 {
				return "seek-ge <key>\n"
			}
			valid = iter.SeekGE([]byte(parts[1]))
		case "seek-prefix-ge":
			if len(parts) != 2 {
				return "seek-prefix-ge <key>\n"
			}
			valid = iter.SeekPrefixGE([]byte(parts[1]))
		case "seek-lt":
			if len(parts) != 2 {
				return "seek-lt <key>\n"
			}
			valid = iter.SeekLT([]byte(parts[1]))
		case "seek-ge-limit":
			if len(parts) != 3 {
				return "seek-ge-limit <key> <limit>\n"
			}
			validityState = iter.SeekGEWithLimit(
				[]byte(parts[1]), []byte(parts[2]))
			printValidityState = true
		case "seek-lt-limit":
			if len(parts) != 3 {
				return "seek-lt-limit <key> <limit>\n"
			}
			validityState = iter.SeekLTWithLimit(
				[]byte(parts[1]), []byte(parts[2]))
			printValidityState = true
		case "inspect":
			if len(parts) != 2 {
				return "inspect <field>\n"
			}
			field := parts[1]
			switch field {
			case "lastPositioningOp":
				op := "?"
				switch iter.lastPositioningOp {
				case unknownLastPositionOp:
					op = "unknown"
				case seekPrefixGELastPositioningOp:
					op = "seekprefixge"
				case seekGELastPositioningOp:
					op = "seekge"
				case seekLTLastPositioningOp:
					op = "seeklt"
				case invalidatedLastPositionOp:
					op = "invalidate"
				}
				fmt.Fprintf(&b, "%s=%q\n", field, op)
			default:
				return fmt.Sprintf("unrecognized inspect field %q\n", field)
			}
			continue
		case "next-limit":
			if len(parts) != 2 {
				return "next-limit <limit>\n"
			}
			validityState = iter.NextWithLimit([]byte(parts[1]))
			printValidityState = true
		case "internal-next":
			validity, keyKind := iter.internalNext()
			switch validity {
			case internalNextError:
				fmt.Fprintf(&b, "err: %s\n", iter.Error())
			case internalNextExhausted:
				fmt.Fprint(&b, ".\n")
			case internalNextValid:
				fmt.Fprintf(&b, "%s\n", keyKind)
			default:
				panic("unreachable")
			}
			continue
		case "can-deterministically-single-delete":
			ok, err := CanDeterministicallySingleDelete(iter)
			if err != nil {
				fmt.Fprintf(&b, "err: %s\n", err)
			} else {
				fmt.Fprintf(&b, "%t\n", ok)
			}
			continue
		case "prev-limit":
			if len(parts) != 2 {
				return "prev-limit <limit>\n"
			}
			validityState = iter.PrevWithLimit([]byte(parts[1]))
			printValidityState = true
		case "first":
			valid = iter.First()
		case "last":
			valid = iter.Last()
		case "next":
			valid = iter.Next()
		case "next-prefix":
			valid = iter.NextPrefix()
		case "prev":
			valid = iter.Prev()
		case "set-bounds":
			if len(parts) <= 1 || len(parts) > 3 {
				return "set-bounds lower=<lower> upper=<upper>\n"
			}
			var lower []byte
			var upper []byte
			for _, part := range parts[1:] {
				arg := strings.Split(part, "=")
				switch arg[0] {
				case "lower":
					lower = []byte(arg[1])
				case "upper":
					upper = []byte(arg[1])
				default:
					return fmt.Sprintf("set-bounds: unknown arg: %s", arg)
				}
			}
			iter.SetBounds(lower, upper)
			valid = iter.Valid()
		case "set-options":
			opts := iter.opts
			if _, err := parseIterOptions(&opts, &iter.opts, parts[1:]); err != nil {
				return fmt.Sprintf("set-options: %s", err.Error())
			}
			iter.SetOptions(&opts)
			valid = iter.Valid()
		case "stats":
			stats := iter.Stats()
			// The timing is non-deterministic, so set to 0.
			stats.InternalStats.BlockReadDuration = 0
			fmt.Fprintf(&b, "stats: %s\n", stats.String())
			continue
		case "clone":
			var opts CloneOptions
			if len(parts) > 1 {
				var iterOpts IterOptions
				if foundAny, err := parseIterOptions(&iterOpts, &iter.opts, parts[1:]); err != nil {
					return fmt.Sprintf("clone: %s", err.Error())
				} else if foundAny {
					opts.IterOptions = &iterOpts
				}
				for _, part := range parts[1:] {
					if arg := strings.Split(part, "="); len(arg) == 2 && arg[0] == "refresh-batch" {
						var err error
						opts.RefreshBatchView, err = strconv.ParseBool(arg[1])
						if err != nil {
							return fmt.Sprintf("clone: refresh-batch: %s", err.Error())
						}
					}
				}
			}
			clonedIter, err := iter.Clone(opts)
			if err != nil {
				fmt.Fprintf(&b, "error in clone, skipping rest of input: err=%v\n", err)
				return b.String()
			}
			if err = iter.Close(); err != nil {
				fmt.Fprintf(&b, "err=%v\n", err)
			}
			iter = clonedIter
		case "is-using-combined":
			if iter.opts.KeyTypes != IterKeyTypePointsAndRanges {
				fmt.Fprintln(&b, "not configured for combined iteration")
			} else if iter.lazyCombinedIter.combinedIterState.initialized {
				fmt.Fprintln(&b, "using combined (non-lazy) iterator")
			} else {
				fmt.Fprintln(&b, "using lazy iterator")
			}
			continue
		default:
			return fmt.Sprintf("unknown op: %s", parts[0])
		}

		valid = valid || validityState == IterValid
		if valid != iter.Valid() {
			fmt.Fprintf(&b, "mismatched valid states: %t vs %t\n", valid, iter.Valid())
		}
		hasPoint, hasRange := iter.HasPointAndRange()
		hasEither := hasPoint || hasRange
		if hasEither != valid {
			fmt.Fprintf(&b, "mismatched valid/HasPointAndRange states: valid=%t HasPointAndRange=(%t,%t)\n", valid, hasPoint, hasRange)
		}

		if valid {
			validityState = IterValid
		}
		printIterState(&b, iter, validityState, printValidityState)
	}
	return b.String()
}

func parseIterOptions(
	opts *IterOptions, ref *IterOptions, parts []string,
) (foundAny bool, err error) {
	const usageString = "[lower=<lower>] [upper=<upper>] [key-types=point|range|both] [mask-suffix=<suffix>] [mask-filter=<bool>] [only-durable=<bool>] [table-filter=reuse|none] [point-filters=reuse|none]\n"
	for _, part := range parts {
		arg := strings.SplitN(part, "=", 2)
		if len(arg) != 2 {
			return false, errors.Newf(usageString)
		}
		switch arg[0] {
		case "point-filters":
			switch arg[1] {
			case "reuse":
				opts.PointKeyFilters = ref.PointKeyFilters
			case "none":
				opts.PointKeyFilters = nil
			default:
				return false, errors.Newf("unknown arg point-filter=%q:\n%s", arg[1], usageString)
			}
		case "lower":
			opts.LowerBound = []byte(arg[1])
		case "upper":
			opts.UpperBound = []byte(arg[1])
		case "key-types":
			switch arg[1] {
			case "point":
				opts.KeyTypes = IterKeyTypePointsOnly
			case "range":
				opts.KeyTypes = IterKeyTypeRangesOnly
			case "both":
				opts.KeyTypes = IterKeyTypePointsAndRanges
			default:
				return false, errors.Newf("unknown key-type %q:\n%s", arg[1], usageString)
			}
		case "mask-suffix":
			opts.RangeKeyMasking.Suffix = []byte(arg[1])
		case "mask-filter":
			opts.RangeKeyMasking.Filter = func() BlockPropertyFilterMask {
				return sstable.NewTestKeysMaskingFilter()
			}
		case "table-filter":
			switch arg[1] {
			case "reuse":
				opts.TableFilter = ref.TableFilter
			case "none":
				opts.TableFilter = nil
			default:
				return false, errors.Newf("unknown arg table-filter=%q:\n%s", arg[1], usageString)
			}
		case "only-durable":
			var err error
			opts.OnlyReadGuaranteedDurable, err = strconv.ParseBool(arg[1])
			if err != nil {
				return false, errors.Newf("cannot parse only-durable=%q: %s", arg[1], err)
			}
		default:
			continue
		}
		foundAny = true
	}
	return foundAny, nil
}

func printIterState(
	b io.Writer, iter *Iterator, validity IterValidityState, printValidityState bool,
) {
	var validityStateStr string
	if printValidityState {
		switch validity {
		case IterExhausted:
			validityStateStr = " exhausted"
		case IterValid:
			validityStateStr = " valid"
		case IterAtLimit:
			validityStateStr = " at-limit"
		}
	}
	if err := iter.Error(); err != nil {
		fmt.Fprintf(b, "err=%v\n", err)
	} else if validity == IterValid {
		switch {
		case iter.opts.pointKeys():
			hasPoint, hasRange := iter.HasPointAndRange()
			fmt.Fprintf(b, "%s:%s (", iter.Key(), validityStateStr)
			if hasPoint {
				fmt.Fprintf(b, "%s, ", iter.Value())
			} else {
				fmt.Fprint(b, "., ")
			}
			if hasRange {
				start, end := iter.RangeBounds()
				fmt.Fprintf(b, "[%s-%s)", formatASCIIKey(start), formatASCIIKey(end))
				writeRangeKeys(b, iter)
			} else {
				fmt.Fprint(b, ".")
			}
			if iter.RangeKeyChanged() {
				fmt.Fprint(b, " UPDATED")
			}
			fmt.Fprint(b, ")")
		default:
			if iter.Valid() {
				hasPoint, hasRange := iter.HasPointAndRange()
				if hasPoint || !hasRange {
					panic(fmt.Sprintf("pebble: unexpected HasPointAndRange (%t, %t)", hasPoint, hasRange))
				}
				start, end := iter.RangeBounds()
				fmt.Fprintf(b, "%s [%s-%s)", iter.Key(), formatASCIIKey(start), formatASCIIKey(end))
				writeRangeKeys(b, iter)
			} else {
				fmt.Fprint(b, ".")
			}
			if iter.RangeKeyChanged() {
				fmt.Fprint(b, " UPDATED")
			}
		}
		fmt.Fprintln(b)
	} else {
		fmt.Fprintf(b, ".%s\n", validityStateStr)
	}
}

func formatASCIIKey(b []byte) string {
	if bytes.IndexFunc(b, func(r rune) bool { return r < 'A' || r > 'z' }) != -1 {
		// This key is not just ASCII letters. Quote it.
		return fmt.Sprintf("%q", b)
	}
	return string(b)
}

func writeRangeKeys(b io.Writer, iter *Iterator) {
	rangeKeys := iter.RangeKeys()
	for j := 0; j < len(rangeKeys); j++ {
		if j > 0 {
			fmt.Fprint(b, ",")
		}
		fmt.Fprintf(b, " %s=%s", rangeKeys[j].Suffix, rangeKeys[j].Value)
	}
}

func runInternalIterCmd(
	t *testing.T, d *datadriven.TestData, iter internalIterator, opts ...iterCmdOpt,
) string {
	var o iterCmdOpts
	for _, opt := range opts {
		opt(&o)
	}

	getKV := func(key *InternalKey, val LazyValue) (*InternalKey, []byte) {
		v, _, err := val.Value(nil)
		require.NoError(t, err)
		return key, v
	}
	var b bytes.Buffer
	var prefix []byte
	for _, line := range strings.Split(d.Input, "\n") {
		parts := strings.Fields(line)
		if len(parts) == 0 {
			continue
		}
		var key *InternalKey
		var value []byte
		switch parts[0] {
		case "seek-ge":
			if len(parts) < 2 || len(parts) > 3 {
				return "seek-ge <key> [<try-seek-using-next>]\n"
			}
			prefix = nil
			var flags base.SeekGEFlags
			if len(parts) == 3 {
				if trySeekUsingNext, err := strconv.ParseBool(parts[2]); err != nil {
					return err.Error()
				} else if trySeekUsingNext {
					flags = flags.EnableTrySeekUsingNext()
				}
			}
			key, value = getKV(iter.SeekGE([]byte(strings.TrimSpace(parts[1])), flags))
		case "seek-prefix-ge":
			if len(parts) != 2 && len(parts) != 3 {
				return "seek-prefix-ge <key> [<try-seek-using-next>]\n"
			}
			prefix = []byte(strings.TrimSpace(parts[1]))
			var flags base.SeekGEFlags
			if len(parts) == 3 {
				if trySeekUsingNext, err := strconv.ParseBool(parts[2]); err != nil {
					return err.Error()
				} else if trySeekUsingNext {
					flags = flags.EnableTrySeekUsingNext()
				}
			}
			key, value = getKV(iter.SeekPrefixGE(prefix, prefix /* key */, flags))
		case "seek-lt":
			if len(parts) != 2 {
				return "seek-lt <key>\n"
			}
			prefix = nil
			key, value = getKV(iter.SeekLT([]byte(strings.TrimSpace(parts[1])), base.SeekLTFlagsNone))
		case "first":
			prefix = nil
			key, value = getKV(iter.First())
		case "last":
			prefix = nil
			key, value = getKV(iter.Last())
		case "next":
			key, value = getKV(iter.Next())
		case "prev":
			key, value = getKV(iter.Prev())
		case "set-bounds":
			if len(parts) <= 1 || len(parts) > 3 {
				return "set-bounds lower=<lower> upper=<upper>\n"
			}
			var lower []byte
			var upper []byte
			for _, part := range parts[1:] {
				arg := strings.Split(strings.TrimSpace(part), "=")
				switch arg[0] {
				case "lower":
					lower = []byte(arg[1])
				case "upper":
					upper = []byte(arg[1])
				default:
					return fmt.Sprintf("set-bounds: unknown arg: %s", arg)
				}
			}
			iter.SetBounds(lower, upper)
			continue
		case "stats":
			if o.stats != nil {
				// The timing is non-deterministic, so set to 0.
				o.stats.BlockReadDuration = 0
				fmt.Fprintf(&b, "%+v\n", *o.stats)
			}
			continue
		case "reset-stats":
			if o.stats != nil {
				*o.stats = base.InternalIteratorStats{}
			}
			continue
		default:
			return fmt.Sprintf("unknown op: %s", parts[0])
		}
		if key != nil {
			if o.verboseKey {
				fmt.Fprintf(&b, "%s:%s\n", key, value)
			} else {
				fmt.Fprintf(&b, "%s:%s\n", key.UserKey, value)
			}
		} else if err := iter.Error(); err != nil {
			fmt.Fprintf(&b, "err=%v\n", err)
		} else {
			fmt.Fprintf(&b, ".\n")
		}
	}
	return b.String()
}

func runBatchDefineCmd(d *datadriven.TestData, b *Batch) error {
	for _, line := range strings.Split(d.Input, "\n") {
		parts := strings.Fields(line)
		if len(parts) == 0 {
			continue
		}
		if parts[1] == `<nil>` {
			parts[1] = ""
		}
		var err error
		switch parts[0] {
		case "set":
			if len(parts) != 3 {
				return errors.Errorf("%s expects 2 arguments", parts[0])
			}
			err = b.Set([]byte(parts[1]), []byte(parts[2]), nil)
		case "del":
			if len(parts) != 2 {
				return errors.Errorf("%s expects 1 argument", parts[0])
			}
			err = b.Delete([]byte(parts[1]), nil)
		case "del-sized":
			if len(parts) != 3 {
				return errors.Errorf("%s expects 2 arguments", parts[0])
			}
			var valSize uint64
			valSize, err = strconv.ParseUint(parts[2], 10, 32)
			if err != nil {
				return err
			}
			err = b.DeleteSized([]byte(parts[1]), uint32(valSize), nil)
		case "singledel":
			if len(parts) != 2 {
				return errors.Errorf("%s expects 1 argument", parts[0])
			}
			err = b.SingleDelete([]byte(parts[1]), nil)
		case "del-range":
			if len(parts) != 3 {
				return errors.Errorf("%s expects 2 arguments", parts[0])
			}
			err = b.DeleteRange([]byte(parts[1]), []byte(parts[2]), nil)
		case "merge":
			if len(parts) != 3 {
				return errors.Errorf("%s expects 2 arguments", parts[0])
			}
			err = b.Merge([]byte(parts[1]), []byte(parts[2]), nil)
		case "range-key-set":
			if len(parts) < 4 || len(parts) > 5 {
				return errors.Errorf("%s expects 3 or 4 arguments", parts[0])
			}
			var val []byte
			if len(parts) == 5 {
				val = []byte(parts[4])
			}
			err = b.RangeKeySet(
				[]byte(parts[1]),
				[]byte(parts[2]),
				[]byte(parts[3]),
				val,
				nil)
		case "range-key-unset":
			if len(parts) != 4 {
				return errors.Errorf("%s expects 3 arguments", parts[0])
			}
			err = b.RangeKeyUnset(
				[]byte(parts[1]),
				[]byte(parts[2]),
				[]byte(parts[3]),
				nil)
		case "range-key-del":
			if len(parts) != 3 {
				return errors.Errorf("%s expects 2 arguments", parts[0])
			}
			err = b.RangeKeyDelete(
				[]byte(parts[1]),
				[]byte(parts[2]),
				nil)
		default:
			return errors.Errorf("unknown op: %s", parts[0])
		}
		if err != nil {
			return err
		}
	}
	return nil
}

func runBuildRemoteCmd(td *datadriven.TestData, d *DB, storage remote.Storage) error {
	b := d.NewIndexedBatch()
	if err := runBatchDefineCmd(td, b); err != nil {
		return err
	}

	if len(td.CmdArgs) < 1 {
		return errors.New("build <path>: argument missing")
	}
	path := td.CmdArgs[0].String()

	// Override table format, if provided.
	tableFormat := d.opts.FormatMajorVersion.MaxTableFormat()
	for _, cmdArg := range td.CmdArgs[1:] {
		switch cmdArg.Key {
		case "format":
			switch cmdArg.Vals[0] {
			case "leveldb":
				tableFormat = sstable.TableFormatLevelDB
			case "rocksdbv2":
				tableFormat = sstable.TableFormatRocksDBv2
			case "pebblev1":
				tableFormat = sstable.TableFormatPebblev1
			case "pebblev2":
				tableFormat = sstable.TableFormatPebblev2
			case "pebblev3":
				tableFormat = sstable.TableFormatPebblev3
			case "pebblev4":
				tableFormat = sstable.TableFormatPebblev4
			default:
				return errors.Errorf("unknown format string %s", cmdArg.Vals[0])
			}
		}
	}

	writeOpts := d.opts.MakeWriterOptions(0 /* level */, tableFormat)

	f, err := storage.CreateObject(path)
	if err != nil {
		return err
	}
	w := sstable.NewWriter(objstorageprovider.NewRemoteWritable(f), writeOpts)
	iter := b.newInternalIter(nil)
	for key, val := iter.First(); key != nil; key, val = iter.Next() {
		tmp := *key
		tmp.SetSeqNum(0)
		if err := w.Add(tmp, val.InPlaceValue()); err != nil {
			return err
		}
	}
	if err := iter.Close(); err != nil {
		return err
	}

	if rdi := b.newRangeDelIter(nil, math.MaxUint64); rdi != nil {
		for s := rdi.First(); s != nil; s = rdi.Next() {
			err := rangedel.Encode(s, func(k base.InternalKey, v []byte) error {
				k.SetSeqNum(0)
				return w.Add(k, v)
			})
			if err != nil {
				return err
			}
		}
	}

	if rki := b.newRangeKeyIter(nil, math.MaxUint64); rki != nil {
		for s := rki.First(); s != nil; s = rki.Next() {
			for _, k := range s.Keys {
				var err error
				switch k.Kind() {
				case base.InternalKeyKindRangeKeySet:
					err = w.RangeKeySet(s.Start, s.End, k.Suffix, k.Value)
				case base.InternalKeyKindRangeKeyUnset:
					err = w.RangeKeyUnset(s.Start, s.End, k.Suffix)
				case base.InternalKeyKindRangeKeyDelete:
					err = w.RangeKeyDelete(s.Start, s.End)
				default:
					panic("not a range key")
				}
				if err != nil {
					return err
				}
			}
		}
	}

	return w.Close()
}

func runBuildCmd(td *datadriven.TestData, d *DB, fs vfs.FS) error {
	b := d.NewIndexedBatch()
	if err := runBatchDefineCmd(td, b); err != nil {
		return err
	}

	if len(td.CmdArgs) < 1 {
		return errors.New("build <path>: argument missing")
	}
	path := td.CmdArgs[0].String()

	// Override table format, if provided.
	tableFormat := d.opts.FormatMajorVersion.MaxTableFormat()
	for _, cmdArg := range td.CmdArgs[1:] {
		switch cmdArg.Key {
		case "format":
			switch cmdArg.Vals[0] {
			case "leveldb":
				tableFormat = sstable.TableFormatLevelDB
			case "rocksdbv2":
				tableFormat = sstable.TableFormatRocksDBv2
			case "pebblev1":
				tableFormat = sstable.TableFormatPebblev1
			case "pebblev2":
				tableFormat = sstable.TableFormatPebblev2
			case "pebblev3":
				tableFormat = sstable.TableFormatPebblev3
			case "pebblev4":
				tableFormat = sstable.TableFormatPebblev4
			default:
				return errors.Errorf("unknown format string %s", cmdArg.Vals[0])
			}
		}
	}

	writeOpts := d.opts.MakeWriterOptions(0 /* level */, tableFormat)

	f, err := fs.Create(path)
	if err != nil {
		return err
	}
	w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), writeOpts)
	iter := b.newInternalIter(nil)
	for key, val := iter.First(); key != nil; key, val = iter.Next() {
		tmp := *key
		tmp.SetSeqNum(0)
		if err := w.Add(tmp, val.InPlaceValue()); err != nil {
			return err
		}
	}
	if err := iter.Close(); err != nil {
		return err
	}

	if rdi := b.newRangeDelIter(nil, math.MaxUint64); rdi != nil {
		for s := rdi.First(); s != nil; s = rdi.Next() {
			err := rangedel.Encode(s, func(k base.InternalKey, v []byte) error {
				k.SetSeqNum(0)
				return w.Add(k, v)
			})
			if err != nil {
				return err
			}
		}
	}

	if rki := b.newRangeKeyIter(nil, math.MaxUint64); rki != nil {
		for s := rki.First(); s != nil; s = rki.Next() {
			for _, k := range s.Keys {
				var err error
				switch k.Kind() {
				case base.InternalKeyKindRangeKeySet:
					err = w.RangeKeySet(s.Start, s.End, k.Suffix, k.Value)
				case base.InternalKeyKindRangeKeyUnset:
					err = w.RangeKeyUnset(s.Start, s.End, k.Suffix)
				case base.InternalKeyKindRangeKeyDelete:
					err = w.RangeKeyDelete(s.Start, s.End)
				default:
					panic("not a range key")
				}
				if err != nil {
					return err
				}
			}
		}
	}

	return w.Close()
}

func runCompactCmd(td *datadriven.TestData, d *DB) error {
	if len(td.CmdArgs) > 4 {
		return errors.Errorf("%s expects at most four arguments", td.Cmd)
	}
	parts := strings.Split(td.CmdArgs[0].Key, "-")
	if len(parts) != 2 {
		return errors.Errorf("expected <begin>-<end>: %s", td.Input)
	}
	parallelize := td.HasArg("parallel")
	if len(td.CmdArgs) >= 2 && strings.HasPrefix(td.CmdArgs[1].Key, "L") {
		levelString := td.CmdArgs[1].String()
		iStart := base.MakeInternalKey([]byte(parts[0]), InternalKeySeqNumMax, InternalKeyKindMax)
		iEnd := base.MakeInternalKey([]byte(parts[1]), 0, 0)
		if levelString[0] != 'L' {
			return errors.Errorf("expected L<n>: %s", levelString)
		}
		level, err := strconv.Atoi(levelString[1:])
		if err != nil {
			return err
		}
		return d.manualCompact(iStart.UserKey, iEnd.UserKey, level, parallelize)
	}
	return d.Compact([]byte(parts[0]), []byte(parts[1]), parallelize)
}

// runDBDefineCmd prepares a database state, returning the opened
// database with the initialized state.
//
// The command accepts input describing memtables and sstables to
// construct. Each new table is indicated by a line containing the
// level of the next table to build (eg, "L6"), or "mem" to build
// a memtable. Each subsequent line contains a new key-value pair.
//
// Point keys and range deletions should be encoded as the
// InternalKey's string representation, as understood by
// ParseInternalKey, followed a colon and the corresponding value.
//
//	b.SET.50:foo
//	c.DEL.20
//
// Range keys may be encoded by prefixing the line with `rangekey:`,
// followed by the keyspan.Span string representation, as understood
// by keyspan.ParseSpan.
//
//	rangekey:b-d:{(#5,RANGEKEYSET,@2,foo)}
//
// # Mechanics
//
// runDBDefineCmd works by simulating a flush for every file written.
// Keys are written to a memtable. When a file is complete, the table
// is flushed to physical files through manually invoking runCompaction.
// The resulting version edit is then manipulated to write the files
// to the indicated level.
//
// Because of it's low-level manipulation, runDBDefineCmd does allow the
// creation of invalid database states. If opts.DebugCheck is set, the
// level checker should detect the invalid state.
func runDBDefineCmd(td *datadriven.TestData, opts *Options) (*DB, error) {
	opts = opts.EnsureDefaults()
	opts.FS = vfs.NewMem()

	var snapshots []uint64
	var levelMaxBytes map[int]int64
	for _, arg := range td.CmdArgs {
		switch arg.Key {
		case "target-file-sizes":
			opts.Levels = make([]LevelOptions, len(arg.Vals))
			for i := range arg.Vals {
				size, err := strconv.ParseInt(arg.Vals[i], 10, 64)
				if err != nil {
					return nil, err
				}
				opts.Levels[i].TargetFileSize = size
			}
		case "snapshots":
			snapshots = make([]uint64, len(arg.Vals))
			for i := range arg.Vals {
				seqNum, err := strconv.ParseUint(arg.Vals[i], 10, 64)
				if err != nil {
					return nil, err
				}
				snapshots[i] = seqNum
				if i > 0 && snapshots[i] < snapshots[i-1] {
					return nil, errors.New("Snapshots must be in ascending order")
				}
			}
		case "lbase-max-bytes":
			lbaseMaxBytes, err := strconv.ParseInt(arg.Vals[0], 10, 64)
			if err != nil {
				return nil, err
			}
			opts.LBaseMaxBytes = lbaseMaxBytes
		case "level-max-bytes":
			levelMaxBytes = map[int]int64{}
			for i := range arg.Vals {
				j := strings.Index(arg.Vals[i], ":")
				levelStr := strings.TrimSpace(arg.Vals[i][:j])
				level, err := strconv.Atoi(levelStr[1:])
				if err != nil {
					return nil, err
				}
				size, err := strconv.ParseInt(strings.TrimSpace(arg.Vals[i][j+1:]), 10, 64)
				if err != nil {
					return nil, err
				}
				levelMaxBytes[level] = size
			}
		case "auto-compactions":
			switch arg.Vals[0] {
			case "off":
				opts.DisableAutomaticCompactions = true
			case "on":
				opts.DisableAutomaticCompactions = false
			default:
				return nil, errors.Errorf("Unrecognized %q %q arg value: %q", td.Cmd, arg.Key, arg.Vals[0])
			}
		case "enable-table-stats":
			enable, err := strconv.ParseBool(arg.Vals[0])
			if err != nil {
				return nil, errors.Errorf("%s: could not parse %q as bool: %s", td.Cmd, arg.Vals[0], err)
			}
			opts.private.disableTableStats = !enable
		case "block-size":
			size, err := strconv.Atoi(arg.Vals[0])
			if err != nil {
				return nil, err
			}
			for _, levelOpts := range opts.Levels {
				levelOpts.BlockSize = size
			}
		case "format-major-version":
			fmv, err := strconv.Atoi(arg.Vals[0])
			if err != nil {
				return nil, err
			}
			opts.FormatMajorVersion = FormatMajorVersion(fmv)
		case "disable-multi-level":
			opts.Experimental.MultiLevelCompactionHeuristic = NoMultiLevel{}
		}
	}

	// This is placed after the argument parsing above, because the arguments
	// to define should be parsed even if td.Input is empty.
	if td.Input == "" {
		// Empty LSM.
		d, err := Open("", opts)
		if err != nil {
			return nil, err
		}
		d.mu.Lock()
		for i := range snapshots {
			s := &Snapshot{db: d}
			s.seqNum = snapshots[i]
			d.mu.snapshots.pushBack(s)
		}
		for l, maxBytes := range levelMaxBytes {
			d.mu.versions.picker.(*compactionPickerByScore).levelMaxBytes[l] = maxBytes
		}
		d.mu.Unlock()
		return d, nil
	}

	d, err := Open("", opts)
	if err != nil {
		return nil, err
	}
	d.mu.Lock()
	d.mu.versions.dynamicBaseLevel = false
	for i := range snapshots {
		s := &Snapshot{db: d}
		s.seqNum = snapshots[i]
		d.mu.snapshots.pushBack(s)
	}
	defer d.mu.Unlock()

	var mem *memTable
	var start, end *base.InternalKey
	ve := &versionEdit{}
	level := -1

	maybeFlush := func() error {
		if level < 0 {
			return nil
		}

		toFlush := flushableList{{
			flushable: mem,
			flushed:   make(chan struct{}),
		}}
		c := newFlush(d.opts, d.mu.versions.currentVersion(),
			d.mu.versions.picker.getBaseLevel(), toFlush, time.Now())
		c.disableSpanElision = true
		// NB: define allows the test to exactly specify which keys go
		// into which sstables. If the test has a small target file
		// size to test grandparent limits, etc, the maxOutputFileSize
		// can cause splitting /within/ the bounds specified to the
		// test. Ignore the target size here, and split only according
		// to the user-defined boundaries.
		c.maxOutputFileSize = math.MaxUint64

		newVE, _, _, err := d.runCompaction(0, c)
		if err != nil {
			return err
		}
		largestSeqNum := d.mu.versions.logSeqNum.Load()
		for _, f := range newVE.NewFiles {
			if start != nil {
				f.Meta.SmallestPointKey = *start
				f.Meta.Smallest = *start
			}
			if end != nil {
				f.Meta.LargestPointKey = *end
				f.Meta.Largest = *end
			}
			if largestSeqNum <= f.Meta.LargestSeqNum {
				largestSeqNum = f.Meta.LargestSeqNum + 1
			}
			ve.NewFiles = append(ve.NewFiles, newFileEntry{
				Level: level,
				Meta:  f.Meta,
			})
		}
		// The committed keys were never written to the WAL, so neither
		// the logSeqNum nor the commit pipeline's visibleSeqNum have
		// been ratcheted. Manually ratchet them to the largest sequence
		// number committed to ensure iterators opened from the database
		// correctly observe the committed keys.
		if d.mu.versions.logSeqNum.Load() < largestSeqNum {
			d.mu.versions.logSeqNum.Store(largestSeqNum)
		}
		if d.mu.versions.visibleSeqNum.Load() < largestSeqNum {
			d.mu.versions.visibleSeqNum.Store(largestSeqNum)
		}
		level = -1
		return nil
	}

	// Example, a-c.
	parseMeta := func(s string) (*fileMetadata, error) {
		parts := strings.Split(s, "-")
		if len(parts) != 2 {
			return nil, errors.Errorf("malformed table spec: %s", s)
		}
		m := (&fileMetadata{}).ExtendPointKeyBounds(
			opts.Comparer.Compare,
			InternalKey{UserKey: []byte(parts[0])},
			InternalKey{UserKey: []byte(parts[1])},
		)
		m.InitPhysicalBacking()
		return m, nil
	}

	// Example, compact: a-c.
	parseCompaction := func(outputLevel int, s string) (*compaction, error) {
		m, err := parseMeta(s[len("compact:"):])
		if err != nil {
			return nil, err
		}
		c := &compaction{
			inputs:   []compactionLevel{{}, {level: outputLevel}},
			smallest: m.Smallest,
			largest:  m.Largest,
		}
		c.startLevel, c.outputLevel = &c.inputs[0], &c.inputs[1]
		return c, nil
	}

	for _, line := range strings.Split(td.Input, "\n") {
		fields := strings.Fields(line)
		if len(fields) > 0 {
			switch fields[0] {
			case "mem":
				if err := maybeFlush(); err != nil {
					return nil, err
				}
				// Add a memtable layer.
				if !d.mu.mem.mutable.empty() {
					d.mu.mem.mutable = newMemTable(memTableOptions{Options: d.opts})
					entry := d.newFlushableEntry(d.mu.mem.mutable, 0, 0)
					entry.readerRefs.Add(1)
					d.mu.mem.queue = append(d.mu.mem.queue, entry)
					d.updateReadStateLocked(nil)
				}
				mem = d.mu.mem.mutable
				start, end = nil, nil
				fields = fields[1:]
			case "L0", "L1", "L2", "L3", "L4", "L5", "L6":
				if err := maybeFlush(); err != nil {
					return nil, err
				}
				var err error
				if level, err = strconv.Atoi(fields[0][1:]); err != nil {
					return nil, err
				}
				fields = fields[1:]
				start, end = nil, nil
				boundFields := 0
				for _, field := range fields {
					toBreak := false
					switch {
					case strings.HasPrefix(field, "start="):
						ikey := base.ParseInternalKey(strings.TrimPrefix(field, "start="))
						start = &ikey
						boundFields++
					case strings.HasPrefix(field, "end="):
						ikey := base.ParseInternalKey(strings.TrimPrefix(field, "end="))
						end = &ikey
						boundFields++
					default:
						toBreak = true
					}
					if toBreak {
						break
					}
				}
				fields = fields[boundFields:]
				mem = newMemTable(memTableOptions{Options: d.opts})
			}
		}

		for _, data := range fields {
			i := strings.Index(data, ":")
			// Define in-progress compactions.
			if data[:i] == "compact" {
				c, err := parseCompaction(level, data)
				if err != nil {
					return nil, err
				}
				d.mu.compact.inProgress[c] = struct{}{}
				continue
			}
			if data[:i] == "rangekey" {
				span := keyspan.ParseSpan(data[i:])
				err := rangekey.Encode(&span, func(k base.InternalKey, v []byte) error {
					return mem.set(k, v)
				})
				if err != nil {
					return nil, err
				}
				continue
			}
			key := base.ParseInternalKey(data[:i])
			valueStr := data[i+1:]
			value := []byte(valueStr)
			var randBytes int
			if n, err := fmt.Sscanf(valueStr, "<rand-bytes=%d>", &randBytes); err == nil && n == 1 {
				value = make([]byte, randBytes)
				rnd := rand.New(rand.NewSource(int64(key.SeqNum())))
				if _, err := rnd.Read(value[:]); err != nil {
					return nil, err
				}
			}
			if err := mem.set(key, value); err != nil {
				return nil, err
			}
		}
	}

	if err := maybeFlush(); err != nil {
		return nil, err
	}

	if len(ve.NewFiles) > 0 {
		jobID := d.mu.nextJobID
		d.mu.nextJobID++
		d.mu.versions.logLock()
		if err := d.mu.versions.logAndApply(jobID, ve, newFileMetrics(ve.NewFiles), false, func() []compactionInfo {
			return nil
		}); err != nil {
			return nil, err
		}
		d.updateReadStateLocked(nil)
		d.updateTableStatsLocked(ve.NewFiles)
	}

	for l, maxBytes := range levelMaxBytes {
		d.mu.versions.picker.(*compactionPickerByScore).levelMaxBytes[l] = maxBytes
	}

	return d, nil
}

func runTableStatsCmd(td *datadriven.TestData, d *DB) string {
	u, err := strconv.ParseUint(strings.TrimSpace(td.Input), 10, 64)
	if err != nil {
		return err.Error()
	}
	fileNum := base.FileNum(u)

	d.mu.Lock()
	defer d.mu.Unlock()
	v := d.mu.versions.currentVersion()
	for _, levelMetadata := range v.Levels {
		iter := levelMetadata.Iter()
		for f := iter.First(); f != nil; f = iter.Next() {
			if f.FileNum != fileNum {
				continue
			}

			if !f.StatsValid() {
				d.waitTableStats()
			}

			var b bytes.Buffer
			fmt.Fprintf(&b, "num-entries: %d\n", f.Stats.NumEntries)
			fmt.Fprintf(&b, "num-deletions: %d\n", f.Stats.NumDeletions)
			fmt.Fprintf(&b, "num-range-key-sets: %d\n", f.Stats.NumRangeKeySets)
			fmt.Fprintf(&b, "point-deletions-bytes-estimate: %d\n", f.Stats.PointDeletionsBytesEstimate)
			fmt.Fprintf(&b, "range-deletions-bytes-estimate: %d\n", f.Stats.RangeDeletionsBytesEstimate)
			return b.String()
		}
	}
	return "(not found)"
}

func runTableFileSizesCmd(td *datadriven.TestData, d *DB) string {
	d.mu.Lock()
	defer d.mu.Unlock()
	return runVersionFileSizes(d.mu.versions.currentVersion())
}

func runVersionFileSizes(v *version) string {
	var buf bytes.Buffer
	for l, levelMetadata := range v.Levels {
		if levelMetadata.Empty() {
			continue
		}
		fmt.Fprintf(&buf, "L%d:\n", l)
		iter := levelMetadata.Iter()
		for f := iter.First(); f != nil; f = iter.Next() {
			fmt.Fprintf(&buf, "  %s: %d bytes (%s)", f, f.Size, humanize.Bytes.Uint64(f.Size))
			if f.IsCompacting() {
				fmt.Fprintf(&buf, " (IsCompacting)")
			}
			fmt.Fprintln(&buf)
		}
	}
	return buf.String()
}

// Prints some metadata about some sstable which is currently in the latest
// version.
func runMetadataCommand(t *testing.T, td *datadriven.TestData, d *DB) string {
	var file int
	td.ScanArgs(t, "file", &file)
	var m *fileMetadata
	d.mu.Lock()
	currVersion := d.mu.versions.currentVersion()
	for _, level := range currVersion.Levels {
		lIter := level.Iter()
		for f := lIter.First(); f != nil; f = lIter.Next() {
			if f.FileNum == base.FileNum(uint64(file)) {
				m = f
				break
			}
		}
	}
	d.mu.Unlock()
	var buf bytes.Buffer
	// Add more metadata as needed.
	fmt.Fprintf(&buf, "size: %d\n", m.Size)
	return buf.String()
}

func runSSTablePropertiesCmd(t *testing.T, td *datadriven.TestData, d *DB) string {
	var file int
	td.ScanArgs(t, "file", &file)

	// See if we can grab the FileMetadata associated with the file. This is needed
	// to easily construct virtual sstable properties.
	var m *fileMetadata
	d.mu.Lock()
	currVersion := d.mu.versions.currentVersion()
	for _, level := range currVersion.Levels {
		lIter := level.Iter()
		for f := lIter.First(); f != nil; f = lIter.Next() {
			if f.FileNum == base.FileNum(uint64(file)) {
				m = f
				break
			}
		}
	}
	d.mu.Unlock()

	// Note that m can be nil here if the sstable exists in the file system, but
	// not in the lsm. If m is nil just assume that file is not virtual.

	backingFileNum := base.FileNum(uint64(file)).DiskFileNum()
	if m != nil {
		backingFileNum = m.FileBacking.DiskFileNum
	}
	fileName := base.MakeFilename(fileTypeTable, backingFileNum)
	f, err := d.opts.FS.Open(fileName)
	if err != nil {
		return err.Error()
	}
	readable, err := sstable.NewSimpleReadable(f)
	if err != nil {
		return err.Error()
	}
	// TODO(bananabrick): cacheOpts is used to set the file number on a Reader,
	// and virtual sstables expect this file number to be set. Split out the
	// opts into fileNum opts, and cache opts.
	cacheOpts := private.SSTableCacheOpts(0, backingFileNum).(sstable.ReaderOption)
	r, err := sstable.NewReader(readable, d.opts.MakeReaderOptions(), cacheOpts)
	if err != nil {
		return err.Error()
	}
	defer r.Close()

	var v sstable.VirtualReader
	props := r.Properties.String()
	if m != nil && m.Virtual {
		v = sstable.MakeVirtualReader(r, m.VirtualMeta(), false /* isForeign */)
		props = v.Properties.String()
	}
	if len(td.Input) == 0 {
		return props
	}
	var buf bytes.Buffer
	propsSlice := strings.Split(props, "\n")
	for _, requestedProp := range strings.Split(td.Input, "\n") {
		fmt.Fprintf(&buf, "%s:\n", requestedProp)
		for _, prop := range propsSlice {
			if strings.Contains(prop, requestedProp) {
				fmt.Fprintf(&buf, "  %s\n", prop)
			}
		}
	}
	return buf.String()
}

func runPopulateCmd(t *testing.T, td *datadriven.TestData, b *Batch) {
	var maxKeyLength, valLength int
	var timestamps []int
	td.ScanArgs(t, "keylen", &maxKeyLength)
	td.MaybeScanArgs(t, "timestamps", &timestamps)
	td.MaybeScanArgs(t, "vallen", &valLength)
	// Default to writing timestamps @1.
	if len(timestamps) == 0 {
		timestamps = append(timestamps, 1)
	}

	ks := testkeys.Alpha(maxKeyLength)
	buf := make([]byte, ks.MaxLen()+testkeys.MaxSuffixLen)
	vbuf := make([]byte, valLength)
	for i := int64(0); i < ks.Count(); i++ {
		for _, ts := range timestamps {
			n := testkeys.WriteKeyAt(buf, ks, i, int64(ts))

			// Default to using the key as the value, but if the user provided
			// the vallen argument, generate a random value of the specified
			// length.
			value := buf[:n]
			if valLength > 0 {
				_, err := crand.Read(vbuf)
				require.NoError(t, err)
				value = vbuf
			}
			require.NoError(t, b.Set(buf[:n], value, nil))
		}
	}
}

// waitTableStats waits until all new files' statistics have been loaded. It's
// used in tests. The d.mu mutex must be locked while calling this method.
func (d *DB) waitTableStats() {
	for d.mu.tableStats.loading || len(d.mu.tableStats.pending) > 0 {
		d.mu.tableStats.cond.Wait()
	}
}

func runIngestAndExciseCmd(td *datadriven.TestData, d *DB, fs vfs.FS) error {
	var exciseSpan KeyRange
	paths := make([]string, 0, len(td.CmdArgs))
	for i, arg := range td.CmdArgs {
		switch td.CmdArgs[i].Key {
		case "excise":
			if len(td.CmdArgs[i].Vals) != 1 {
				return errors.New("expected 2 values for excise separated by -, eg. ingest-and-excise foo1 excise=\"start-end\"")
			}
			fields := strings.Split(td.CmdArgs[i].Vals[0], "-")
			if len(fields) != 2 {
				return errors.New("expected 2 values for excise separated by -, eg. ingest-and-excise foo1 excise=\"start-end\"")
			}
			exciseSpan.Start = []byte(fields[0])
			exciseSpan.End = []byte(fields[1])
		default:
			paths = append(paths, arg.String())
		}
	}

	if _, err := d.IngestAndExcise(paths, nil /* shared */, exciseSpan); err != nil {
		return err
	}
	return nil
}

func runIngestCmd(td *datadriven.TestData, d *DB, fs vfs.FS) error {
	paths := make([]string, 0, len(td.CmdArgs))
	for _, arg := range td.CmdArgs {
		paths = append(paths, arg.String())
	}

	if err := d.Ingest(paths); err != nil {
		return err
	}
	return nil
}

func runIngestExternalCmd(td *datadriven.TestData, d *DB, locator string) error {
	external := make([]ExternalFile, 0)
	for _, arg := range strings.Split(td.Input, "\n") {
		fields := strings.Split(arg, ",")
		if len(fields) != 4 {
			return errors.New("usage: path,size,smallest,largest")
		}
		ef := ExternalFile{}
		ef.Locator = remote.Locator(locator)
		ef.ObjName = fields[0]
		sizeInt, err := strconv.Atoi(fields[1])
		if err != nil {
			return err
		}
		ef.Size = uint64(sizeInt)
		ef.SmallestUserKey = []byte(fields[2])
		ef.LargestUserKey = []byte(fields[3])
		ef.HasPointKey = true
		external = append(external, ef)
	}

	if _, err := d.IngestExternalFiles(external); err != nil {
		return err
	}
	return nil
}

func runForceIngestCmd(td *datadriven.TestData, d *DB) error {
	var paths []string
	var level int
	for _, arg := range td.CmdArgs {
		switch arg.Key {
		case "paths":
			paths = append(paths, arg.Vals...)
		case "level":
			var err error
			level, err = strconv.Atoi(arg.Vals[0])
			if err != nil {
				return err
			}
		}
	}
	_, err := d.ingest(paths, func(
		tableNewIters,
		keyspan.TableNewSpanIter,
		IterOptions,
		*Comparer,
		*version,
		int,
		map[*compaction]struct{},
		*fileMetadata,
		bool,
	) (int, *fileMetadata, error) {
		return level, nil, nil
	}, nil /* shared */, KeyRange{}, nil /* external */)
	return err
}

func runLSMCmd(td *datadriven.TestData, d *DB) string {
	d.mu.Lock()
	defer d.mu.Unlock()
	if td.HasArg("verbose") {
		return d.mu.versions.currentVersion().DebugString(d.opts.Comparer.FormatKey)
	}
	return d.mu.versions.currentVersion().String()
}
