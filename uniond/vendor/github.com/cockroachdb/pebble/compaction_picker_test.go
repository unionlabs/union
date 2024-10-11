// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"fmt"
	"math"
	"sort"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/humanize"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

func loadVersion(t *testing.T, d *datadriven.TestData) (*version, *Options, string) {
	var sizes [numLevels]int64
	opts := &Options{}
	opts.testingRandomized(t)
	opts.EnsureDefaults()

	if len(d.CmdArgs) != 1 {
		return nil, nil, fmt.Sprintf("%s expects 1 argument", d.Cmd)
	}
	var err error
	opts.LBaseMaxBytes, err = strconv.ParseInt(d.CmdArgs[0].Key, 10, 64)
	if err != nil {
		return nil, nil, err.Error()
	}

	var files [numLevels][]*fileMetadata
	if len(d.Input) > 0 {
		// Parse each line as
		//
		// <level>: <size> [compensation]
		//
		// Creating sstables within the level whose file sizes total to `size`
		// and whose compensated file sizes total to `size`+`compensation`. If
		// size is sufficiently large, only one single file is created. See
		// the TODO below.
		for _, data := range strings.Split(d.Input, "\n") {
			parts := strings.Split(data, " ")
			parts[0] = strings.TrimSuffix(strings.TrimSpace(parts[0]), ":")
			if len(parts) < 2 {
				return nil, nil, fmt.Sprintf("malformed test:\n%s", d.Input)
			}
			level, err := strconv.Atoi(parts[0])
			if err != nil {
				return nil, nil, err.Error()
			}
			if files[level] != nil {
				return nil, nil, fmt.Sprintf("level %d already filled", level)
			}
			size, err := strconv.ParseUint(strings.TrimSpace(parts[1]), 10, 64)
			if err != nil {
				return nil, nil, err.Error()
			}
			var compensation uint64
			if len(parts) == 3 {
				compensation, err = strconv.ParseUint(strings.TrimSpace(parts[2]), 10, 64)
				if err != nil {
					return nil, nil, err.Error()
				}
			}

			var lastFile *fileMetadata
			for i := uint64(1); sizes[level] < int64(size); i++ {
				var key InternalKey
				if level == 0 {
					// For L0, make `size` overlapping files.
					key = base.MakeInternalKey([]byte(fmt.Sprintf("%04d", 1)), i, InternalKeyKindSet)
				} else {
					key = base.MakeInternalKey([]byte(fmt.Sprintf("%04d", i)), i, InternalKeyKindSet)
				}
				m := (&fileMetadata{
					FileNum:        base.FileNum(uint64(level)*100_000 + i),
					SmallestSeqNum: key.SeqNum(),
					LargestSeqNum:  key.SeqNum(),
					Size:           1,
					Stats: manifest.TableStats{
						RangeDeletionsBytesEstimate: 0,
					},
				}).ExtendPointKeyBounds(opts.Comparer.Compare, key, key)
				m.InitPhysicalBacking()
				m.StatsMarkValid()
				lastFile = m
				if size >= 100 {
					// If the requested size of the level is very large only add a single
					// file in order to avoid massive blow-up in the number of files in
					// the Version.
					//
					// TODO(peter): There is tension between the testing in
					// TestCompactionPickerLevelMaxBytes and
					// TestCompactionPickerTargetLevel. Clean this up somehow.
					m.Size = size
					if level != 0 {
						endKey := base.MakeInternalKey([]byte(fmt.Sprintf("%04d", size)), i, InternalKeyKindSet)
						m.ExtendPointKeyBounds(opts.Comparer.Compare, key, endKey)
					}
				}
				files[level] = append(files[level], m)
				sizes[level] += int64(m.Size)
			}
			// Let all the compensation be due to the last file.
			if lastFile != nil && compensation > 0 {
				lastFile.Stats.RangeDeletionsBytesEstimate = compensation
			}
		}
	}

	vers := newVersion(opts, files)
	return vers, opts, ""
}

