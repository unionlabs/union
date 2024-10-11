// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"strconv"
	"strings"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/bloom"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/cache"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/vfs"
)

func optsFromArgs(td *datadriven.TestData, writerOpts *WriterOptions) error {
	for _, arg := range td.CmdArgs {
		switch arg.Key {
		case "leveldb":
			if len(arg.Vals) != 0 {
				return errors.Errorf("%s: arg %s expects 0 values", td.Cmd, arg.Key)
			}
			writerOpts.TableFormat = TableFormatLevelDB
		case "block-size":
			if len(arg.Vals) != 1 {
				return errors.Errorf("%s: arg %s expects 1 value", td.Cmd, arg.Key)
			}
			var err error
			writerOpts.BlockSize, err = strconv.Atoi(arg.Vals[0])
			if err != nil {
				return err
			}
		case "index-block-size":
			if len(arg.Vals) != 1 {
				return errors.Errorf("%s: arg %s expects 1 value", td.Cmd, arg.Key)
			}
			var err error
			writerOpts.IndexBlockSize, err = strconv.Atoi(arg.Vals[0])
			if err != nil {
				return err
			}
		case "filter":
			writerOpts.FilterPolicy = bloom.FilterPolicy(10)
		case "comparer-split-4b-suffix":
			writerOpts.Comparer = test4bSuffixComparer
		case "writing-to-lowest-level":
			writerOpts.WritingToLowestLevel = true
		case "is-strict-obsolete":
			writerOpts.IsStrictObsolete = true
		}
	}
	return nil
}

func runBuildCmd(
	td *datadriven.TestData, writerOpts *WriterOptions, cacheSize int,
) (*WriterMetadata, *Reader, error) {

	f0 := &memFile{}
	if err := optsFromArgs(td, writerOpts); err != nil {
		return nil, nil, err
	}

	w := NewWriter(f0, *writerOpts)
	var rangeDels []keyspan.Span
	rangeDelFrag := keyspan.Fragmenter{
		Cmp:    DefaultComparer.Compare,
		Format: DefaultComparer.FormatKey,
		Emit: func(s keyspan.Span) {
			rangeDels = append(rangeDels, s)
		},
	}
	var rangeKeys []keyspan.Span
	rangeKeyFrag := keyspan.Fragmenter{
		Cmp:    DefaultComparer.Compare,
		Format: DefaultComparer.FormatKey,
		Emit: func(s keyspan.Span) {
			rangeKeys = append(rangeKeys, s)
		},
	}
	for _, data := range strings.Split(td.Input, "\n") {
		if strings.HasPrefix(data, "rangekey:") {
			var err error
			func() {
				defer func() {
					if r := recover(); r != nil {
						err = errors.Errorf("%v", r)
					}
				}()
				rangeKeyFrag.Add(keyspan.ParseSpan(strings.TrimPrefix(data, "rangekey:")))
			}()
			if err != nil {
				return nil, nil, err
			}
			continue
		}

		forceObsolete := false
		if strings.HasPrefix(data, "force-obsolete:") {
			data = strings.TrimSpace(strings.TrimPrefix(data, "force-obsolete:"))
			forceObsolete = true
		}
		j := strings.Index(data, ":")
		key := base.ParseInternalKey(data[:j])
		value := []byte(data[j+1:])
		switch key.Kind() {
		case InternalKeyKindRangeDelete:
			if forceObsolete {
				return nil, nil, errors.Errorf("force-obsolete is not allowed for RANGEDEL")
			}
			var err error
			func() {
				defer func() {
					if r := recover(); r != nil {
						err = errors.Errorf("%v", r)
					}
				}()
				rangeDelFrag.Add(keyspan.Span{
					Start: key.UserKey,
					End:   value,
					Keys:  []keyspan.Key{{Trailer: key.Trailer}},
				})
			}()
			if err != nil {
				return nil, nil, err
			}
		default:
			if err := w.AddWithForceObsolete(key, value, forceObsolete); err != nil {
				return nil, nil, err
			}
		}
	}
	rangeDelFrag.Finish()
	for _, v := range rangeDels {
		for _, k := range v.Keys {
			ik := base.InternalKey{UserKey: v.Start, Trailer: k.Trailer}
			if err := w.Add(ik, v.End); err != nil {
				return nil, nil, err
			}
		}
	}
	rangeKeyFrag.Finish()
	for _, s := range rangeKeys {
		if err := w.addRangeKeySpan(s); err != nil {
			return nil, nil, err
		}
	}
	if err := w.Close(); err != nil {
		return nil, nil, err
	}
	meta, err := w.Metadata()
	if err != nil {
		return nil, nil, err
	}

	readerOpts := ReaderOptions{Comparer: writerOpts.Comparer}
	if writerOpts.FilterPolicy != nil {
		readerOpts.Filters = map[string]FilterPolicy{
			writerOpts.FilterPolicy.Name(): writerOpts.FilterPolicy,
		}
	}
	if cacheSize > 0 {
		readerOpts.Cache = cache.New(int64(cacheSize))
		defer readerOpts.Cache.Unref()
	}
	r, err := NewMemReader(f0.Data(), readerOpts)
	if err != nil {
		return nil, nil, err
	}
	return meta, r, nil
}