func TestCompactionPickerByScoreLevelMaxBytes(t *testing.T) {
	datadriven.RunTest(t, "testdata/compaction_picker_level_max_bytes",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "init":
				vers, opts, errMsg := loadVersion(t, d)
				if errMsg != "" {
					return errMsg
				}

				p, ok := newCompactionPicker(vers, opts, nil).(*compactionPickerByScore)
				require.True(t, ok)
				var buf bytes.Buffer
				for level := p.getBaseLevel(); level < numLevels; level++ {
					fmt.Fprintf(&buf, "%d: %d\n", level, p.levelMaxBytes[level])
				}
				return buf.String()

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
}

func TestCompactionPickerTargetLevel(t *testing.T) {
	var vers *version
	var opts *Options
	var pickerByScore *compactionPickerByScore

	parseInProgress := func(vals []string) ([]compactionInfo, error) {
		var levels []int
		for _, s := range vals {
			l, err := strconv.ParseInt(s, 10, 8)
			if err != nil {
				return nil, err
			}
			levels = append(levels, int(l))
		}
		if len(levels)%2 != 0 {
			return nil, errors.New("odd number of levels with ongoing compactions")
		}
		var inProgress []compactionInfo
		for i := 0; i < len(levels); i += 2 {
			inProgress = append(inProgress, compactionInfo{
				inputs: []compactionLevel{
					{level: levels[i]},
					{level: levels[i+1]},
				},
				outputLevel: levels[i+1],
			})
		}
		return inProgress, nil
	}

	resetCompacting := func() {
		for _, files := range vers.Levels {
			files.Slice().Each(func(f *fileMetadata) {
				f.CompactionState = manifest.CompactionStateNotCompacting
			})
		}
	}

	datadriven.RunTest(t, "testdata/compaction_picker_target_level",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "init":
				// loadVersion expects a single datadriven argument that it
				// sets as Options.LBaseMaxBytes. It parses the input as
				// newline-separated levels, specifying the level's file size
				// and optionally additional compensation to be added during
				// compensated file size calculations. Eg:
				//
				// init <LBaseMaxBytes>
				// <level>: <size> [compensation]
				// <level>: <size> [compensation]
				var errMsg string
				vers, opts, errMsg = loadVersion(t, d)
				if errMsg != "" {
					return errMsg
				}
				return runVersionFileSizes(vers)
			case "init_cp":
				resetCompacting()

				var inProgress []compactionInfo
				if arg, ok := d.Arg("ongoing"); ok {
					var err error
					inProgress, err = parseInProgress(arg.Vals)
					if err != nil {
						return err.Error()
					}
				}

				p := newCompactionPicker(vers, opts, inProgress)
				var ok bool
				pickerByScore, ok = p.(*compactionPickerByScore)
				require.True(t, ok)
				return fmt.Sprintf("base: %d", pickerByScore.baseLevel)
			case "queue":
				var b strings.Builder
				var inProgress []compactionInfo
				for {
					env := compactionEnv{
						diskAvailBytes:          math.MaxUint64,
						earliestUnflushedSeqNum: InternalKeySeqNumMax,
						inProgressCompactions:   inProgress,
					}
					pc := pickerByScore.pickAuto(env)
					if pc == nil {
						break
					}
					fmt.Fprintf(&b, "L%d->L%d: %.1f\n", pc.startLevel.level, pc.outputLevel.level, pc.score)
					inProgress = append(inProgress, compactionInfo{
						inputs:      pc.inputs,
						outputLevel: pc.outputLevel.level,
						smallest:    pc.smallest,
						largest:     pc.largest,
					})
					if pc.outputLevel.level == 0 {
						// Once we pick one L0->L0 compaction, we'll keep on doing so
						// because the test isn't marking files as Compacting.
						break
					}
					for _, cl := range pc.inputs {
						cl.files.Each(func(f *fileMetadata) {
							f.CompactionState = manifest.CompactionStateCompacting
							fmt.Fprintf(&b, "  %s marked as compacting\n", f)
						})
					}
				}

				resetCompacting()
				return b.String()
			case "pick":
				resetCompacting()

				var inProgress []compactionInfo
				if len(d.CmdArgs) == 1 {
					arg := d.CmdArgs[0]
					if arg.Key != "ongoing" {
						return "unknown arg: " + arg.Key
					}
					var err error
					inProgress, err = parseInProgress(arg.Vals)
					if err != nil {
						return err.Error()
					}
				}

				// Mark files as compacting for each in-progress compaction.
				for i := range inProgress {
					c := &inProgress[i]
					for j, cl := range c.inputs {
						iter := vers.Levels[cl.level].Iter()
						for f := iter.First(); f != nil; f = iter.Next() {
							if !f.IsCompacting() {
								f.CompactionState = manifest.CompactionStateCompacting
								c.inputs[j].files = iter.Take().Slice()
								break
							}
						}
					}
					if c.inputs[0].level == 0 && c.outputLevel != 0 {
						// L0->Lbase: mark all of Lbase as compacting.
						c.inputs[1].files = vers.Levels[c.outputLevel].Slice()
						for _, in := range c.inputs {
							in.files.Each(func(f *fileMetadata) {
								f.CompactionState = manifest.CompactionStateCompacting
							})
						}
					}
				}

				var b strings.Builder
				fmt.Fprintf(&b, "Initial state before pick:\n%s", runVersionFileSizes(vers))
				pc := pickerByScore.pickAuto(compactionEnv{
					earliestUnflushedSeqNum: InternalKeySeqNumMax,
					inProgressCompactions:   inProgress,
				})
				if pc != nil {
					fmt.Fprintf(&b, "Picked: L%d->L%d: %0.1f\n", pc.startLevel.level, pc.outputLevel.level, pc.score)
				}
				if pc == nil {
					fmt.Fprintln(&b, "Picked: no compaction")
				}
				return b.String()
			case "pick_manual":
				var startLevel int
				var start, end string
				d.MaybeScanArgs(t, "level", &startLevel)
				d.MaybeScanArgs(t, "start", &start)
				d.MaybeScanArgs(t, "end", &end)

				iStart := base.MakeInternalKey([]byte(start), InternalKeySeqNumMax, InternalKeyKindMax)
				iEnd := base.MakeInternalKey([]byte(end), 0, 0)
				manual := &manualCompaction{
					done:  make(chan error, 1),
					level: startLevel,
					start: iStart.UserKey,
					end:   iEnd.UserKey,
				}

				pc, retryLater := pickManualCompaction(
					pickerByScore.vers,
					opts,
					compactionEnv{
						earliestUnflushedSeqNum: InternalKeySeqNumMax,
					},
					pickerByScore.getBaseLevel(),
					manual)
				if pc == nil {
					return fmt.Sprintf("nil, retryLater = %v", retryLater)
				}

				return fmt.Sprintf("L%d->L%d, retryLater = %v", pc.startLevel.level, pc.outputLevel.level, retryLater)
			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
}

func TestCompactionPickerEstimatedCompactionDebt(t *testing.T) {
	datadriven.RunTest(t, "testdata/compaction_picker_estimated_debt",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "init":
				vers, opts, errMsg := loadVersion(t, d)
				if errMsg != "" {
					return errMsg
				}
				opts.MemTableSize = 1000

				p := newCompactionPicker(vers, opts, nil)
				return fmt.Sprintf("%d\n", p.estimatedCompactionDebt(0))

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
}

func TestCompactionPickerL0(t *testing.T) {
	opts := (*Options)(nil).EnsureDefaults()
	opts.Experimental.L0CompactionConcurrency = 1

	parseMeta := func(s string) (*fileMetadata, error) {
		parts := strings.Split(s, ":")
		fileNum, err := strconv.Atoi(parts[0])
		if err != nil {
			return nil, err
		}
		fields := strings.Fields(parts[1])
		parts = strings.Split(fields[0], "-")
		if len(parts) != 2 {
			return nil, errors.Errorf("malformed table spec: %s", s)
		}
		m := (&fileMetadata{
			FileNum: base.FileNum(fileNum),
		}).ExtendPointKeyBounds(
			opts.Comparer.Compare,
			base.ParseInternalKey(strings.TrimSpace(parts[0])),
			base.ParseInternalKey(strings.TrimSpace(parts[1])),
		)
		m.SmallestSeqNum = m.Smallest.SeqNum()
		m.LargestSeqNum = m.Largest.SeqNum()
		m.InitPhysicalBacking()
		return m, nil
	}

	var picker *compactionPickerByScore
	var inProgressCompactions []compactionInfo
	var pc *pickedCompaction

	datadriven.RunTest(t, "testdata/compaction_picker_L0", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "define":
			fileMetas := [manifest.NumLevels][]*fileMetadata{}
			baseLevel := manifest.NumLevels - 1
			level := 0
			var err error
			lines := strings.Split(td.Input, "\n")
			var compactionLines []string

			for len(lines) > 0 {
				data := strings.TrimSpace(lines[0])
				lines = lines[1:]
				switch data {
				case "L0", "L1", "L2", "L3", "L4", "L5", "L6":
					level, err = strconv.Atoi(data[1:])
					if err != nil {
						return err.Error()
					}
				case "compactions":
					compactionLines, lines = lines, nil
				default:
					meta, err := parseMeta(data)
					if err != nil {
						return err.Error()
					}
					if level != 0 && level < baseLevel {
						baseLevel = level
					}
					fileMetas[level] = append(fileMetas[level], meta)
				}
			}

			// Parse in-progress compactions in the form of:
			//   L0 000001 -> L2 000005
			inProgressCompactions = nil
			for len(compactionLines) > 0 {
				parts := strings.Fields(compactionLines[0])
				compactionLines = compactionLines[1:]

				var level int
				var info compactionInfo
				first := true
				compactionFiles := map[int][]*fileMetadata{}
				for _, p := range parts {
					switch p {
					case "L0", "L1", "L2", "L3", "L4", "L5", "L6":
						var err error
						level, err = strconv.Atoi(p[1:])
						if err != nil {
							return err.Error()
						}
						if len(info.inputs) > 0 && info.inputs[len(info.inputs)-1].level == level {
							// eg, L0 -> L0 compaction or L6 -> L6 compaction
							continue
						}
						if info.outputLevel < level {
							info.outputLevel = level
						}
						info.inputs = append(info.inputs, compactionLevel{level: level})
					case "->":
						continue
					default:
						fileNum, err := strconv.Atoi(p)
						if err != nil {
							return err.Error()
						}
						var compactFile *fileMetadata
						for _, m := range fileMetas[level] {
							if m.FileNum == FileNum(fileNum) {
								compactFile = m
							}
						}
						if compactFile == nil {
							return fmt.Sprintf("cannot find compaction file %s", FileNum(fileNum))
						}
						compactFile.CompactionState = manifest.CompactionStateCompacting
						if first || base.InternalCompare(DefaultComparer.Compare, info.largest, compactFile.Largest) < 0 {
							info.largest = compactFile.Largest
						}
						if first || base.InternalCompare(DefaultComparer.Compare, info.smallest, compactFile.Smallest) > 0 {
							info.smallest = compactFile.Smallest
						}
						first = false
						compactionFiles[level] = append(compactionFiles[level], compactFile)
					}
				}
				for i, cl := range info.inputs {
					files := compactionFiles[cl.level]
					info.inputs[i].files = manifest.NewLevelSliceSeqSorted(files)
					// Mark as intra-L0 compacting if the compaction is
					// L0 -> L0.
					if info.outputLevel == 0 {
						for _, f := range files {
							f.IsIntraL0Compacting = true
						}
					}
				}
				inProgressCompactions = append(inProgressCompactions, info)
			}

			version := newVersion(opts, fileMetas)
			version.L0Sublevels.InitCompactingFileInfo(inProgressL0Compactions(inProgressCompactions))
			vs := &versionSet{
				opts:    opts,
				cmp:     DefaultComparer.Compare,
				cmpName: DefaultComparer.Name,
			}
			vs.versions.Init(nil)
			vs.append(version)
			picker = &compactionPickerByScore{
				opts:      opts,
				vers:      version,
				baseLevel: baseLevel,
			}
			vs.picker = picker
			picker.initLevelMaxBytes(inProgressCompactions)

			var buf bytes.Buffer
			fmt.Fprint(&buf, version.String())
			if len(inProgressCompactions) > 0 {
				fmt.Fprintln(&buf, "compactions")
				for _, c := range inProgressCompactions {
					fmt.Fprintf(&buf, "  %s\n", c.String())
				}
			}
			return buf.String()
		case "pick-auto":
			td.MaybeScanArgs(t, "l0_compaction_threshold", &opts.L0CompactionThreshold)
			td.MaybeScanArgs(t, "l0_compaction_file_threshold", &opts.L0CompactionFileThreshold)

			pc = picker.pickAuto(compactionEnv{
				diskAvailBytes:          math.MaxUint64,
				earliestUnflushedSeqNum: math.MaxUint64,
				inProgressCompactions:   inProgressCompactions,
			})
			var result strings.Builder
			if pc != nil {
				checkClone(t, pc)
				c := newCompaction(pc, opts, time.Now(), nil /* provider */)
				fmt.Fprintf(&result, "L%d -> L%d\n", pc.startLevel.level, pc.outputLevel.level)
				fmt.Fprintf(&result, "L%d: %s\n", pc.startLevel.level, fileNums(pc.startLevel.files))
				if !pc.outputLevel.files.Empty() {
					fmt.Fprintf(&result, "L%d: %s\n", pc.outputLevel.level, fileNums(pc.outputLevel.files))
				}
				if !c.grandparents.Empty() {
					fmt.Fprintf(&result, "grandparents: %s\n", fileNums(c.grandparents))
				}
			} else {
				return "nil"
			}
			return result.String()
		case "mark-for-compaction":
			var fileNum uint64
			td.ScanArgs(t, "file", &fileNum)
			for l, lm := range picker.vers.Levels {
				iter := lm.Iter()
				for f := iter.First(); f != nil; f = iter.Next() {
					if f.FileNum != base.FileNum(fileNum) {
						continue
					}
					f.MarkedForCompaction = true
					picker.vers.Stats.MarkedForCompaction++
					picker.vers.Levels[l].InvalidateAnnotation(markedForCompactionAnnotator{})
					return fmt.Sprintf("marked L%d.%s", l, f.FileNum)
				}
			}
			return "not-found"
		case "max-output-file-size":
			if pc == nil {
				return "no compaction"
			}
			return fmt.Sprintf("%d", pc.maxOutputFileSize)
		case "max-overlap-bytes":
			if pc == nil {
				return "no compaction"
			}
			return fmt.Sprintf("%d", pc.maxOverlapBytes)
		}
		return fmt.Sprintf("unrecognized command: %s", td.Cmd)
	})
}

func TestCompactionPickerConcurrency(t *testing.T) {
	opts := (*Options)(nil).EnsureDefaults()
	opts.Experimental.L0CompactionConcurrency = 1

	parseMeta := func(s string) (*fileMetadata, error) {
		parts := strings.Split(s, ":")
		fileNum, err := strconv.Atoi(parts[0])
		if err != nil {
			return nil, err
		}
		fields := strings.Fields(parts[1])
		parts = strings.Split(fields[0], "-")
		if len(parts) != 2 {
			return nil, errors.Errorf("malformed table spec: %s", s)
		}
		m := (&fileMetadata{
			FileNum: base.FileNum(fileNum),
			Size:    1028,
		}).ExtendPointKeyBounds(
			opts.Comparer.Compare,
			base.ParseInternalKey(strings.TrimSpace(parts[0])),
			base.ParseInternalKey(strings.TrimSpace(parts[1])),
		)
		m.InitPhysicalBacking()
		for _, p := range fields[1:] {
			if strings.HasPrefix(p, "size=") {
				v, err := strconv.Atoi(strings.TrimPrefix(p, "size="))
				if err != nil {
					return nil, err
				}
				m.Size = uint64(v)
			}
		}
		m.SmallestSeqNum = m.Smallest.SeqNum()
		m.LargestSeqNum = m.Largest.SeqNum()
		return m, nil
	}

	var picker *compactionPickerByScore
	var inProgressCompactions []compactionInfo

	datadriven.RunTest(t, "testdata/compaction_picker_concurrency", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "define":
			fileMetas := [manifest.NumLevels][]*fileMetadata{}
			level := 0
			var err error
			lines := strings.Split(td.Input, "\n")
			var compactionLines []string

			for len(lines) > 0 {
				data := strings.TrimSpace(lines[0])
				lines = lines[1:]
				switch data {
				case "L0", "L1", "L2", "L3", "L4", "L5", "L6":
					level, err = strconv.Atoi(data[1:])
					if err != nil {
						return err.Error()
					}
				case "compactions":
					compactionLines, lines = lines, nil
				default:
					meta, err := parseMeta(data)
					if err != nil {
						return err.Error()
					}
					fileMetas[level] = append(fileMetas[level], meta)
				}
			}

			// Parse in-progress compactions in the form of:
			//   L0 000001 -> L2 000005
			inProgressCompactions = nil
			for len(compactionLines) > 0 {
				parts := strings.Fields(compactionLines[0])
				compactionLines = compactionLines[1:]

				var level int
				var info compactionInfo
				first := true
				compactionFiles := map[int][]*fileMetadata{}
				for _, p := range parts {
					switch p {
					case "L0", "L1", "L2", "L3", "L4", "L5", "L6":
						var err error
						level, err = strconv.Atoi(p[1:])
						if err != nil {
							return err.Error()
						}
						if len(info.inputs) > 0 && info.inputs[len(info.inputs)-1].level == level {
							// eg, L0 -> L0 compaction or L6 -> L6 compaction
							continue
						}
						if info.outputLevel < level {
							info.outputLevel = level
						}
						info.inputs = append(info.inputs, compactionLevel{level: level})
					case "->":
						continue
					default:
						fileNum, err := strconv.Atoi(p)
						if err != nil {
							return err.Error()
						}
						var compactFile *fileMetadata
						for _, m := range fileMetas[level] {
							if m.FileNum == FileNum(fileNum) {
								compactFile = m
							}
						}
						if compactFile == nil {
							return fmt.Sprintf("cannot find compaction file %s", FileNum(fileNum))
						}
						compactFile.CompactionState = manifest.CompactionStateCompacting
						if first || base.InternalCompare(DefaultComparer.Compare, info.largest, compactFile.Largest) < 0 {
							info.largest = compactFile.Largest
						}
						if first || base.InternalCompare(DefaultComparer.Compare, info.smallest, compactFile.Smallest) > 0 {
							info.smallest = compactFile.Smallest
						}
						first = false
						compactionFiles[level] = append(compactionFiles[level], compactFile)
					}
				}
				for i, cl := range info.inputs {
					files := compactionFiles[cl.level]
					if cl.level == 0 {
						info.inputs[i].files = manifest.NewLevelSliceSeqSorted(files)
					} else {
						info.inputs[i].files = manifest.NewLevelSliceKeySorted(DefaultComparer.Compare, files)
					}
					// Mark as intra-L0 compacting if the compaction is
					// L0 -> L0.
					if info.outputLevel == 0 {
						for _, f := range files {
							f.IsIntraL0Compacting = true
						}
					}
				}
				inProgressCompactions = append(inProgressCompactions, info)
			}

			version := newVersion(opts, fileMetas)
			version.L0Sublevels.InitCompactingFileInfo(inProgressL0Compactions(inProgressCompactions))
			vs := &versionSet{
				opts:    opts,
				cmp:     DefaultComparer.Compare,
				cmpName: DefaultComparer.Name,
			}
			vs.versions.Init(nil)
			vs.append(version)

			picker = newCompactionPicker(version, opts, inProgressCompactions).(*compactionPickerByScore)
			vs.picker = picker

			var buf bytes.Buffer
			fmt.Fprint(&buf, version.String())
			if len(inProgressCompactions) > 0 {
				fmt.Fprintln(&buf, "compactions")
				for _, c := range inProgressCompactions {
					fmt.Fprintf(&buf, "  %s\n", c.String())
				}
			}
			return buf.String()

		case "pick-auto":
			td.MaybeScanArgs(t, "l0_compaction_threshold", &opts.L0CompactionThreshold)
			td.MaybeScanArgs(t, "l0_compaction_concurrency", &opts.Experimental.L0CompactionConcurrency)
			td.MaybeScanArgs(t, "compaction_debt_concurrency", &opts.Experimental.CompactionDebtConcurrency)

			pc := picker.pickAuto(compactionEnv{
				earliestUnflushedSeqNum: math.MaxUint64,
				inProgressCompactions:   inProgressCompactions,
			})
			var result strings.Builder
			if pc != nil {
				c := newCompaction(pc, opts, time.Now(), nil /* provider */)
				fmt.Fprintf(&result, "L%d -> L%d\n", pc.startLevel.level, pc.outputLevel.level)
				fmt.Fprintf(&result, "L%d: %s\n", pc.startLevel.level, fileNums(pc.startLevel.files))
				if !pc.outputLevel.files.Empty() {
					fmt.Fprintf(&result, "L%d: %s\n", pc.outputLevel.level, fileNums(pc.outputLevel.files))
				}
				if !c.grandparents.Empty() {
					fmt.Fprintf(&result, "grandparents: %s\n", fileNums(c.grandparents))
				}
			} else {
				return "nil"
			}
			return result.String()
		}
		return fmt.Sprintf("unrecognized command: %s", td.Cmd)
	})
}