func runBuildRawCmd(
	td *datadriven.TestData, opts *WriterOptions,
) (*WriterMetadata, *Reader, error) {
	mem := vfs.NewMem()
	provider, err := objstorageprovider.Open(objstorageprovider.DefaultSettings(mem, "" /* dirName */))
	if err != nil {
		return nil, nil, err
	}
	defer provider.Close()

	f0, _, err := provider.Create(context.Background(), base.FileTypeTable, base.FileNum(0).DiskFileNum(), objstorage.CreateOptions{})
	if err != nil {
		return nil, nil, err
	}

	w := NewWriter(f0, *opts)
	for i := range td.CmdArgs {
		arg := &td.CmdArgs[i]
		if arg.Key == "range-del-v1" {
			w.rangeDelV1Format = true
			break
		}
	}

	for _, data := range strings.Split(td.Input, "\n") {
		if strings.HasPrefix(data, "rangekey:") {
			data = strings.TrimPrefix(data, "rangekey:")
			if err := w.addRangeKeySpan(keyspan.ParseSpan(data)); err != nil {
				return nil, nil, err
			}
			continue
		}

		j := strings.Index(data, ":")
		key := base.ParseInternalKey(data[:j])
		value := []byte(data[j+1:])
		switch key.Kind() {
		case base.InternalKeyKindRangeKeyDelete,
			base.InternalKeyKindRangeKeyUnset,
			base.InternalKeyKindRangeKeySet:
			if err := w.AddRangeKey(key, value); err != nil {
				return nil, nil, err
			}
		default:
			if err := w.Add(key, value); err != nil {
				return nil, nil, err
			}
		}
	}
	if err := w.Close(); err != nil {
		return nil, nil, err
	}
	meta, err := w.Metadata()
	if err != nil {
		return nil, nil, err
	}

	f1, err := provider.OpenForReading(context.Background(), base.FileTypeTable, base.FileNum(0).DiskFileNum(), objstorage.OpenOptions{})
	if err != nil {
		return nil, nil, err
	}
	r, err := NewReader(f1, ReaderOptions{})
	if err != nil {
		return nil, nil, err
	}
	return meta, r, nil
}

func scanGlobalSeqNum(td *datadriven.TestData) (uint64, error) {
	for _, arg := range td.CmdArgs {
		switch arg.Key {
		case "globalSeqNum":
			if len(arg.Vals) != 1 {
				return 0, errors.Errorf("%s: arg %s expects 1 value", td.Cmd, arg.Key)
			}
			v, err := strconv.Atoi(arg.Vals[0])
			if err != nil {
				return 0, err
			}
			return uint64(v), nil
		}
	}
	return 0, nil
}

type runIterCmdOption func(*runIterCmdOptions)

type runIterCmdOptions struct {
	everyOp      func(io.Writer)
	everyOpAfter func(io.Writer)
	stats        *base.InternalIteratorStats
}

func runIterCmdEveryOp(everyOp func(io.Writer)) runIterCmdOption {
	return func(opts *runIterCmdOptions) { opts.everyOp = everyOp }
}

func runIterCmdEveryOpAfter(everyOp func(io.Writer)) runIterCmdOption {
	return func(opts *runIterCmdOptions) { opts.everyOpAfter = everyOp }
}

func runIterCmdStats(stats *base.InternalIteratorStats) runIterCmdOption {
	return func(opts *runIterCmdOptions) { opts.stats = stats }
}

func runIterCmd(
	td *datadriven.TestData, origIter Iterator, printValue bool, opt ...runIterCmdOption,
) string {
	var opts runIterCmdOptions
	for _, o := range opt {
		o(&opts)
	}

	iter := newIterAdapter(origIter)
	defer iter.Close()

	var b bytes.Buffer
	var prefix []byte
	for _, line := range strings.Split(td.Input, "\n") {
		parts := strings.Fields(line)
		if len(parts) == 0 {
			continue
		}
		switch parts[0] {
		case "seek-ge":
			if len(parts) < 2 || len(parts) > 3 {
				return "seek-ge <key> [<try-seek-using-next]\n"
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
			iter.SeekGE([]byte(strings.TrimSpace(parts[1])), flags)
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
			iter.SeekPrefixGE(prefix, prefix /* key */, flags)
		case "seek-lt":
			if len(parts) != 2 {
				return "seek-lt <key>\n"
			}
			prefix = nil
			iter.SeekLT([]byte(strings.TrimSpace(parts[1])), base.SeekLTFlagsNone)
		case "first":
			prefix = nil
			iter.First()
		case "last":
			prefix = nil
			iter.Last()
		case "next":
			iter.Next()
		case "next-ignore-result":
			iter.NextIgnoreResult()
		case "prev":
			iter.Prev()
		case "next-prefix":
			if len(parts) != 1 {
				return "next-prefix should have no parameter\n"
			}
			if iter.Key() == nil {
				return "next-prefix cannot be called on exhauster iterator\n"
			}
			k := iter.Key().UserKey
			prefixLen := testkeys.Comparer.Split(k)
			k = k[:prefixLen]
			kSucc := testkeys.Comparer.ImmediateSuccessor(nil, k)
			iter.NextPrefix(kSucc)
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
					if len(lower) == 0 {
						lower = nil
					}
				case "upper":
					upper = []byte(arg[1])
					if len(upper) == 0 {
						upper = nil
					}
				default:
					return fmt.Sprintf("set-bounds: unknown arg: %s", arg)
				}
			}
			iter.SetBounds(lower, upper)
		case "stats":
			// The timing is non-deterministic, so set to 0.
			opts.stats.BlockReadDuration = 0
			fmt.Fprintf(&b, "%+v\n", *opts.stats)
			continue
		case "reset-stats":
			*opts.stats = base.InternalIteratorStats{}
			continue
		case "internal-iter-state":
			fmt.Fprintf(&b, "| %T:\n", origIter)
			si, _ := origIter.(*singleLevelIterator)
			if twoLevelIter, ok := origIter.(*twoLevelIterator); ok {
				si = &twoLevelIter.singleLevelIterator
				if twoLevelIter.topLevelIndex.valid() {
					fmt.Fprintf(&b, "|  topLevelIndex.Key() = %q\n", twoLevelIter.topLevelIndex.Key())
					v := twoLevelIter.topLevelIndex.value()
					bhp, err := decodeBlockHandleWithProperties(v.InPlaceValue())
					if err != nil {
						fmt.Fprintf(&b, "|  topLevelIndex.InPlaceValue() failed to decode as BHP: %s\n", err)
					} else {
						fmt.Fprintf(&b, "|  topLevelIndex.InPlaceValue() = (Offset: %d, Length: %d, Props: %x)\n",
							bhp.Offset, bhp.Length, bhp.Props)
					}
				} else {
					fmt.Fprintf(&b, "|  topLevelIndex iter invalid\n")
				}
				fmt.Fprintf(&b, "|  topLevelIndex.isDataInvalidated()=%t\n", twoLevelIter.topLevelIndex.isDataInvalidated())
			}
			if si.index.valid() {
				fmt.Fprintf(&b, "|  index.Key() = %q\n", si.index.Key())
				v := si.index.value()
				bhp, err := decodeBlockHandleWithProperties(v.InPlaceValue())
				if err != nil {
					fmt.Fprintf(&b, "|  index.InPlaceValue() failed to decode as BHP: %s\n", err)
				} else {
					fmt.Fprintf(&b, "|  index.InPlaceValue() = (Offset: %d, Length: %d, Props: %x)\n",
						bhp.Offset, bhp.Length, bhp.Props)
				}
			} else {
				fmt.Fprintf(&b, "|  index iter invalid\n")
			}
			fmt.Fprintf(&b, "|  index.isDataInvalidated()=%t\n", si.index.isDataInvalidated())
			fmt.Fprintf(&b, "|  data.isDataInvalidated()=%t\n", si.data.isDataInvalidated())
			fmt.Fprintf(&b, "|  hideObsoletePoints = %t\n", si.hideObsoletePoints)
			fmt.Fprintf(&b, "|  dataBH = (Offset: %d, Length: %d)\n", si.dataBH.Offset, si.dataBH.Length)
			fmt.Fprintf(&b, "|  (boundsCmp,positionedUsingLatestBounds) = (%d,%t)\n", si.boundsCmp, si.positionedUsingLatestBounds)
			fmt.Fprintf(&b, "|  exhaustedBounds = %d\n", si.exhaustedBounds)

			continue
		}
		if opts.everyOp != nil {
			opts.everyOp(&b)
		}
		if iter.Valid() && checkValidPrefix(prefix, iter.Key().UserKey) {
			fmt.Fprintf(&b, "<%s:%d>", iter.Key().UserKey, iter.Key().SeqNum())
			if printValue {
				fmt.Fprintf(&b, ":%s", string(iter.Value()))
			}
		} else if err := iter.Error(); err != nil {
			fmt.Fprintf(&b, "<err=%v>", err)
		} else {
			fmt.Fprintf(&b, ".")
		}
		if opts.everyOpAfter != nil {
			opts.everyOpAfter(&b)
		}
		b.WriteString("\n")
	}
	return b.String()
}

func runRewriteCmd(
	td *datadriven.TestData, r *Reader, writerOpts WriterOptions,
) (*WriterMetadata, *Reader, error) {
	var from, to []byte
	for _, arg := range td.CmdArgs {
		switch arg.Key {
		case "from":
			from = []byte(arg.Vals[0])
		case "to":
			to = []byte(arg.Vals[0])
		}
	}
	if from == nil || to == nil {
		return nil, r, errors.New("missing from/to")
	}

	opts := writerOpts
	if err := optsFromArgs(td, &opts); err != nil {
		return nil, r, err
	}

	f := &memFile{}
	meta, _, err := rewriteKeySuffixesInBlocks(r, f, opts, from, to, 2)
	if err != nil {
		return nil, r, errors.Wrap(err, "rewrite failed")
	}
	readerOpts := ReaderOptions{Comparer: opts.Comparer}
	if opts.FilterPolicy != nil {
		readerOpts.Filters = map[string]FilterPolicy{
			opts.FilterPolicy.Name(): opts.FilterPolicy,
		}
	}
	r.Close()

	r, err = NewMemReader(f.Data(), readerOpts)
	if err != nil {
		return nil, nil, err
	}
	return meta, r, nil
}