func TestCompactionPickerPickReadTriggered(t *testing.T) {
	opts := (*Options)(nil).EnsureDefaults()
	var picker *compactionPickerByScore
	var rcList readCompactionQueue
	var vers *version

	parseMeta := func(s string) (*fileMetadata, error) {
		parts := strings.Split(s, ":")
		fileNum, err := strconv.Atoi(parts[0])
		if err != nil {
			return nil, err
		}
		fields := strings.Fields(parts[1])
		parts = strings.Split(fields[0], "-")
		if len(parts) != 2 {
			return nil, errors.Errorf("malformed table spec: %s. usage: <file-num>:start.SET.1-end.SET.2", s)
		}
		m := (&fileMetadata{
			FileNum: base.FileNum(fileNum),
			Size:    1028,
		}).ExtendPointKeyBounds(
			opts.Comparer.Compare,
			base.ParseInternalKey(strings.TrimSpace(parts[0])),
			base.ParseInternalKey(strings.TrimSpace(parts[1])),
		)
		m.InitPhysicalBacking()
		for _, p := range fields[1:] {
			if strings.HasPrefix(p, "size=") {
				v, err := strconv.Atoi(strings.TrimPrefix(p, "size="))
				if err != nil {
					return nil, err
				}
				m.Size = uint64(v)
			}
		}
		m.SmallestSeqNum = m.Smallest.SeqNum()
		m.LargestSeqNum = m.Largest.SeqNum()
		return m, nil
	}

	datadriven.RunTest(t, "testdata/compaction_picker_read_triggered", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "define":
			rcList = readCompactionQueue{}
			fileMetas := [manifest.NumLevels][]*fileMetadata{}
			level := 0
			var err error
			lines := strings.Split(td.Input, "\n")

			for len(lines) > 0 {
				data := strings.TrimSpace(lines[0])
				lines = lines[1:]
				switch data {
				case "L0", "L1", "L2", "L3", "L4", "L5", "L6":
					level, err = strconv.Atoi(data[1:])
					if err != nil {
						return err.Error()
					}
				default:
					meta, err := parseMeta(data)
					if err != nil {
						return err.Error()
					}
					fileMetas[level] = append(fileMetas[level], meta)
				}
			}

			vers = newVersion(opts, fileMetas)
			vs := &versionSet{
				opts:    opts,
				cmp:     DefaultComparer.Compare,
				cmpName: DefaultComparer.Name,
			}
			vs.versions.Init(nil)
			vs.append(vers)
			var inProgressCompactions []compactionInfo
			picker = newCompactionPicker(vers, opts, inProgressCompactions).(*compactionPickerByScore)
			vs.picker = picker

			var buf bytes.Buffer
			fmt.Fprint(&buf, vers.String())
			return buf.String()

		case "add-read-compaction":
			for _, line := range strings.Split(td.Input, "\n") {
				if line == "" {
					continue
				}
				parts := strings.Split(line, " ")
				if len(parts) != 3 {
					return "error: malformed data for add-read-compaction. usage: <level>: <start>-<end> <filenum>"
				}
				if l, err := strconv.Atoi(parts[0][:1]); err == nil {
					keys := strings.Split(parts[1], "-")
					fileNum, _ := strconv.Atoi(parts[2])

					rc := readCompaction{
						level:   l,
						start:   []byte(keys[0]),
						end:     []byte(keys[1]),
						fileNum: base.FileNum(fileNum),
					}
					rcList.add(&rc, DefaultComparer.Compare)
				} else {
					return err.Error()
				}
			}
			return ""

		case "show-read-compactions":
			var sb strings.Builder
			if rcList.size == 0 {
				sb.WriteString("(none)")
			}
			for i := 0; i < rcList.size; i++ {
				rc := rcList.at(i)
				sb.WriteString(fmt.Sprintf("(level: %d, start: %s, end: %s)\n", rc.level, string(rc.start), string(rc.end)))
			}
			return sb.String()

		case "pick-auto":
			pc := picker.pickAuto(compactionEnv{
				earliestUnflushedSeqNum: math.MaxUint64,
				readCompactionEnv: readCompactionEnv{
					readCompactions: &rcList,
					flushing:        false,
				},
			})
			var result strings.Builder
			if pc != nil {
				fmt.Fprintf(&result, "L%d -> L%d\n", pc.startLevel.level, pc.outputLevel.level)
				fmt.Fprintf(&result, "L%d: %s\n", pc.startLevel.level, fileNums(pc.startLevel.files))
				if !pc.outputLevel.files.Empty() {
					fmt.Fprintf(&result, "L%d: %s\n", pc.outputLevel.level, fileNums(pc.outputLevel.files))
				}
			} else {
				return "nil"
			}
			return result.String()
		}
		return fmt.Sprintf("unrecognized command: %s", td.Cmd)
	})
}

type alwaysMultiLevel struct{}

func (d alwaysMultiLevel) pick(
	pcOrig *pickedCompaction, opts *Options, diskAvailBytes uint64,
) *pickedCompaction {
	pcMulti := pcOrig.clone()
	if !pcMulti.setupMultiLevelCandidate(opts, diskAvailBytes) {
		return pcOrig
	}
	return pcMulti
}

func (d alwaysMultiLevel) allowL0() bool {
	return false
}

func TestPickedCompactionSetupInputs(t *testing.T) {
	opts := &Options{}
	opts.EnsureDefaults()

	parseMeta := func(s string) *fileMetadata {
		parts := strings.Split(strings.TrimSpace(s), " ")
		var fileSize uint64
		var compacting bool
		for _, part := range parts {
			switch {
			case part == "compacting":
				compacting = true
			case strings.HasPrefix(part, "size="):
				v, err := strconv.ParseUint(strings.TrimPrefix(part, "size="), 10, 64)
				require.NoError(t, err)
				fileSize = v
			}
		}
		tableParts := strings.Split(parts[0], "-")
		if len(tableParts) != 2 {
			t.Fatalf("malformed table spec: %s", s)
		}
		state := manifest.CompactionStateNotCompacting
		if compacting {
			state = manifest.CompactionStateCompacting
		}
		m := (&fileMetadata{
			CompactionState: state,
			Size:            fileSize,
		}).ExtendPointKeyBounds(
			opts.Comparer.Compare,
			base.ParseInternalKey(strings.TrimSpace(tableParts[0])),
			base.ParseInternalKey(strings.TrimSpace(tableParts[1])),
		)
		m.SmallestSeqNum = m.Smallest.SeqNum()
		m.LargestSeqNum = m.Largest.SeqNum()
		m.InitPhysicalBacking()
		return m
	}

	setupInputTest := func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "setup-inputs":
			var availBytes uint64 = math.MaxUint64
			var maxLevelBytes [7]int64
			args := d.CmdArgs

			if len(args) > 0 && args[0].Key == "avail-bytes" {
				require.Equal(t, 1, len(args[0].Vals))
				var err error
				availBytes, err = strconv.ParseUint(args[0].Vals[0], 10, 64)
				require.NoError(t, err)
				args = args[1:]
			}

			if len(args) != 2 {
				return "setup-inputs [avail-bytes=XXX] <start> <end>"
			}

			pc := &pickedCompaction{
				cmp:    DefaultComparer.Compare,
				inputs: []compactionLevel{{level: -1}, {level: -1}},
			}
			pc.startLevel, pc.outputLevel = &pc.inputs[0], &pc.inputs[1]
			var currentLevel int
			var files [numLevels][]*fileMetadata
			fileNum := FileNum(1)

			for _, data := range strings.Split(d.Input, "\n") {
				switch data[:2] {
				case "L0", "L1", "L2", "L3", "L4", "L5", "L6":
					levelArgs := strings.Fields(data)
					level, err := strconv.Atoi(levelArgs[0][1:])
					if err != nil {
						return err.Error()
					}
					currentLevel = level
					if len(levelArgs) > 1 {
						maxSizeArg := strings.Replace(levelArgs[1], "max-size=", "", 1)
						maxSize, err := strconv.ParseInt(maxSizeArg, 10, 64)
						if err != nil {
							return err.Error()
						}
						maxLevelBytes[level] = maxSize
					} else {
						maxLevelBytes[level] = math.MaxInt64
					}
					if pc.startLevel.level == -1 {
						pc.startLevel.level = level

					} else if pc.outputLevel.level == -1 {
						if pc.startLevel.level >= level {
							return fmt.Sprintf("startLevel=%d >= outputLevel=%d\n", pc.startLevel.level, level)
						}
						pc.outputLevel.level = level
					}
				default:
					meta := parseMeta(data)
					meta.FileNum = fileNum
					fileNum++
					files[currentLevel] = append(files[currentLevel], meta)
				}
			}

			if pc.outputLevel.level == -1 {
				pc.outputLevel.level = pc.startLevel.level + 1
			}
			pc.version = newVersion(opts, files)
			pc.startLevel.files = pc.version.Overlaps(pc.startLevel.level, pc.cmp,
				[]byte(args[0].String()), []byte(args[1].String()), false /* exclusiveEnd */)

			var isCompacting bool
			if !pc.setupInputs(opts, availBytes, pc.startLevel) {
				isCompacting = true
			}
			origPC := pc
			pc = pc.maybeAddLevel(opts, availBytes)
			// If pc points to a new pickedCompaction, a new multi level compaction
			// was initialized.
			initMultiLevel := pc != origPC
			checkClone(t, pc)
			var buf bytes.Buffer
			for _, cl := range pc.inputs {
				if cl.files.Empty() {
					continue
				}

				fmt.Fprintf(&buf, "L%d\n", cl.level)
				cl.files.Each(func(f *fileMetadata) {
					fmt.Fprintf(&buf, "  %s\n", f)
				})
			}
			if isCompacting {
				fmt.Fprintf(&buf, "is-compacting\n")
			}

			if initMultiLevel {
				extraLevel := pc.extraLevels[0].level
				fmt.Fprintf(&buf, "init-multi-level(%d,%d,%d)\n", pc.startLevel.level, extraLevel,
					pc.outputLevel.level)
				fmt.Fprintf(&buf, "Original WriteAmp %.2f; ML WriteAmp %.2f\n", origPC.predictedWriteAmp(), pc.predictedWriteAmp())
				fmt.Fprintf(&buf, "Original OverlappingRatio %.2f; ML OverlappingRatio %.2f\n", origPC.overlappingRatio(), pc.overlappingRatio())
			}
			return buf.String()

		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	}

	t.Logf("Test basic setup inputs behavior without multi level compactions")
	opts.Experimental.MultiLevelCompactionHeuristic = NoMultiLevel{}
	datadriven.RunTest(t, "testdata/compaction_setup_inputs",
		setupInputTest)

	t.Logf("Turning multi level compaction on")
	opts.Experimental.MultiLevelCompactionHeuristic = alwaysMultiLevel{}
	datadriven.RunTest(t, "testdata/compaction_setup_inputs_multilevel_dummy",
		setupInputTest)

	t.Logf("Try Write-Amp Heuristic")
	opts.Experimental.MultiLevelCompactionHeuristic = WriteAmpHeuristic{}
	datadriven.RunTest(t, "testdata/compaction_setup_inputs_multilevel_write_amp",
		setupInputTest)
}

func TestPickedCompactionExpandInputs(t *testing.T) {
	opts := &Options{}
	opts.EnsureDefaults()
	cmp := DefaultComparer.Compare
	var files []*fileMetadata

	parseMeta := func(s string) *fileMetadata {
		parts := strings.Split(s, "-")
		if len(parts) != 2 {
			t.Fatalf("malformed table spec: %s", s)
		}
		m := (&fileMetadata{}).ExtendPointKeyBounds(
			opts.Comparer.Compare,
			base.ParseInternalKey(parts[0]),
			base.ParseInternalKey(parts[1]),
		)
		m.InitPhysicalBacking()
		return m
	}

	datadriven.RunTest(t, "testdata/compaction_expand_inputs",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "define":
				files = nil
				if len(d.Input) == 0 {
					return ""
				}
				for _, data := range strings.Split(d.Input, "\n") {
					meta := parseMeta(data)
					meta.FileNum = FileNum(len(files))
					files = append(files, meta)
				}
				manifest.SortBySmallest(files, cmp)
				return ""

			case "expand-inputs":
				pc := &pickedCompaction{
					cmp:    cmp,
					inputs: []compactionLevel{{level: 1}},
				}
				pc.startLevel = &pc.inputs[0]

				var filesLevelled [numLevels][]*fileMetadata
				filesLevelled[pc.startLevel.level] = files
				pc.version = newVersion(opts, filesLevelled)

				if len(d.CmdArgs) != 1 {
					return fmt.Sprintf("%s expects 1 argument", d.Cmd)
				}
				index, err := strconv.ParseInt(d.CmdArgs[0].String(), 10, 64)
				if err != nil {
					return err.Error()
				}

				// Advance the iterator to position `index`.
				iter := pc.version.Levels[pc.startLevel.level].Iter()
				_ = iter.First()
				for i := int64(0); i < index; i++ {
					_ = iter.Next()
				}

				inputs, _ := expandToAtomicUnit(cmp, iter.Take().Slice(), true /* disableIsCompacting */)

				var buf bytes.Buffer
				inputs.Each(func(f *fileMetadata) {
					fmt.Fprintf(&buf, "%d: %s-%s\n", f.FileNum, f.Smallest, f.Largest)
				})
				return buf.String()

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
}

func TestCompactionOutputFileSize(t *testing.T) {
	opts := (*Options)(nil).EnsureDefaults()
	var picker *compactionPickerByScore
	var vers *version

	parseMeta := func(s string) (*fileMetadata, error) {
		parts := strings.Split(s, ":")
		fileNum, err := strconv.Atoi(parts[0])
		if err != nil {
			return nil, err
		}
		fields := strings.Fields(parts[1])
		parts = strings.Split(fields[0], "-")
		if len(parts) != 2 {
			return nil, errors.Errorf("malformed table spec: %s. usage: <file-num>:start.SET.1-end.SET.2", s)
		}
		m := (&fileMetadata{
			FileNum: base.FileNum(fileNum),
			Size:    1028,
		}).ExtendPointKeyBounds(
			opts.Comparer.Compare,
			base.ParseInternalKey(strings.TrimSpace(parts[0])),
			base.ParseInternalKey(strings.TrimSpace(parts[1])),
		)
		m.InitPhysicalBacking()
		for _, p := range fields[1:] {
			if strings.HasPrefix(p, "size=") {
				v, err := strconv.Atoi(strings.TrimPrefix(p, "size="))
				if err != nil {
					return nil, err
				}
				m.Size = uint64(v)
			}
			if strings.HasPrefix(p, "range-deletions-bytes-estimate=") {
				v, err := strconv.Atoi(strings.TrimPrefix(p, "range-deletions-bytes-estimate="))
				if err != nil {
					return nil, err
				}
				m.Stats.RangeDeletionsBytesEstimate = uint64(v)
				m.Stats.NumDeletions = 1 // At least one range del responsible for the deletion bytes.
				m.StatsMarkValid()
			}
		}
		m.SmallestSeqNum = m.Smallest.SeqNum()
		m.LargestSeqNum = m.Largest.SeqNum()
		return m, nil
	}

	datadriven.RunTest(t, "testdata/compaction_output_file_size", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "define":
			fileMetas := [manifest.NumLevels][]*fileMetadata{}
			level := 0
			var err error
			lines := strings.Split(td.Input, "\n")

			for len(lines) > 0 {
				data := strings.TrimSpace(lines[0])
				lines = lines[1:]
				switch data {
				case "L0", "L1", "L2", "L3", "L4", "L5", "L6":
					level, err = strconv.Atoi(data[1:])
					if err != nil {
						return err.Error()
					}
				default:
					meta, err := parseMeta(data)
					if err != nil {
						return err.Error()
					}
					fileMetas[level] = append(fileMetas[level], meta)
				}
			}

			vers = newVersion(opts, fileMetas)
			vs := &versionSet{
				opts:    opts,
				cmp:     DefaultComparer.Compare,
				cmpName: DefaultComparer.Name,
			}
			vs.versions.Init(nil)
			vs.append(vers)
			var inProgressCompactions []compactionInfo
			picker = newCompactionPicker(vers, opts, inProgressCompactions).(*compactionPickerByScore)
			vs.picker = picker

			var buf bytes.Buffer
			fmt.Fprint(&buf, vers.String())
			return buf.String()

		case "pick-auto":
			pc := picker.pickAuto(compactionEnv{
				earliestUnflushedSeqNum: math.MaxUint64,
				earliestSnapshotSeqNum:  math.MaxUint64,
			})
			var buf bytes.Buffer
			if pc != nil {
				fmt.Fprintf(&buf, "L%d -> L%d\n", pc.startLevel.level, pc.outputLevel.level)
				fmt.Fprintf(&buf, "L%d: %s\n", pc.startLevel.level, fileNums(pc.startLevel.files))
				fmt.Fprintf(&buf, "maxOutputFileSize: %d\n", pc.maxOutputFileSize)
			} else {
				return "nil"
			}
			return buf.String()

		default:
			return fmt.Sprintf("unrecognized command: %s", td.Cmd)
		}
	})
}

func TestCompactionPickerCompensatedSize(t *testing.T) {
	testCases := []struct {
		size                  uint64
		pointDelEstimateBytes uint64
		rangeDelEstimateBytes uint64
		wantBytes             uint64
	}{
		{
			size:                  100,
			pointDelEstimateBytes: 0,
			rangeDelEstimateBytes: 0,
			wantBytes:             100,
		},
		{
			size:                  100,
			pointDelEstimateBytes: 10,
			rangeDelEstimateBytes: 0,
			wantBytes:             100 + 10,
		},
		{
			size:                  100,
			pointDelEstimateBytes: 10,
			rangeDelEstimateBytes: 5,
			wantBytes:             100 + 10 + 5,
		},
	}

	for _, tc := range testCases {
		t.Run("", func(t *testing.T) {
			f := &fileMetadata{Size: tc.size}
			f.InitPhysicalBacking()
			f.Stats.PointDeletionsBytesEstimate = tc.pointDelEstimateBytes
			f.Stats.RangeDeletionsBytesEstimate = tc.rangeDelEstimateBytes
			gotBytes := compensatedSize(f)
			require.Equal(t, tc.wantBytes, gotBytes)
		})
	}
}

func TestCompactionPickerPickFile(t *testing.T) {
	fs := vfs.NewMem()
	opts := &Options{
		Comparer:           testkeys.Comparer,
		FormatMajorVersion: FormatNewest,
		FS:                 fs,
	}

	d, err := Open("", opts)
	require.NoError(t, err)
	defer func() {
		if d != nil {
			require.NoError(t, d.Close())
		}
	}()

	datadriven.RunTest(t, "testdata/compaction_picker_pick_file", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "define":
			require.NoError(t, d.Close())

			d, err = runDBDefineCmd(td, opts)
			if err != nil {
				return err.Error()
			}
			d.mu.Lock()
			s := d.mu.versions.currentVersion().String()
			d.mu.Unlock()
			return s

		case "file-sizes":
			return runTableFileSizesCmd(td, d)

		case "pick-file":
			s := strings.TrimPrefix(td.CmdArgs[0].String(), "L")
			level, err := strconv.Atoi(s)
			if err != nil {
				return fmt.Sprintf("unable to parse arg %q as level", td.CmdArgs[0].String())
			}
			if level == 0 {
				panic("L0 picking unimplemented")
			}
			d.mu.Lock()
			defer d.mu.Unlock()

			// Use maybeScheduleCompactionPicker to take care of all of the
			// initialization of the compaction-picking environment, but never
			// pick a compaction; just call pickFile using the user-provided
			// level.
			var lf manifest.LevelFile
			var ok bool
			d.maybeScheduleCompactionPicker(func(untypedPicker compactionPicker, env compactionEnv) *pickedCompaction {
				p := untypedPicker.(*compactionPickerByScore)
				lf, ok = pickCompactionSeedFile(p.vers, opts, level, level+1, env.earliestSnapshotSeqNum)
				return nil
			})
			if !ok {
				return "(none)"
			}
			return lf.FileMetadata.String()

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

type pausableCleaner struct {
	mu      sync.Mutex
	cond    sync.Cond
	paused  bool
	cleaner Cleaner
}

func (c *pausableCleaner) Clean(fs vfs.FS, fileType base.FileType, path string) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	for c.paused {
		c.cond.Wait()
	}
	return c.cleaner.Clean(fs, fileType, path)
}

func (c *pausableCleaner) pause() {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.paused = true
}

func (c *pausableCleaner) resume() {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.paused = false
	c.cond.Broadcast()
}

func TestCompactionPickerScores(t *testing.T) {
	fs := vfs.NewMem()
	cleaner := pausableCleaner{cleaner: DeleteCleaner{}}
	cleaner.cond.L = &cleaner.mu
	opts := &Options{
		Cleaner:                     &cleaner,
		Comparer:                    testkeys.Comparer,
		DisableAutomaticCompactions: true,
		FormatMajorVersion:          FormatNewest,
		FS:                          fs,
	}

	d, err := Open("", opts)
	require.NoError(t, err)
	defer func() {
		if d != nil {
			cleaner.resume()
			require.NoError(t, closeAllSnapshots(d))
			require.NoError(t, d.Close())
		}
	}()

	var buf bytes.Buffer
	datadriven.RunTest(t, "testdata/compaction_picker_scores", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "define":
			require.NoError(t, closeAllSnapshots(d))
			require.NoError(t, d.Close())

			if td.HasArg("pause-cleaning") {
				cleaner.pause()
			}

			d, err = runDBDefineCmd(td, opts)
			if err != nil {
				return err.Error()
			}
			d.mu.Lock()
			s := d.mu.versions.currentVersion().String()
			d.mu.Unlock()
			return s

		case "disable-table-stats":
			d.mu.Lock()
			d.opts.private.disableTableStats = true
			d.mu.Unlock()
			return ""

		case "enable-table-stats":
			d.mu.Lock()
			d.opts.private.disableTableStats = false
			d.maybeCollectTableStatsLocked()
			d.mu.Unlock()
			return ""

		case "resume-cleaning":
			cleaner.resume()
			return ""

		case "ingest":
			if err = runBuildCmd(td, d, d.opts.FS); err != nil {
				return err.Error()
			}
			if err = runIngestCmd(td, d, d.opts.FS); err != nil {
				return err.Error()
			}
			d.mu.Lock()
			s := d.mu.versions.currentVersion().String()
			d.mu.Unlock()
			return s

		case "lsm":
			return runLSMCmd(td, d)

		case "maybe-compact":
			buf.Reset()
			d.mu.Lock()
			d.opts.DisableAutomaticCompactions = false
			d.maybeScheduleCompaction()
			fmt.Fprintf(&buf, "%d compactions in progress:", d.mu.compact.compactingCount)
			for c := range d.mu.compact.inProgress {
				fmt.Fprintf(&buf, "\n%s", c)
			}
			d.opts.DisableAutomaticCompactions = true
			d.mu.Unlock()
			return buf.String()

		case "scores":
			waitFor := "completion"
			td.MaybeScanArgs(t, "wait-for-compaction", &waitFor)

			// Wait for any running compactions to complete before calculating
			// scores. Otherwise, the output of this command is
			// nondeterministic.
			switch waitFor {
			case "completion":
				d.mu.Lock()
				for d.mu.compact.compactingCount > 0 {
					d.mu.compact.cond.Wait()
				}
				d.mu.Unlock()
			case "version-edit":
				func() {
					for {
						d.mu.Lock()
						wait := len(d.mu.compact.inProgress) > 0
						for c := range d.mu.compact.inProgress {
							wait = wait && !c.versionEditApplied
						}
						d.mu.Unlock()
						if !wait {
							return
						}
						// d.mu.compact.cond isn't notified until the compaction
						// is removed from inProgress, so we need to just sleep
						// and check again soon.
						time.Sleep(10 * time.Millisecond)
					}
				}()
			default:
				panic(fmt.Sprintf("unrecognized `wait-for-compaction` value: %q", waitFor))
			}

			buf.Reset()
			fmt.Fprintf(&buf, "L       Size   Score\n")
			for l, lm := range d.Metrics().Levels {
				if l < numLevels-1 {
					fmt.Fprintf(&buf, "L%-3d\t%-7s%.1f\n", l, humanize.Bytes.Int64(lm.Size), lm.Score)
				} else {
					fmt.Fprintf(&buf, "L%-3d\t%-7s-\n", l, humanize.Bytes.Int64(lm.Size))
				}
			}
			return buf.String()

		case "wait-pending-table-stats":
			return runTableStatsCmd(td, d)

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func fileNums(files manifest.LevelSlice) string {
	var ss []string
	files.Each(func(f *fileMetadata) {
		ss = append(ss, f.FileNum.String())
	})
	sort.Strings(ss)
	return strings.Join(ss, ",")
}

func checkClone(t *testing.T, pc *pickedCompaction) {
	pcClone := pc.clone()
	require.Equal(t, pc.String(), pcClone.String())

	// ensure all input files are in new address
	for i := range pc.inputs {
		// Len could be zero if setup inputs rejected a level
		if pc.inputs[i].files.Len() > 0 {
			require.NotEqual(t, &pc.inputs[i], &pcClone.inputs[i])
		}
	}
	for i := range pc.startLevel.l0SublevelInfo {
		if pc.startLevel.l0SublevelInfo[i].Len() > 0 {
			require.NotEqual(t, &pc.startLevel.l0SublevelInfo[i], &pcClone.startLevel.l0SublevelInfo[i])
		}
	}
}
