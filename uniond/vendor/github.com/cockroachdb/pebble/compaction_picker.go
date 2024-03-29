// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"fmt"
	"math"
	"sort"
	"strings"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/humanize"
	"github.com/cockroachdb/pebble/internal/manifest"
)

// The minimum count for an intra-L0 compaction. This matches the RocksDB
// heuristic.
const minIntraL0Count = 4

type compactionEnv struct {
	// diskAvailBytes holds a statistic on the number of bytes available on
	// disk, as reported by the filesystem. It's used to be more restrictive in
	// expanding compactions if available disk space is limited.
	//
	// The cached value (d.diskAvailBytes) is updated whenever a file is deleted
	// and whenever a compaction or flush completes. Since file removal is the
	// primary means of reclaiming space, there is a rough bound on the
	// statistic's staleness when available bytes is growing. Compactions and
	// flushes are longer, slower operations and provide a much looser bound
	// when available bytes is decreasing.
	diskAvailBytes          uint64
	earliestUnflushedSeqNum uint64
	earliestSnapshotSeqNum  uint64
	inProgressCompactions   []compactionInfo
	readCompactionEnv       readCompactionEnv
}

type compactionPicker interface {
	getScores([]compactionInfo) [numLevels]float64
	getBaseLevel() int
	estimatedCompactionDebt(l0ExtraSize uint64) uint64
	pickAuto(env compactionEnv) (pc *pickedCompaction)
	pickElisionOnlyCompaction(env compactionEnv) (pc *pickedCompaction)
	pickRewriteCompaction(env compactionEnv) (pc *pickedCompaction)
	pickReadTriggeredCompaction(env compactionEnv) (pc *pickedCompaction)
	forceBaseLevel1()
}

// readCompactionEnv is used to hold data required to perform read compactions
type readCompactionEnv struct {
	rescheduleReadCompaction *bool
	readCompactions          *readCompactionQueue
	flushing                 bool
}

// Information about in-progress compactions provided to the compaction picker.
// These are used to constrain the new compactions that will be picked.
type compactionInfo struct {
	// versionEditApplied is true if this compaction's version edit has already
	// been committed. The compaction may still be in-progress deleting newly
	// obsolete files.
	versionEditApplied bool
	inputs             []compactionLevel
	outputLevel        int
	smallest           InternalKey
	largest            InternalKey
}

func (info compactionInfo) String() string {
	var buf bytes.Buffer
	var largest int
	for i, in := range info.inputs {
		if i > 0 {
			fmt.Fprintf(&buf, " -> ")
		}
		fmt.Fprintf(&buf, "L%d", in.level)
		in.files.Each(func(m *fileMetadata) {
			fmt.Fprintf(&buf, " %s", m.FileNum)
		})
		if largest < in.level {
			largest = in.level
		}
	}
	if largest != info.outputLevel || len(info.inputs) == 1 {
		fmt.Fprintf(&buf, " -> L%d", info.outputLevel)
	}
	return buf.String()
}

type sortCompactionLevelsByPriority []candidateLevelInfo

func (s sortCompactionLevelsByPriority) Len() int {
	return len(s)
}

// A level should be picked for compaction if the compensatedScoreRatio is >= the
// compactionScoreThreshold.
const compactionScoreThreshold = 1

// Less should return true if s[i] must be placed earlier than s[j] in the final
// sorted list. The candidateLevelInfo for the level placed earlier is more likely
// to be picked for a compaction.
func (s sortCompactionLevelsByPriority) Less(i, j int) bool {
	iShouldCompact := s[i].compensatedScoreRatio >= compactionScoreThreshold
	jShouldCompact := s[j].compensatedScoreRatio >= compactionScoreThreshold
	// Ordering is defined as decreasing on (shouldCompact, uncompensatedScoreRatio)
	// where shouldCompact is 1 for true and 0 for false.
	if iShouldCompact && !jShouldCompact {
		return true
	}
	if !iShouldCompact && jShouldCompact {
		return false
	}

	if s[i].uncompensatedScoreRatio != s[j].uncompensatedScoreRatio {
		return s[i].uncompensatedScoreRatio > s[j].uncompensatedScoreRatio
	}
	return s[i].level < s[j].level
}

func (s sortCompactionLevelsByPriority) Swap(i, j int) {
	s[i], s[j] = s[j], s[i]
}

// sublevelInfo is used to tag a LevelSlice for an L0 sublevel with the
// sublevel.
type sublevelInfo struct {
	manifest.LevelSlice
	sublevel manifest.Level
}

func (cl sublevelInfo) Clone() sublevelInfo {
	return sublevelInfo{
		sublevel:   cl.sublevel,
		LevelSlice: cl.LevelSlice.Reslice(func(start, end *manifest.LevelIterator) {}),
	}
}
func (cl sublevelInfo) String() string {
	return fmt.Sprintf(`Sublevel %s; Levels %s`, cl.sublevel, cl.LevelSlice)
}

// generateSublevelInfo will generate the level slices for each of the sublevels
// from the level slice for all of L0.
func generateSublevelInfo(cmp base.Compare, levelFiles manifest.LevelSlice) []sublevelInfo {
	sublevelMap := make(map[uint64][]*fileMetadata)
	it := levelFiles.Iter()
	for f := it.First(); f != nil; f = it.Next() {
		sublevelMap[uint64(f.SubLevel)] = append(sublevelMap[uint64(f.SubLevel)], f)
	}

	var sublevels []int
	for level := range sublevelMap {
		sublevels = append(sublevels, int(level))
	}
	sort.Ints(sublevels)

	var levelSlices []sublevelInfo
	for _, sublevel := range sublevels {
		metas := sublevelMap[uint64(sublevel)]
		levelSlices = append(
			levelSlices,
			sublevelInfo{
				manifest.NewLevelSliceKeySorted(cmp, metas),
				manifest.L0Sublevel(sublevel),
			},
		)
	}
	return levelSlices
}

// compactionPickerMetrics holds metrics related to the compaction picking process
type compactionPickerMetrics struct {
	// scores contains the compensatedScoreRatio from the candidateLevelInfo.
	scores                      []float64
	singleLevelOverlappingRatio float64
	multiLevelOverlappingRatio  float64
}

// pickedCompaction contains information about a compaction that has already
// been chosen, and is being constructed. Compaction construction info lives in
// this struct, and is copied over into the compaction struct when that's
// created.
type pickedCompaction struct {
	cmp Compare
	// score of the chosen compaction. This is the same as the
	// compensatedScoreRatio in the candidateLevelInfo.
	score float64
	// kind indicates the kind of compaction.
	kind compactionKind
	// startLevel is the level that is being compacted. Inputs from startLevel
	// and outputLevel will be merged to produce a set of outputLevel files.
	startLevel *compactionLevel
	// outputLevel is the level that files are being produced in. outputLevel is
	// equal to startLevel+1 except when:
	//    - if startLevel is 0, the output level equals compactionPicker.baseLevel().
	//    - in multilevel compaction, the output level is the lowest level involved in
	//      the compaction
	outputLevel *compactionLevel
	// extraLevels contain additional levels in between the input and output
	// levels that get compacted in multi level compactions
	extraLevels []*compactionLevel
	inputs      []compactionLevel
	// LBase at the time of compaction picking.
	baseLevel int
	// L0-specific compaction info. Set to a non-nil value for all compactions
	// where startLevel == 0 that were generated by L0Sublevels.
	lcf *manifest.L0CompactionFiles
	// maxOutputFileSize is the maximum size of an individual table created
	// during compaction.
	maxOutputFileSize uint64
	// maxOverlapBytes is the maximum number of bytes of overlap allowed for a
	// single output table with the tables in the grandparent level.
	maxOverlapBytes uint64
	// maxReadCompactionBytes is the maximum bytes a read compaction is allowed to
	// overlap in its output level with. If the overlap is greater than
	// maxReadCompaction bytes, then we don't proceed with the compaction.
	maxReadCompactionBytes uint64
	// The boundaries of the input data.
	smallest      InternalKey
	largest       InternalKey
	version       *version
	pickerMetrics compactionPickerMetrics
}

func defaultOutputLevel(startLevel, baseLevel int) int {
	outputLevel := startLevel + 1
	if startLevel == 0 {
		outputLevel = baseLevel
	}
	if outputLevel >= numLevels-1 {
		outputLevel = numLevels - 1
	}
	return outputLevel
}

func newPickedCompaction(
	opts *Options, cur *version, startLevel, outputLevel, baseLevel int,
) *pickedCompaction {
	if startLevel > 0 && startLevel < baseLevel {
		panic(fmt.Sprintf("invalid compaction: start level %d should not be empty (base level %d)",
			startLevel, baseLevel))
	}

	adjustedLevel := adjustedOutputLevel(outputLevel, baseLevel)
	pc := &pickedCompaction{
		cmp:                    opts.Comparer.Compare,
		version:                cur,
		baseLevel:              baseLevel,
		inputs:                 []compactionLevel{{level: startLevel}, {level: outputLevel}},
		maxOutputFileSize:      uint64(opts.Level(adjustedLevel).TargetFileSize),
		maxOverlapBytes:        maxGrandparentOverlapBytes(opts, adjustedLevel),
		maxReadCompactionBytes: maxReadCompactionBytes(opts, adjustedLevel),
	}
	pc.startLevel = &pc.inputs[0]
	pc.outputLevel = &pc.inputs[1]
	return pc
}

// adjustedOutputLevel is the output level used for the purpose of
// determining the target output file size, overlap bytes, and expanded
// bytes, taking into account the base level.
func adjustedOutputLevel(outputLevel int, baseLevel int) int {
	adjustedOutputLevel := outputLevel
	if adjustedOutputLevel > 0 {
		// Output level is in the range [baseLevel, numLevels]. For the purpose of
		// determining the target output file size, overlap bytes, and expanded
		// bytes, we want to adjust the range to [1,numLevels].
		adjustedOutputLevel = 1 + outputLevel - baseLevel
	}
	return adjustedOutputLevel
}

func newPickedCompactionFromL0(
	lcf *manifest.L0CompactionFiles, opts *Options, vers *version, baseLevel int, isBase bool,
) *pickedCompaction {
	outputLevel := baseLevel
	if !isBase {
		outputLevel = 0 // Intra L0
	}

	pc := newPickedCompaction(opts, vers, 0, outputLevel, baseLevel)
	pc.lcf = lcf
	pc.outputLevel.level = outputLevel

	// Manually build the compaction as opposed to calling
	// pickAutoHelper. This is because L0Sublevels has already added
	// any overlapping L0 SSTables that need to be added, and
	// because compactions built by L0SSTables do not necessarily
	// pick contiguous sequences of files in pc.version.Levels[0].
	files := make([]*manifest.FileMetadata, 0, len(lcf.Files))
	iter := vers.Levels[0].Iter()
	for f := iter.First(); f != nil; f = iter.Next() {
		if lcf.FilesIncluded[f.L0Index] {
			files = append(files, f)
		}
	}
	pc.startLevel.files = manifest.NewLevelSliceSeqSorted(files)
	return pc
}

func (pc *pickedCompaction) String() string {
	var builder strings.Builder
	builder.WriteString(fmt.Sprintf(`Score=%f, `, pc.score))
	builder.WriteString(fmt.Sprintf(`Kind=%s, `, pc.kind))
	builder.WriteString(fmt.Sprintf(`AdjustedOutputLevel=%d, `, adjustedOutputLevel(pc.outputLevel.level, pc.baseLevel)))
	builder.WriteString(fmt.Sprintf(`maxOutputFileSize=%d, `, pc.maxOutputFileSize))
	builder.WriteString(fmt.Sprintf(`maxReadCompactionBytes=%d, `, pc.maxReadCompactionBytes))
	builder.WriteString(fmt.Sprintf(`smallest=%s, `, pc.smallest))
	builder.WriteString(fmt.Sprintf(`largest=%s, `, pc.largest))
	builder.WriteString(fmt.Sprintf(`version=%s, `, pc.version))
	builder.WriteString(fmt.Sprintf(`inputs=%s, `, pc.inputs))
	builder.WriteString(fmt.Sprintf(`startlevel=%s, `, pc.startLevel))
	builder.WriteString(fmt.Sprintf(`outputLevel=%s, `, pc.outputLevel))
	builder.WriteString(fmt.Sprintf(`extraLevels=%s, `, pc.extraLevels))
	builder.WriteString(fmt.Sprintf(`l0SublevelInfo=%s, `, pc.startLevel.l0SublevelInfo))
	builder.WriteString(fmt.Sprintf(`lcf=%s`, pc.lcf))
	return builder.String()
}

// Clone creates a deep copy of the pickedCompaction
func (pc *pickedCompaction) clone() *pickedCompaction {

	// Quickly copy over fields that do not require special deep copy care, and
	// set all fields that will require a deep copy to nil.
	newPC := &pickedCompaction{
		cmp:                    pc.cmp,
		score:                  pc.score,
		kind:                   pc.kind,
		baseLevel:              pc.baseLevel,
		maxOutputFileSize:      pc.maxOutputFileSize,
		maxOverlapBytes:        pc.maxOverlapBytes,
		maxReadCompactionBytes: pc.maxReadCompactionBytes,
		smallest:               pc.smallest.Clone(),
		largest:                pc.largest.Clone(),

		// TODO(msbutler): properly clone picker metrics
		pickerMetrics: pc.pickerMetrics,

		// Both copies see the same manifest, therefore, it's ok for them to se
		// share the same pc. version.
		version: pc.version,
	}

	newPC.inputs = make([]compactionLevel, len(pc.inputs))
	newPC.extraLevels = make([]*compactionLevel, 0, len(pc.extraLevels))
	for i := range pc.inputs {
		newPC.inputs[i] = pc.inputs[i].Clone()
		if i == 0 {
			newPC.startLevel = &newPC.inputs[i]
		} else if i == len(pc.inputs)-1 {
			newPC.outputLevel = &newPC.inputs[i]
		} else {
			newPC.extraLevels = append(newPC.extraLevels, &newPC.inputs[i])
		}
	}

	if len(pc.startLevel.l0SublevelInfo) > 0 {
		newPC.startLevel.l0SublevelInfo = make([]sublevelInfo, len(pc.startLevel.l0SublevelInfo))
		for i := range pc.startLevel.l0SublevelInfo {
			newPC.startLevel.l0SublevelInfo[i] = pc.startLevel.l0SublevelInfo[i].Clone()
		}
	}
	if pc.lcf != nil {
		newPC.lcf = pc.lcf.Clone()
	}
	return newPC
}

// maybeExpandedBounds is a helper function for setupInputs which ensures the
// pickedCompaction's smallest and largest internal keys are updated iff
// the candidate keys expand the key span. This avoids a bug for multi-level
// compactions: during the second call to setupInputs, the picked compaction's
// smallest and largest keys should not decrease the key span.
func (pc *pickedCompaction) maybeExpandBounds(smallest InternalKey, largest InternalKey) {
	emptyKey := InternalKey{}
	if base.InternalCompare(pc.cmp, smallest, emptyKey) == 0 {
		if base.InternalCompare(pc.cmp, largest, emptyKey) != 0 {
			panic("either both candidate keys are empty or neither are empty")
		}
		return
	}
	if base.InternalCompare(pc.cmp, pc.smallest, emptyKey) == 0 {
		if base.InternalCompare(pc.cmp, pc.largest, emptyKey) != 0 {
			panic("either both pc keys are empty or neither are empty")
		}
		pc.smallest = smallest
		pc.largest = largest
		return
	}
	if base.InternalCompare(pc.cmp, pc.smallest, smallest) >= 0 {
		pc.smallest = smallest
	}
	if base.InternalCompare(pc.cmp, pc.largest, largest) <= 0 {
		pc.largest = largest
	}
}

// setupInputs returns true if a compaction has been set up. It returns false if
// a concurrent compaction is occurring on the start or output level files.
func (pc *pickedCompaction) setupInputs(
	opts *Options, diskAvailBytes uint64, startLevel *compactionLevel,
) bool {
	// maxExpandedBytes is the maximum size of an expanded compaction. If
	// growing a compaction results in a larger size, the original compaction
	// is used instead.
	maxExpandedBytes := expandedCompactionByteSizeLimit(
		opts, adjustedOutputLevel(pc.outputLevel.level, pc.baseLevel), diskAvailBytes,
	)

	// Expand the initial inputs to a clean cut.
	var isCompacting bool
	startLevel.files, isCompacting = expandToAtomicUnit(pc.cmp, startLevel.files, false /* disableIsCompacting */)
	if isCompacting {
		return false
	}
	pc.maybeExpandBounds(manifest.KeyRange(pc.cmp, startLevel.files.Iter()))

	// Determine the sstables in the output level which overlap with the input
	// sstables, and then expand those tables to a clean cut. No need to do
	// this for intra-L0 compactions; outputLevel.files is left empty for those.
	if startLevel.level != pc.outputLevel.level {
		pc.outputLevel.files = pc.version.Overlaps(pc.outputLevel.level, pc.cmp, pc.smallest.UserKey,
			pc.largest.UserKey, pc.largest.IsExclusiveSentinel())
		pc.outputLevel.files, isCompacting = expandToAtomicUnit(pc.cmp, pc.outputLevel.files,
			false /* disableIsCompacting */)
		if isCompacting {
			return false
		}
		pc.maybeExpandBounds(manifest.KeyRange(pc.cmp,
			startLevel.files.Iter(), pc.outputLevel.files.Iter()))
	}

	// Grow the sstables in startLevel.level as long as it doesn't affect the number
	// of sstables included from pc.outputLevel.level.
	if pc.lcf != nil && startLevel.level == 0 && pc.outputLevel.level != 0 {
		// Call the L0-specific compaction extension method. Similar logic as
		// pc.grow. Additional L0 files are optionally added to the compaction at
		// this step. Note that the bounds passed in are not the bounds of the
		// compaction, but rather the smallest and largest internal keys that
		// the compaction cannot include from L0 without pulling in more Lbase
		// files. Consider this example:
		//
		// L0:        c-d e+f g-h
		// Lbase: a-b     e+f     i-j
		//        a b c d e f g h i j
		//
		// The e-f files have already been chosen in the compaction. As pulling
		// in more LBase files is undesirable, the logic below will pass in
		// smallest = b and largest = i to ExtendL0ForBaseCompactionTo, which
		// will expand the compaction to include c-d and g-h from L0. The
		// bounds passed in are exclusive; the compaction cannot be expanded
		// to include files that "touch" it.
		smallestBaseKey := base.InvalidInternalKey
		largestBaseKey := base.InvalidInternalKey
		if pc.outputLevel.files.Empty() {
			baseIter := pc.version.Levels[pc.outputLevel.level].Iter()
			if sm := baseIter.SeekLT(pc.cmp, pc.smallest.UserKey); sm != nil {
				smallestBaseKey = sm.Largest
			}
			if la := baseIter.SeekGE(pc.cmp, pc.largest.UserKey); la != nil {
				largestBaseKey = la.Smallest
			}
		} else {
			// NB: We use Reslice to access the underlying level's files, but
			// we discard the returned slice. The pc.outputLevel.files slice
			// is not modified.
			_ = pc.outputLevel.files.Reslice(func(start, end *manifest.LevelIterator) {
				if sm := start.Prev(); sm != nil {
					smallestBaseKey = sm.Largest
				}
				if la := end.Next(); la != nil {
					largestBaseKey = la.Smallest
				}
			})
		}
		oldLcf := pc.lcf.Clone()
		if pc.version.L0Sublevels.ExtendL0ForBaseCompactionTo(smallestBaseKey, largestBaseKey, pc.lcf) {
			var newStartLevelFiles []*fileMetadata
			iter := pc.version.Levels[0].Iter()
			var sizeSum uint64
			for j, f := 0, iter.First(); f != nil; j, f = j+1, iter.Next() {
				if pc.lcf.FilesIncluded[f.L0Index] {
					newStartLevelFiles = append(newStartLevelFiles, f)
					sizeSum += f.Size
				}
			}
			if sizeSum+pc.outputLevel.files.SizeSum() < maxExpandedBytes {
				startLevel.files = manifest.NewLevelSliceSeqSorted(newStartLevelFiles)
				pc.smallest, pc.largest = manifest.KeyRange(pc.cmp,
					startLevel.files.Iter(), pc.outputLevel.files.Iter())
			} else {
				*pc.lcf = *oldLcf
			}
		}
	} else if pc.grow(pc.smallest, pc.largest, maxExpandedBytes, startLevel) {
		pc.maybeExpandBounds(manifest.KeyRange(pc.cmp,
			startLevel.files.Iter(), pc.outputLevel.files.Iter()))
	}

	if pc.startLevel.level == 0 {
		// We don't change the input files for the compaction beyond this point.
		pc.startLevel.l0SublevelInfo = generateSublevelInfo(pc.cmp, pc.startLevel.files)
	}

	return true
}

// grow grows the number of inputs at c.level without changing the number of
// c.level+1 files in the compaction, and returns whether the inputs grew. sm
// and la are the smallest and largest InternalKeys in all of the inputs.
func (pc *pickedCompaction) grow(
	sm, la InternalKey, maxExpandedBytes uint64, startLevel *compactionLevel,
) bool {
	if pc.outputLevel.files.Empty() {
		return false
	}
	grow0 := pc.version.Overlaps(startLevel.level, pc.cmp, sm.UserKey,
		la.UserKey, la.IsExclusiveSentinel())
	grow0, isCompacting := expandToAtomicUnit(pc.cmp, grow0, false /* disableIsCompacting */)
	if isCompacting {
		return false
	}
	if grow0.Len() <= startLevel.files.Len() {
		return false
	}
	if grow0.SizeSum()+pc.outputLevel.files.SizeSum() >= maxExpandedBytes {
		return false
	}
	// We need to include the outputLevel iter because without it, in a multiLevel scenario,
	// sm1 and la1 could shift the output level keyspace when pc.outputLevel.files is set to grow1.
	sm1, la1 := manifest.KeyRange(pc.cmp, grow0.Iter(), pc.outputLevel.files.Iter())
	grow1 := pc.version.Overlaps(pc.outputLevel.level, pc.cmp, sm1.UserKey,
		la1.UserKey, la1.IsExclusiveSentinel())
	grow1, isCompacting = expandToAtomicUnit(pc.cmp, grow1, false /* disableIsCompacting */)
	if isCompacting {
		return false
	}
	if grow1.Len() != pc.outputLevel.files.Len() {
		return false
	}
	startLevel.files = grow0
	pc.outputLevel.files = grow1
	return true
}

func (pc *pickedCompaction) compactionSize() uint64 {
	var bytesToCompact uint64
	for i := range pc.inputs {
		bytesToCompact += pc.inputs[i].files.SizeSum()
	}
	return bytesToCompact
}

// setupMultiLevelCandidated returns true if it successfully added another level
// to the compaction.
func (pc *pickedCompaction) setupMultiLevelCandidate(opts *Options, diskAvailBytes uint64) bool {
	pc.inputs = append(pc.inputs, compactionLevel{level: pc.outputLevel.level + 1})

	// Recalibrate startLevel and outputLevel:
	//  - startLevel and outputLevel pointers may be obsolete after appending to pc.inputs.
	//  - push outputLevel to extraLevels and move the new level to outputLevel
	pc.startLevel = &pc.inputs[0]
	pc.extraLevels = []*compactionLevel{&pc.inputs[1]}
	pc.outputLevel = &pc.inputs[2]
	return pc.setupInputs(opts, diskAvailBytes, pc.extraLevels[len(pc.extraLevels)-1])
}

// expandToAtomicUnit expands the provided level slice within its level both
// forwards and backwards to its "atomic compaction unit" boundaries, if
// necessary.
//
// While picking compaction inputs, this is required to maintain the invariant
// that the versions of keys at level+1 are older than the versions of keys at
// level. Tables are added to the right of the current slice tables such that
// the rightmost table has a "clean cut". A clean cut is either a change in
// user keys, or when the largest key in the left sstable is a range tombstone
// sentinel key (InternalKeyRangeDeleteSentinel).
//
// In addition to maintaining the seqnum invariant, expandToAtomicUnit is used
// to provide clean boundaries for range tombstone truncation during
// compaction. In order to achieve these clean boundaries, expandToAtomicUnit
// needs to find a "clean cut" on the left edge of the compaction as well.
// This is necessary in order for "atomic compaction units" to always be
// compacted as a unit. Failure to do this leads to a subtle bug with
// truncation of range tombstones to atomic compaction unit boundaries.
// Consider the scenario:
//
//	L3:
//	  12:[a#2,15-b#1,1]
//	  13:[b#0,15-d#72057594037927935,15]
//
// These sstables contain a range tombstone [a-d)#2 which spans the two
// sstables. The two sstables need to always be kept together. Compacting
// sstable 13 independently of sstable 12 would result in:
//
//	L3:
//	  12:[a#2,15-b#1,1]
//	L4:
//	  14:[b#0,15-d#72057594037927935,15]
//
// This state is still ok, but when sstable 12 is next compacted, its range
// tombstones will be truncated at "b" (the largest key in its atomic
// compaction unit). In the scenario here, that could result in b#1 becoming
// visible when it should be deleted.
//
// isCompacting is returned true for any atomic units that contain files that
// have in-progress compactions, i.e. FileMetadata.Compacting == true. If
// disableIsCompacting is true, isCompacting always returns false. This helps
// avoid spurious races from being detected when this method is used outside
// of compaction picking code.
//
// TODO(jackson): Compactions and flushes no longer split a user key between two
// sstables. We could perform a migration, re-compacting any sstables with split
// user keys, which would allow us to remove atomic compaction unit expansion
// code.
func expandToAtomicUnit(
	cmp Compare, inputs manifest.LevelSlice, disableIsCompacting bool,
) (slice manifest.LevelSlice, isCompacting bool) {
	// NB: Inputs for L0 can't be expanded and *version.Overlaps guarantees
	// that we get a 'clean cut.' For L0, Overlaps will return a slice without
	// access to the rest of the L0 files, so it's OK to try to reslice.
	if inputs.Empty() {
		// Nothing to expand.
		return inputs, false
	}

	// TODO(jackson): Update to avoid use of LevelIterator.Current(). The
	// Reslice interface will require some tweaking, because we currently rely
	// on Reslice having already positioned the LevelIterator appropriately.

	inputs = inputs.Reslice(func(start, end *manifest.LevelIterator) {
		iter := start.Clone()
		iter.Prev()
		for cur, prev := start.Current(), iter.Current(); prev != nil; cur, prev = start.Prev(), iter.Prev() {
			if cur.IsCompacting() {
				isCompacting = true
			}
			if cmp(prev.Largest.UserKey, cur.Smallest.UserKey) < 0 {
				break
			}
			if prev.Largest.IsExclusiveSentinel() {
				// The table prev has a largest key indicating that the user key
				// prev.largest.UserKey doesn't actually exist in the table.
				break
			}
			// prev.Largest.UserKey == cur.Smallest.UserKey, so we need to
			// include prev in the compaction.
		}

		iter = end.Clone()
		iter.Next()
		for cur, next := end.Current(), iter.Current(); next != nil; cur, next = end.Next(), iter.Next() {
			if cur.IsCompacting() {
				isCompacting = true
			}
			if cmp(cur.Largest.UserKey, next.Smallest.UserKey) < 0 {
				break
			}
			if cur.Largest.IsExclusiveSentinel() {
				// The table cur has a largest key indicating that the user key
				// cur.largest.UserKey doesn't actually exist in the table.
				break
			}
			// cur.Largest.UserKey == next.Smallest.UserKey, so we need to
			// include next in the compaction.
		}
	})
	inputIter := inputs.Iter()
	isCompacting = !disableIsCompacting &&
		(isCompacting || inputIter.First().IsCompacting() || inputIter.Last().IsCompacting())
	return inputs, isCompacting
}

func newCompactionPicker(
	v *version, opts *Options, inProgressCompactions []compactionInfo,
) compactionPicker {
	p := &compactionPickerByScore{
		opts: opts,
		vers: v,
	}
	p.initLevelMaxBytes(inProgressCompactions)
	return p
}

// Information about a candidate compaction level that has been identified by
// the compaction picker.
type candidateLevelInfo struct {
	// The compensatedScore of the level after adjusting according to the other
	// levels' sizes. For L0, the compensatedScoreRatio is equivalent to the
	// uncompensatedScoreRatio as we don't account for level size compensation in
	// L0.
	compensatedScoreRatio float64
	// The score of the level after accounting for level size compensation before
	// adjusting according to other levels' sizes. For L0, the compensatedScore
	// is equivalent to the uncompensatedScore as we don't account for level
	// size compensation in L0.
	compensatedScore float64
	// The score of the level to be compacted, calculated using uncompensated file
	// sizes and without any adjustments.
	uncompensatedScore float64
	// uncompensatedScoreRatio is the uncompensatedScore adjusted according to
	// the other levels' sizes.
	uncompensatedScoreRatio float64
	level                   int
	// The level to compact to.
	outputLevel int
	// The file in level that will be compacted. Additional files may be
	// picked by the compaction, and a pickedCompaction created for the
	// compaction.
	file manifest.LevelFile
}

func (c *candidateLevelInfo) shouldCompact() bool {
	return c.compensatedScoreRatio >= compactionScoreThreshold
}

func fileCompensation(f *fileMetadata) uint64 {
	return uint64(f.Stats.PointDeletionsBytesEstimate) + f.Stats.RangeDeletionsBytesEstimate
}

// compensatedSize returns f's file size, inflated according to compaction
// priorities.
func compensatedSize(f *fileMetadata) uint64 {
	// Add in the estimate of disk space that may be reclaimed by compacting the
	// file's tombstones.
	return f.Size + fileCompensation(f)
}

// compensatedSizeAnnotator implements manifest.Annotator, annotating B-Tree
// nodes with the sum of the files' compensated sizes. Its annotation type is
// a *uint64. Compensated sizes may change once a table's stats are loaded
// asynchronously, so its values are marked as cacheable only if a file's
// stats have been loaded.
type compensatedSizeAnnotator struct {
}

var _ manifest.Annotator = compensatedSizeAnnotator{}

func (a compensatedSizeAnnotator) Zero(dst interface{}) interface{} {
	if dst == nil {
		return new(uint64)
	}
	v := dst.(*uint64)
	*v = 0
	return v
}

func (a compensatedSizeAnnotator) Accumulate(
	f *fileMetadata, dst interface{},
) (v interface{}, cacheOK bool) {
	vptr := dst.(*uint64)
	*vptr = *vptr + compensatedSize(f)
	return vptr, f.StatsValid()
}

func (a compensatedSizeAnnotator) Merge(src interface{}, dst interface{}) interface{} {
	srcV := src.(*uint64)
	dstV := dst.(*uint64)
	*dstV = *dstV + *srcV
	return dstV
}

// totalCompensatedSize computes the compensated size over a file metadata
// iterator. Note that this function is linear in the files available to the
// iterator. Use the compensatedSizeAnnotator if querying the total
// compensated size of a level.
func totalCompensatedSize(iter manifest.LevelIterator) uint64 {
	var sz uint64
	for f := iter.First(); f != nil; f = iter.Next() {
		sz += compensatedSize(f)
	}
	return sz
}

// compactionPickerByScore holds the state and logic for picking a compaction. A
// compaction picker is associated with a single version. A new compaction
// picker is created and initialized every time a new version is installed.
type compactionPickerByScore struct {
	opts *Options
	vers *version
	// The level to target for L0 compactions. Levels L1 to baseLevel must be
	// empty.
	baseLevel int
	// levelMaxBytes holds the dynamically adjusted max bytes setting for each
	// level.
	levelMaxBytes [numLevels]int64
}

var _ compactionPicker = &compactionPickerByScore{}

func (p *compactionPickerByScore) getScores(inProgress []compactionInfo) [numLevels]float64 {
	var scores [numLevels]float64
	for _, info := range p.calculateLevelScores(inProgress) {
		scores[info.level] = info.compensatedScoreRatio
	}
	return scores
}

func (p *compactionPickerByScore) getBaseLevel() int {
	if p == nil {
		return 1
	}
	return p.baseLevel
}

// estimatedCompactionDebt estimates the number of bytes which need to be
// compacted before the LSM tree becomes stable.
func (p *compactionPickerByScore) estimatedCompactionDebt(l0ExtraSize uint64) uint64 {
	if p == nil {
		return 0
	}

	// We assume that all the bytes in L0 need to be compacted to Lbase. This is
	// unlike the RocksDB logic that figures out whether L0 needs compaction.
	bytesAddedToNextLevel := l0ExtraSize + p.vers.Levels[0].Size()
	lbaseSize := p.vers.Levels[p.baseLevel].Size()

	var compactionDebt uint64
	if bytesAddedToNextLevel > 0 && lbaseSize > 0 {
		// We only incur compaction debt if both L0 and Lbase contain data. If L0
		// is empty, no compaction is necessary. If Lbase is empty, a move-based
		// compaction from L0 would occur.
		compactionDebt += bytesAddedToNextLevel + lbaseSize
	}

	// loop invariant: At the beginning of the loop, bytesAddedToNextLevel is the
	// bytes added to `level` in the loop.
	for level := p.baseLevel; level < numLevels-1; level++ {
		levelSize := p.vers.Levels[level].Size() + bytesAddedToNextLevel
		nextLevelSize := p.vers.Levels[level+1].Size()
		if levelSize > uint64(p.levelMaxBytes[level]) {
			bytesAddedToNextLevel = levelSize - uint64(p.levelMaxBytes[level])
			if nextLevelSize > 0 {
				// We only incur compaction debt if the next level contains data. If the
				// next level is empty, a move-based compaction would be used.
				levelRatio := float64(nextLevelSize) / float64(levelSize)
				// The current level contributes bytesAddedToNextLevel to compactions.
				// The next level contributes levelRatio * bytesAddedToNextLevel.
				compactionDebt += uint64(float64(bytesAddedToNextLevel) * (levelRatio + 1))
			}
		} else {
			// We're not moving any bytes to the next level.
			bytesAddedToNextLevel = 0
		}
	}
	return compactionDebt
}

func (p *compactionPickerByScore) initLevelMaxBytes(inProgressCompactions []compactionInfo) {
	// The levelMaxBytes calculations here differ from RocksDB in two ways:
	//
	// 1. The use of dbSize vs maxLevelSize. RocksDB uses the size of the maximum
	//    level in L1-L6, rather than determining the size of the bottom level
	//    based on the total amount of data in the dB. The RocksDB calculation is
	//    problematic if L0 contains a significant fraction of data, or if the
	//    level sizes are roughly equal and thus there is a significant fraction
	//    of data outside of the largest level.
	//
	// 2. Not adjusting the size of Lbase based on L0. RocksDB computes
	//    baseBytesMax as the maximum of the configured LBaseMaxBytes and the
	//    size of L0. This is problematic because baseBytesMax is used to compute
	//    the max size of lower levels. A very large baseBytesMax will result in
	//    an overly large value for the size of lower levels which will caused
	//    those levels not to be compacted even when they should be
	//    compacted. This often results in "inverted" LSM shapes where Ln is
	//    larger than Ln+1.

	// Determine the first non-empty level and the total DB size.
	firstNonEmptyLevel := -1
	var dbSize uint64
	for level := 1; level < numLevels; level++ {
		if p.vers.Levels[level].Size() > 0 {
			if firstNonEmptyLevel == -1 {
				firstNonEmptyLevel = level
			}
			dbSize += p.vers.Levels[level].Size()
		}
	}
	for _, c := range inProgressCompactions {
		if c.outputLevel == 0 || c.outputLevel == -1 {
			continue
		}
		if c.inputs[0].level == 0 && (firstNonEmptyLevel == -1 || c.outputLevel < firstNonEmptyLevel) {
			firstNonEmptyLevel = c.outputLevel
		}
	}

	// Initialize the max-bytes setting for each level to "infinity" which will
	// disallow compaction for that level. We'll fill in the actual value below
	// for levels we want to allow compactions from.
	for level := 0; level < numLevels; level++ {
		p.levelMaxBytes[level] = math.MaxInt64
	}

	if dbSize == 0 {
		// No levels for L1 and up contain any data. Target L0 compactions for the
		// last level or to the level to which there is an ongoing L0 compaction.
		p.baseLevel = numLevels - 1
		if firstNonEmptyLevel >= 0 {
			p.baseLevel = firstNonEmptyLevel
		}
		return
	}

	dbSize += p.vers.Levels[0].Size()
	bottomLevelSize := dbSize - dbSize/uint64(p.opts.Experimental.LevelMultiplier)

	curLevelSize := bottomLevelSize
	for level := numLevels - 2; level >= firstNonEmptyLevel; level-- {
		curLevelSize = uint64(float64(curLevelSize) / float64(p.opts.Experimental.LevelMultiplier))
	}

	// Compute base level (where L0 data is compacted to).
	baseBytesMax := uint64(p.opts.LBaseMaxBytes)
	p.baseLevel = firstNonEmptyLevel
	for p.baseLevel > 1 && curLevelSize > baseBytesMax {
		p.baseLevel--
		curLevelSize = uint64(float64(curLevelSize) / float64(p.opts.Experimental.LevelMultiplier))
	}

	smoothedLevelMultiplier := 1.0
	if p.baseLevel < numLevels-1 {
		smoothedLevelMultiplier = math.Pow(
			float64(bottomLevelSize)/float64(baseBytesMax),
			1.0/float64(numLevels-p.baseLevel-1))
	}

	levelSize := float64(baseBytesMax)
	for level := p.baseLevel; level < numLevels; level++ {
		if level > p.baseLevel && levelSize > 0 {
			levelSize *= smoothedLevelMultiplier
		}
		// Round the result since test cases use small target level sizes, which
		// can be impacted by floating-point imprecision + integer truncation.
		roundedLevelSize := math.Round(levelSize)
		if roundedLevelSize > float64(math.MaxInt64) {
			p.levelMaxBytes[level] = math.MaxInt64
		} else {
			p.levelMaxBytes[level] = int64(roundedLevelSize)
		}
	}
}

type levelSizeAdjust struct {
	incomingActualBytes      uint64
	outgoingActualBytes      uint64
	outgoingCompensatedBytes uint64
}

func (a levelSizeAdjust) compensated() uint64 {
	return a.incomingActualBytes - a.outgoingCompensatedBytes
}

func (a levelSizeAdjust) actual() uint64 {
	return a.incomingActualBytes - a.outgoingActualBytes
}

func calculateSizeAdjust(inProgressCompactions []compactionInfo) [numLevels]levelSizeAdjust {
	// Compute size adjustments for each level based on the in-progress
	// compactions. We sum the file sizes of all files leaving and entering each
	// level in in-progress compactions. For outgoing files, we also sum a
	// separate sum of 'compensated file sizes', which are inflated according
	// to deletion estimates.
	//
	// When we adjust a level's size according to these values during score
	// calculation, we subtract the compensated size of start level inputs to
	// account for the fact that score calculation uses compensated sizes.
	//
	// Since compensated file sizes may be compensated because they reclaim
	// space from the output level's files, we only add the real file size to
	// the output level.
	//
	// This is slightly different from RocksDB's behavior, which simply elides
	// compacting files from the level size calculation.
	var sizeAdjust [numLevels]levelSizeAdjust
	for i := range inProgressCompactions {
		c := &inProgressCompactions[i]
		// If this compaction's version edit has already been applied, there's
		// no need to adjust: The LSM we'll examine will already reflect the
		// new LSM state.
		if c.versionEditApplied {
			continue
		}

		for _, input := range c.inputs {
			actualSize := input.files.SizeSum()
			compensatedSize := totalCompensatedSize(input.files.Iter())

			if input.level != c.outputLevel {
				sizeAdjust[input.level].outgoingCompensatedBytes += compensatedSize
				sizeAdjust[input.level].outgoingActualBytes += actualSize
				if c.outputLevel != -1 {
					sizeAdjust[c.outputLevel].incomingActualBytes += actualSize
				}
			}
		}
	}
	return sizeAdjust
}

func levelCompensatedSize(lm manifest.LevelMetadata) uint64 {
	return *lm.Annotation(compensatedSizeAnnotator{}).(*uint64)
}

func (p *compactionPickerByScore) calculateLevelScores(
	inProgressCompactions []compactionInfo,
) [numLevels]candidateLevelInfo {
	var scores [numLevels]candidateLevelInfo
	for i := range scores {
		scores[i].level = i
		scores[i].outputLevel = i + 1
	}
	l0UncompensatedScore := calculateL0UncompensatedScore(p.vers, p.opts, inProgressCompactions)
	scores[0] = candidateLevelInfo{
		outputLevel:        p.baseLevel,
		uncompensatedScore: l0UncompensatedScore,
		compensatedScore:   l0UncompensatedScore, /* No level size compensation for L0 */
	}
	sizeAdjust := calculateSizeAdjust(inProgressCompactions)
	for level := 1; level < numLevels; level++ {
		compensatedLevelSize := levelCompensatedSize(p.vers.Levels[level]) + sizeAdjust[level].compensated()
		scores[level].compensatedScore = float64(compensatedLevelSize) / float64(p.levelMaxBytes[level])
		scores[level].uncompensatedScore = float64(p.vers.Levels[level].Size()+sizeAdjust[level].actual()) / float64(p.levelMaxBytes[level])
	}

	// Adjust each level's {compensated, uncompensated}Score by the uncompensatedScore
	// of the next level to get a {compensated, uncompensated}ScoreRatio. If the
	// next level has a high uncompensatedScore, and is thus a priority for compaction,
	// this reduces the priority for compacting the current level. If the next level
	// has a low uncompensatedScore (i.e. it is below its target size), this increases
	// the priority for compacting the current level.
	//
	// The effect of this adjustment is to help prioritize compactions in lower
	// levels. The following example shows the compensatedScoreRatio and the
	// compensatedScore. In this scenario, L0 has 68 sublevels. L3 (a.k.a. Lbase)
	// is significantly above its target size. The original score prioritizes
	// compactions from those two levels, but doing so ends up causing a future
	// problem: data piles up in the higher levels, starving L5->L6 compactions,
	// and to a lesser degree starving L4->L5 compactions.
	//
	// Note that in the example shown there is no level size compensation so the
	// compensatedScore and the uncompensatedScore is the same for each level.
	//
	//        compensatedScoreRatio   compensatedScore   uncompensatedScore   size   max-size
	//   L0                     3.2               68.0                 68.0  2.2 G          -
	//   L3                     3.2               21.1                 21.1  1.3 G       64 M
	//   L4                     3.4                6.7                  6.7  3.1 G      467 M
	//   L5                     3.4                2.0                  2.0  6.6 G      3.3 G
	//   L6                     0.6                0.6                  0.6   14 G       24 G
	var prevLevel int
	for level := p.baseLevel; level < numLevels; level++ {
		// The compensated scores, and uncompensated scores will be turned into
		// ratios as they're adjusted according to other levels' sizes.
		scores[prevLevel].compensatedScoreRatio = scores[prevLevel].compensatedScore
		scores[prevLevel].uncompensatedScoreRatio = scores[prevLevel].uncompensatedScore

		// Avoid absurdly large scores by placing a floor on the score that we'll
		// adjust a level by. The value of 0.01 was chosen somewhat arbitrarily.
		const minScore = 0.01
		if scores[prevLevel].compensatedScoreRatio >= compactionScoreThreshold {
			if scores[level].uncompensatedScore >= minScore {
				scores[prevLevel].compensatedScoreRatio /= scores[level].uncompensatedScore
			} else {
				scores[prevLevel].compensatedScoreRatio /= minScore
			}
		}
		if scores[prevLevel].uncompensatedScoreRatio >= compactionScoreThreshold {
			if scores[level].uncompensatedScore >= minScore {
				scores[prevLevel].uncompensatedScoreRatio /= scores[level].uncompensatedScore
			} else {
				scores[prevLevel].uncompensatedScoreRatio /= minScore
			}
		}
		prevLevel = level
	}
	// Set the score ratios for the lowest level.
	// INVARIANT: prevLevel == numLevels-1
	scores[prevLevel].compensatedScoreRatio = scores[prevLevel].compensatedScore
	scores[prevLevel].uncompensatedScoreRatio = scores[prevLevel].uncompensatedScore

	sort.Sort(sortCompactionLevelsByPriority(scores[:]))
	return scores
}

// calculateL0UncompensatedScore calculates a float score representing the
// relative priority of compacting L0. Level L0 is special in that files within
// L0 may overlap one another, so a different set of heuristics that take into
// account read amplification apply.
func calculateL0UncompensatedScore(
	vers *version, opts *Options, inProgressCompactions []compactionInfo,
) float64 {
	// Use the sublevel count to calculate the score. The base vs intra-L0
	// compaction determination happens in pickAuto, not here.
	score := float64(2*vers.L0Sublevels.MaxDepthAfterOngoingCompactions()) /
		float64(opts.L0CompactionThreshold)

	// Also calculate a score based on the file count but use it only if it
	// produces a higher score than the sublevel-based one. This heuristic is
	// designed to accommodate cases where L0 is accumulating non-overlapping
	// files in L0. Letting too many non-overlapping files accumulate in few
	// sublevels is undesirable, because:
	// 1) we can produce a massive backlog to compact once files do overlap.
	// 2) constructing L0 sublevels has a runtime that grows superlinearly with
	//    the number of files in L0 and must be done while holding D.mu.
	noncompactingFiles := vers.Levels[0].Len()
	for _, c := range inProgressCompactions {
		for _, cl := range c.inputs {
			if cl.level == 0 {
				noncompactingFiles -= cl.files.Len()
			}
		}
	}
	fileScore := float64(noncompactingFiles) / float64(opts.L0CompactionFileThreshold)
	if score < fileScore {
		score = fileScore
	}
	return score
}

// pickCompactionSeedFile picks a file from `level` in the `vers` to build a
// compaction around. Currently, this function implements a heuristic similar to
// RocksDB's kMinOverlappingRatio, seeking to minimize write amplification. This
// function is linear with respect to the number of files in `level` and
// `outputLevel`.
func pickCompactionSeedFile(
	vers *version, opts *Options, level, outputLevel int, earliestSnapshotSeqNum uint64,
) (manifest.LevelFile, bool) {
	// Select the file within the level to compact. We want to minimize write
	// amplification, but also ensure that deletes are propagated to the
	// bottom level in a timely fashion so as to reclaim disk space. A table's
	// smallest sequence number provides a measure of its age. The ratio of
	// overlapping-bytes / table-size gives an indication of write
	// amplification (a smaller ratio is preferrable).
	//
	// The current heuristic is based off the the RocksDB kMinOverlappingRatio
	// heuristic. It chooses the file with the minimum overlapping ratio with
	// the target level, which minimizes write amplification.
	//
	// It uses a "compensated size" for the denominator, which is the file
	// size but artificially inflated by an estimate of the space that may be
	// reclaimed through compaction. Currently, we only compensate for range
	// deletions and only with a rough estimate of the reclaimable bytes. This
	// differs from RocksDB which only compensates for point tombstones and
	// only if they exceed the number of non-deletion entries in table.
	//
	// TODO(peter): For concurrent compactions, we may want to try harder to
	// pick a seed file whose resulting compaction bounds do not overlap with
	// an in-progress compaction.

	cmp := opts.Comparer.Compare
	startIter := vers.Levels[level].Iter()
	outputIter := vers.Levels[outputLevel].Iter()

	var file manifest.LevelFile
	smallestRatio := uint64(math.MaxUint64)

	outputFile := outputIter.First()

	for f := startIter.First(); f != nil; f = startIter.Next() {
		var overlappingBytes uint64
		compacting := f.IsCompacting()
		if compacting {
			// Move on if this file is already being compacted. We'll likely
			// still need to move past the overlapping output files regardless,
			// but in cases where all start-level files are compacting we won't.
			continue
		}

		// Trim any output-level files smaller than f.
		for outputFile != nil && sstableKeyCompare(cmp, outputFile.Largest, f.Smallest) < 0 {
			outputFile = outputIter.Next()
		}

		for outputFile != nil && sstableKeyCompare(cmp, outputFile.Smallest, f.Largest) <= 0 && !compacting {
			overlappingBytes += outputFile.Size
			compacting = compacting || outputFile.IsCompacting()

			// For files in the bottommost level of the LSM, the
			// Stats.RangeDeletionsBytesEstimate field is set to the estimate
			// of bytes /within/ the file itself that may be dropped by
			// recompacting the file. These bytes from obsolete keys would not
			// need to be rewritten if we compacted `f` into `outputFile`, so
			// they don't contribute to write amplification. Subtracting them
			// out of the overlapping bytes helps prioritize these compactions
			// that are cheaper than their file sizes suggest.
			if outputLevel == numLevels-1 && outputFile.LargestSeqNum < earliestSnapshotSeqNum {
				overlappingBytes -= outputFile.Stats.RangeDeletionsBytesEstimate
			}

			// If the file in the next level extends beyond f's largest key,
			// break out and don't advance outputIter because f's successor
			// might also overlap.
			//
			// Note, we stop as soon as we encounter an output-level file with a
			// largest key beyond the input-level file's largest bound. We
			// perform a simple user key comparison here using sstableKeyCompare
			// which handles the potential for exclusive largest key bounds.
			// There's some subtlety when the bounds are equal (eg, equal and
			// inclusive, or equal and exclusive). Current Pebble doesn't split
			// user keys across sstables within a level (and in format versions
			// FormatSplitUserKeysMarkedCompacted and later we guarantee no
			// split user keys exist within the entire LSM). In that case, we're
			// assured that neither the input level nor the output level's next
			// file shares the same user key, so compaction expansion will not
			// include them in any compaction compacting `f`.
			//
			// NB: If we /did/ allow split user keys, or we're running on an
			// old database with an earlier format major version where there are
			// existing split user keys, this logic would be incorrect. Consider
			//    L1: [a#120,a#100] [a#80,a#60]
			//    L2: [a#55,a#45] [a#35,a#25] [a#15,a#5]
			// While considering the first file in L1, [a#120,a#100], we'd skip
			// past all of the files in L2. When considering the second file in
			// L1, we'd improperly conclude that the second file overlaps
			// nothing in the second level and is cheap to compact, when in
			// reality we'd need to expand the compaction to include all 5
			// files.
			if sstableKeyCompare(cmp, outputFile.Largest, f.Largest) > 0 {
				break
			}
			outputFile = outputIter.Next()
		}

		// If the input level file or one of the overlapping files is
		// compacting, we're not going to be able to compact this file
		// anyways, so skip it.
		if compacting {
			continue
		}

		compSz := compensatedSize(f)
		scaledRatio := overlappingBytes * 1024 / compSz
		if scaledRatio < smallestRatio {
			smallestRatio = scaledRatio
			file = startIter.Take()
		}
	}
	return file, file.FileMetadata != nil
}

// pickAuto picks the best compaction, if any.
//
// On each call, pickAuto computes per-level size adjustments based on
// in-progress compactions, and computes a per-level score. The levels are
// iterated over in decreasing score order trying to find a valid compaction
// anchored at that level.
//
// If a score-based compaction cannot be found, pickAuto falls back to looking
// for an elision-only compaction to remove obsolete keys.
func (p *compactionPickerByScore) pickAuto(env compactionEnv) (pc *pickedCompaction) {
	// Compaction concurrency is controlled by L0 read-amp. We allow one
	// additional compaction per L0CompactionConcurrency sublevels, as well as
	// one additional compaction per CompactionDebtConcurrency bytes of
	// compaction debt. Compaction concurrency is tied to L0 sublevels as that
	// signal is independent of the database size. We tack on the compaction
	// debt as a second signal to prevent compaction concurrency from dropping
	// significantly right after a base compaction finishes, and before those
	// bytes have been compacted further down the LSM.
	if n := len(env.inProgressCompactions); n > 0 {
		l0ReadAmp := p.vers.L0Sublevels.MaxDepthAfterOngoingCompactions()
		compactionDebt := p.estimatedCompactionDebt(0)
		ccSignal1 := n * p.opts.Experimental.L0CompactionConcurrency
		ccSignal2 := uint64(n) * p.opts.Experimental.CompactionDebtConcurrency
		if l0ReadAmp < ccSignal1 && compactionDebt < ccSignal2 {
			return nil
		}
	}

	scores := p.calculateLevelScores(env.inProgressCompactions)

	// TODO(bananabrick): Either remove, or change this into an event sent to the
	// EventListener.
	logCompaction := func(pc *pickedCompaction) {
		var buf bytes.Buffer
		for i := 0; i < numLevels; i++ {
			if i != 0 && i < p.baseLevel {
				continue
			}

			var info *candidateLevelInfo
			for j := range scores {
				if scores[j].level == i {
					info = &scores[j]
					break
				}
			}

			marker := " "
			if pc.startLevel.level == info.level {
				marker = "*"
			}
			fmt.Fprintf(&buf, "  %sL%d: %5.1f  %5.1f  %5.1f  %5.1f %8s  %8s",
				marker, info.level, info.compensatedScoreRatio, info.compensatedScore,
				info.uncompensatedScoreRatio, info.uncompensatedScore,
				humanize.Bytes.Int64(int64(totalCompensatedSize(
					p.vers.Levels[info.level].Iter(),
				))),
				humanize.Bytes.Int64(p.levelMaxBytes[info.level]),
			)

			count := 0
			for i := range env.inProgressCompactions {
				c := &env.inProgressCompactions[i]
				if c.inputs[0].level != info.level {
					continue
				}
				count++
				if count == 1 {
					fmt.Fprintf(&buf, "  [")
				} else {
					fmt.Fprintf(&buf, " ")
				}
				fmt.Fprintf(&buf, "L%d->L%d", c.inputs[0].level, c.outputLevel)
			}
			if count > 0 {
				fmt.Fprintf(&buf, "]")
			}
			fmt.Fprintf(&buf, "\n")
		}
		p.opts.Logger.Infof("pickAuto: L%d->L%d\n%s",
			pc.startLevel.level, pc.outputLevel.level, buf.String())
	}

	// Check for a score-based compaction. candidateLevelInfos are first sorted
	// by whether they should be compacted, so if we find a level which shouldn't
	// be compacted, we can break early.
	for i := range scores {
		info := &scores[i]
		if !info.shouldCompact() {
			break
		}
		if info.level == numLevels-1 {
			continue
		}

		if info.level == 0 {
			pc = pickL0(env, p.opts, p.vers, p.baseLevel)
			// Fail-safe to protect against compacting the same sstable
			// concurrently.
			if pc != nil && !inputRangeAlreadyCompacting(env, pc) {
				p.addScoresToPickedCompactionMetrics(pc, scores)
				pc.score = info.compensatedScoreRatio
				// TODO(bananabrick): Create an EventListener for logCompaction.
				if false {
					logCompaction(pc)
				}
				return pc
			}
			continue
		}

		// info.level > 0
		var ok bool
		info.file, ok = pickCompactionSeedFile(p.vers, p.opts, info.level, info.outputLevel, env.earliestSnapshotSeqNum)
		if !ok {
			continue
		}

		pc := pickAutoLPositive(env, p.opts, p.vers, *info, p.baseLevel, p.levelMaxBytes)
		// Fail-safe to protect against compacting the same sstable concurrently.
		if pc != nil && !inputRangeAlreadyCompacting(env, pc) {
			p.addScoresToPickedCompactionMetrics(pc, scores)
			pc.score = info.compensatedScoreRatio
			// TODO(bananabrick): Create an EventListener for logCompaction.
			if false {
				logCompaction(pc)
			}
			return pc
		}
	}

	// Check for L6 files with tombstones that may be elided. These files may
	// exist if a snapshot prevented the elision of a tombstone or because of
	// a move compaction. These are low-priority compactions because they
	// don't help us keep up with writes, just reclaim disk space.
	if pc := p.pickElisionOnlyCompaction(env); pc != nil {
		return pc
	}

	if pc := p.pickReadTriggeredCompaction(env); pc != nil {
		return pc
	}

	// NB: This should only be run if a read compaction wasn't
	// scheduled.
	//
	// We won't be scheduling a read compaction right now, and in
	// read heavy workloads, compactions won't be scheduled frequently
	// because flushes aren't frequent. So we need to signal to the
	// iterator to schedule a compaction when it adds compactions to
	// the read compaction queue.
	//
	// We need the nil check here because without it, we have some
	// tests which don't set that variable fail. Since there's a
	// chance that one of those tests wouldn't want extra compactions
	// to be scheduled, I added this check here, instead of
	// setting rescheduleReadCompaction in those tests.
	if env.readCompactionEnv.rescheduleReadCompaction != nil {
		*env.readCompactionEnv.rescheduleReadCompaction = true
	}

	// At the lowest possible compaction-picking priority, look for files marked
	// for compaction. Pebble will mark files for compaction if they have atomic
	// compaction units that span multiple files. While current Pebble code does
	// not construct such sstables, RocksDB and earlier versions of Pebble may
	// have created them. These split user keys form sets of files that must be
	// compacted together for correctness (referred to as "atomic compaction
	// units" within the code). Rewrite them in-place.
	//
	// It's also possible that a file may have been marked for compaction by
	// even earlier versions of Pebble code, since FileMetadata's
	// MarkedForCompaction field is persisted in the manifest. That's okay. We
	// previously would've ignored the designation, whereas now we'll re-compact
	// the file in place.
	if p.vers.Stats.MarkedForCompaction > 0 {
		if pc := p.pickRewriteCompaction(env); pc != nil {
			return pc
		}
	}

	return nil
}

func (p *compactionPickerByScore) addScoresToPickedCompactionMetrics(
	pc *pickedCompaction, candInfo [numLevels]candidateLevelInfo,
) {

	// candInfo is sorted by score, not by compaction level.
	infoByLevel := [numLevels]candidateLevelInfo{}
	for i := range candInfo {
		level := candInfo[i].level
		infoByLevel[level] = candInfo[i]
	}
	// Gather the compaction scores for the levels participating in the compaction.
	pc.pickerMetrics.scores = make([]float64, len(pc.inputs))
	inputIdx := 0
	for i := range infoByLevel {
		if pc.inputs[inputIdx].level == infoByLevel[i].level {
			pc.pickerMetrics.scores[inputIdx] = infoByLevel[i].compensatedScoreRatio
			inputIdx++
		}
		if inputIdx == len(pc.inputs) {
			break
		}
	}
}

// elisionOnlyAnnotator implements the manifest.Annotator interface,
// annotating B-Tree nodes with the *fileMetadata of a file meeting the
// obsolete keys criteria for an elision-only compaction within the subtree.
// If multiple files meet the criteria, it chooses whichever file has the
// lowest LargestSeqNum. The lowest LargestSeqNum file will be the first
// eligible for an elision-only compaction once snapshots less than or equal
// to its LargestSeqNum are closed.
type elisionOnlyAnnotator struct{}

var _ manifest.Annotator = elisionOnlyAnnotator{}

func (a elisionOnlyAnnotator) Zero(interface{}) interface{} {
	return nil
}

func (a elisionOnlyAnnotator) Accumulate(f *fileMetadata, dst interface{}) (interface{}, bool) {
	if f.IsCompacting() {
		return dst, true
	}
	if !f.StatsValid() {
		return dst, false
	}
	// Bottommost files are large and not worthwhile to compact just
	// to remove a few tombstones. Consider a file ineligible if its
	// own range deletions delete less than 10% of its data and its
	// deletion tombstones make up less than 10% of its entries.
	//
	// TODO(jackson): This does not account for duplicate user keys
	// which may be collapsed. Ideally, we would have 'obsolete keys'
	// statistics that would include tombstones, the keys that are
	// dropped by tombstones and duplicated user keys. See #847.
	//
	// Note that tables that contain exclusively range keys (i.e. no point keys,
	// `NumEntries` and `RangeDeletionsBytesEstimate` are both zero) are excluded
	// from elision-only compactions.
	// TODO(travers): Consider an alternative heuristic for elision of range-keys.
	if f.Stats.RangeDeletionsBytesEstimate*10 < f.Size &&
		f.Stats.NumDeletions*10 <= f.Stats.NumEntries {
		return dst, true
	}
	if dst == nil {
		return f, true
	} else if dstV := dst.(*fileMetadata); dstV.LargestSeqNum > f.LargestSeqNum {
		return f, true
	}
	return dst, true
}

func (a elisionOnlyAnnotator) Merge(v interface{}, accum interface{}) interface{} {
	if v == nil {
		return accum
	}
	// If we haven't accumulated an eligible file yet, or f's LargestSeqNum is
	// less than the accumulated file's, use f.
	if accum == nil {
		return v
	}
	f := v.(*fileMetadata)
	accumV := accum.(*fileMetadata)
	if accumV == nil || accumV.LargestSeqNum > f.LargestSeqNum {
		return f
	}
	return accumV
}

// markedForCompactionAnnotator implements the manifest.Annotator interface,
// annotating B-Tree nodes with the *fileMetadata of a file that is marked for
// compaction within the subtree. If multiple files meet the criteria, it
// chooses whichever file has the lowest LargestSeqNum.
type markedForCompactionAnnotator struct{}

var _ manifest.Annotator = markedForCompactionAnnotator{}

func (a markedForCompactionAnnotator) Zero(interface{}) interface{} {
	return nil
}

func (a markedForCompactionAnnotator) Accumulate(
	f *fileMetadata, dst interface{},
) (interface{}, bool) {
	if !f.MarkedForCompaction {
		// Not marked for compaction; return dst.
		return dst, true
	}
	return markedMergeHelper(f, dst)
}

func (a markedForCompactionAnnotator) Merge(v interface{}, accum interface{}) interface{} {
	if v == nil {
		return accum
	}
	accum, _ = markedMergeHelper(v.(*fileMetadata), accum)
	return accum
}

// REQUIRES: f is non-nil, and f.MarkedForCompaction=true.
func markedMergeHelper(f *fileMetadata, dst interface{}) (interface{}, bool) {
	if dst == nil {
		return f, true
	} else if dstV := dst.(*fileMetadata); dstV.LargestSeqNum > f.LargestSeqNum {
		return f, true
	}
	return dst, true
}

// pickElisionOnlyCompaction looks for compactions of sstables in the
// bottommost level containing obsolete records that may now be dropped.
func (p *compactionPickerByScore) pickElisionOnlyCompaction(
	env compactionEnv,
) (pc *pickedCompaction) {
	if p.opts.private.disableElisionOnlyCompactions {
		return nil
	}
	v := p.vers.Levels[numLevels-1].Annotation(elisionOnlyAnnotator{})
	if v == nil {
		return nil
	}
	candidate := v.(*fileMetadata)
	if candidate.IsCompacting() || candidate.LargestSeqNum >= env.earliestSnapshotSeqNum {
		return nil
	}
	lf := p.vers.Levels[numLevels-1].Find(p.opts.Comparer.Compare, candidate)
	if lf == nil {
		panic(fmt.Sprintf("file %s not found in level %d as expected", candidate.FileNum, numLevels-1))
	}

	// Construct a picked compaction of the elision candidate's atomic
	// compaction unit.
	pc = newPickedCompaction(p.opts, p.vers, numLevels-1, numLevels-1, p.baseLevel)
	pc.kind = compactionKindElisionOnly
	var isCompacting bool
	pc.startLevel.files, isCompacting = expandToAtomicUnit(p.opts.Comparer.Compare, lf.Slice(), false /* disableIsCompacting */)
	if isCompacting {
		return nil
	}
	pc.smallest, pc.largest = manifest.KeyRange(pc.cmp, pc.startLevel.files.Iter())
	// Fail-safe to protect against compacting the same sstable concurrently.
	if !inputRangeAlreadyCompacting(env, pc) {
		return pc
	}
	return nil
}

// pickRewriteCompaction attempts to construct a compaction that
// rewrites a file marked for compaction. pickRewriteCompaction will
// pull in adjacent files in the file's atomic compaction unit if
// necessary. A rewrite compaction outputs files to the same level as
// the input level.
func (p *compactionPickerByScore) pickRewriteCompaction(env compactionEnv) (pc *pickedCompaction) {
	for l := numLevels - 1; l >= 0; l-- {
		v := p.vers.Levels[l].Annotation(markedForCompactionAnnotator{})
		if v == nil {
			// Try the next level.
			continue
		}
		candidate := v.(*fileMetadata)
		if candidate.IsCompacting() {
			// Try the next level.
			continue
		}
		lf := p.vers.Levels[l].Find(p.opts.Comparer.Compare, candidate)
		if lf == nil {
			panic(fmt.Sprintf("file %s not found in level %d as expected", candidate.FileNum, numLevels-1))
		}

		inputs := lf.Slice()
		// L0 files generated by a flush have never been split such that
		// adjacent files can contain the same user key. So we do not need to
		// rewrite an atomic compaction unit for L0. Note that there is nothing
		// preventing two different flushes from producing files that are
		// non-overlapping from an InternalKey perspective, but span the same
		// user key. However, such files cannot be in the same L0 sublevel,
		// since each sublevel requires non-overlapping user keys (unlike other
		// levels).
		if l > 0 {
			// Find this file's atomic compaction unit. This is only relevant
			// for levels L1+.
			var isCompacting bool
			inputs, isCompacting = expandToAtomicUnit(
				p.opts.Comparer.Compare,
				inputs,
				false, /* disableIsCompacting */
			)
			if isCompacting {
				// Try the next level.
				continue
			}
		}

		pc = newPickedCompaction(p.opts, p.vers, l, l, p.baseLevel)
		pc.outputLevel.level = l
		pc.kind = compactionKindRewrite
		pc.startLevel.files = inputs
		pc.smallest, pc.largest = manifest.KeyRange(pc.cmp, pc.startLevel.files.Iter())

		// Fail-safe to protect against compacting the same sstable concurrently.
		if !inputRangeAlreadyCompacting(env, pc) {
			if pc.startLevel.level == 0 {
				pc.startLevel.l0SublevelInfo = generateSublevelInfo(pc.cmp, pc.startLevel.files)
			}
			return pc
		}
	}
	return nil
}

// pickAutoLPositive picks an automatic compaction for the candidate
// file in a positive-numbered level. This function must not be used for
// L0.
func pickAutoLPositive(
	env compactionEnv,
	opts *Options,
	vers *version,
	cInfo candidateLevelInfo,
	baseLevel int,
	levelMaxBytes [7]int64,
) (pc *pickedCompaction) {
	if cInfo.level == 0 {
		panic("pebble: pickAutoLPositive called for L0")
	}

	pc = newPickedCompaction(opts, vers, cInfo.level, defaultOutputLevel(cInfo.level, baseLevel), baseLevel)
	if pc.outputLevel.level != cInfo.outputLevel {
		panic("pebble: compaction picked unexpected output level")
	}
	pc.startLevel.files = cInfo.file.Slice()
	// Files in level 0 may overlap each other, so pick up all overlapping ones.
	if pc.startLevel.level == 0 {
		cmp := opts.Comparer.Compare
		smallest, largest := manifest.KeyRange(cmp, pc.startLevel.files.Iter())
		pc.startLevel.files = vers.Overlaps(0, cmp, smallest.UserKey,
			largest.UserKey, largest.IsExclusiveSentinel())
		if pc.startLevel.files.Empty() {
			panic("pebble: empty compaction")
		}
	}

	if !pc.setupInputs(opts, env.diskAvailBytes, pc.startLevel) {
		return nil
	}
	return pc.maybeAddLevel(opts, env.diskAvailBytes)
}

// maybeAddLevel maybe adds a level to the picked compaction.
func (pc *pickedCompaction) maybeAddLevel(opts *Options, diskAvailBytes uint64) *pickedCompaction {
	pc.pickerMetrics.singleLevelOverlappingRatio = pc.overlappingRatio()
	if pc.outputLevel.level == numLevels-1 {
		// Don't add a level if the current output level is in L6
		return pc
	}
	if !opts.Experimental.MultiLevelCompactionHeuristic.allowL0() && pc.startLevel.level == 0 {
		return pc
	}
	if pc.compactionSize() > expandedCompactionByteSizeLimit(
		opts, adjustedOutputLevel(pc.outputLevel.level, pc.baseLevel), diskAvailBytes) {
		// Don't add a level if the current compaction exceeds the compaction size limit
		return pc
	}
	return opts.Experimental.MultiLevelCompactionHeuristic.pick(pc, opts, diskAvailBytes)
}

// MultiLevelHeuristic evaluates whether to add files from the next level into the compaction.
type MultiLevelHeuristic interface {
	// Evaluate returns the preferred compaction.
	pick(pc *pickedCompaction, opts *Options, diskAvailBytes uint64) *pickedCompaction

	// Returns if the heuristic allows L0 to be involved in ML compaction
	allowL0() bool
}

// NoMultiLevel will never add an additional level to the compaction.
type NoMultiLevel struct{}

var _ MultiLevelHeuristic = (*NoMultiLevel)(nil)

func (nml NoMultiLevel) pick(
	pc *pickedCompaction, opts *Options, diskAvailBytes uint64,
) *pickedCompaction {
	return pc
}

func (nml NoMultiLevel) allowL0() bool {
	return false
}

func (pc *pickedCompaction) predictedWriteAmp() float64 {
	var bytesToCompact uint64
	var higherLevelBytes uint64
	for i := range pc.inputs {
		levelSize := pc.inputs[i].files.SizeSum()
		bytesToCompact += levelSize
		if i != len(pc.inputs)-1 {
			higherLevelBytes += levelSize
		}
	}
	return float64(bytesToCompact) / float64(higherLevelBytes)
}

func (pc *pickedCompaction) overlappingRatio() float64 {
	var higherLevelBytes uint64
	var lowestLevelBytes uint64
	for i := range pc.inputs {
		levelSize := pc.inputs[i].files.SizeSum()
		if i == len(pc.inputs)-1 {
			lowestLevelBytes += levelSize
			continue
		}
		higherLevelBytes += levelSize
	}
	return float64(lowestLevelBytes) / float64(higherLevelBytes)
}

// WriteAmpHeuristic defines a multi level compaction heuristic which will add
// an additional level to the picked compaction if it reduces predicted write
// amp of the compaction + the addPropensity constant.
type WriteAmpHeuristic struct {
	// addPropensity is a constant that affects the propensity to conduct multilevel
	// compactions. If positive, a multilevel compaction may get picked even if
	// the single level compaction has lower write amp, and vice versa.
	AddPropensity float64

	// AllowL0 if true, allow l0 to be involved in a ML compaction.
	AllowL0 bool
}

var _ MultiLevelHeuristic = (*WriteAmpHeuristic)(nil)

// TODO(msbutler): microbenchmark the extent to which multilevel compaction
// picking slows down the compaction picking process.  This should be as fast as
// possible since Compaction-picking holds d.mu, which prevents WAL rotations,
// in-progress flushes and compactions from completing, etc. Consider ways to
// deduplicate work, given that setupInputs has already been called.
func (wa WriteAmpHeuristic) pick(
	pcOrig *pickedCompaction, opts *Options, diskAvailBytes uint64,
) *pickedCompaction {
	pcMulti := pcOrig.clone()
	if !pcMulti.setupMultiLevelCandidate(opts, diskAvailBytes) {
		return pcOrig
	}
	picked := pcOrig
	if pcMulti.predictedWriteAmp() <= pcOrig.predictedWriteAmp()+wa.AddPropensity {
		picked = pcMulti
	}
	// Regardless of what compaction was picked, log the multilevelOverlapping ratio.
	picked.pickerMetrics.multiLevelOverlappingRatio = pcMulti.overlappingRatio()
	return picked
}

func (wa WriteAmpHeuristic) allowL0() bool {
	return wa.AllowL0
}

// Helper method to pick compactions originating from L0. Uses information about
// sublevels to generate a compaction.
func pickL0(env compactionEnv, opts *Options, vers *version, baseLevel int) (pc *pickedCompaction) {
	// It is important to pass information about Lbase files to L0Sublevels
	// so it can pick a compaction that does not conflict with an Lbase => Lbase+1
	// compaction. Without this, we observed reduced concurrency of L0=>Lbase
	// compactions, and increasing read amplification in L0.
	//
	// TODO(bilal) Remove the minCompactionDepth parameter once fixing it at 1
	// has been shown to not cause a performance regression.
	lcf, err := vers.L0Sublevels.PickBaseCompaction(1, vers.Levels[baseLevel].Slice())
	if err != nil {
		opts.Logger.Infof("error when picking base compaction: %s", err)
		return
	}
	if lcf != nil {
		pc = newPickedCompactionFromL0(lcf, opts, vers, baseLevel, true)
		pc.setupInputs(opts, env.diskAvailBytes, pc.startLevel)
		if pc.startLevel.files.Empty() {
			opts.Logger.Fatalf("empty compaction chosen")
		}
		return pc.maybeAddLevel(opts, env.diskAvailBytes)
	}

	// Couldn't choose a base compaction. Try choosing an intra-L0
	// compaction. Note that we pass in L0CompactionThreshold here as opposed to
	// 1, since choosing a single sublevel intra-L0 compaction is
	// counterproductive.
	lcf, err = vers.L0Sublevels.PickIntraL0Compaction(env.earliestUnflushedSeqNum, minIntraL0Count)
	if err != nil {
		opts.Logger.Infof("error when picking intra-L0 compaction: %s", err)
		return
	}
	if lcf != nil {
		pc = newPickedCompactionFromL0(lcf, opts, vers, 0, false)
		if !pc.setupInputs(opts, env.diskAvailBytes, pc.startLevel) {
			return nil
		}
		if pc.startLevel.files.Empty() {
			opts.Logger.Fatalf("empty compaction chosen")
		}
		{
			iter := pc.startLevel.files.Iter()
			if iter.First() == nil || iter.Next() == nil {
				// A single-file intra-L0 compaction is unproductive.
				return nil
			}
		}

		pc.smallest, pc.largest = manifest.KeyRange(pc.cmp, pc.startLevel.files.Iter())
	}
	return pc
}

func pickManualCompaction(
	vers *version, opts *Options, env compactionEnv, baseLevel int, manual *manualCompaction,
) (pc *pickedCompaction, retryLater bool) {
	outputLevel := manual.level + 1
	if manual.level == 0 {
		outputLevel = baseLevel
	} else if manual.level < baseLevel {
		// The start level for a compaction must be >= Lbase. A manual
		// compaction could have been created adhering to that condition, and
		// then an automatic compaction came in and compacted all of the
		// sstables in Lbase to Lbase+1 which caused Lbase to change. Simply
		// ignore this manual compaction as there is nothing to do (manual.level
		// points to an empty level).
		return nil, false
	}
	// This conflictsWithInProgress call is necessary for the manual compaction to
	// be retried when it conflicts with an ongoing automatic compaction. Without
	// it, the compaction is dropped due to pc.setupInputs returning false since
	// the input/output range is already being compacted, and the manual
	// compaction ends with a non-compacted LSM.
	if conflictsWithInProgress(manual, outputLevel, env.inProgressCompactions, opts.Comparer.Compare) {
		return nil, true
	}
	pc = newPickedCompaction(opts, vers, manual.level, defaultOutputLevel(manual.level, baseLevel), baseLevel)
	manual.outputLevel = pc.outputLevel.level
	pc.startLevel.files = vers.Overlaps(manual.level, opts.Comparer.Compare, manual.start, manual.end, false)
	if pc.startLevel.files.Empty() {
		// Nothing to do
		return nil, false
	}
	if !pc.setupInputs(opts, env.diskAvailBytes, pc.startLevel) {
		// setupInputs returned false indicating there's a conflicting
		// concurrent compaction.
		return nil, true
	}
	if pc = pc.maybeAddLevel(opts, env.diskAvailBytes); pc == nil {
		return nil, false
	}
	if pc.outputLevel.level != outputLevel {
		if len(pc.extraLevels) > 0 {
			// multilevel compactions relax this invariant
		} else {
			panic("pebble: compaction picked unexpected output level")
		}
	}
	// Fail-safe to protect against compacting the same sstable concurrently.
	if inputRangeAlreadyCompacting(env, pc) {
		return nil, true
	}
	return pc, false
}

func (p *compactionPickerByScore) pickReadTriggeredCompaction(
	env compactionEnv,
) (pc *pickedCompaction) {
	// If a flush is in-progress or expected to happen soon, it means more writes are taking place. We would
	// soon be scheduling more write focussed compactions. In this case, skip read compactions as they are
	// lower priority.
	if env.readCompactionEnv.flushing || env.readCompactionEnv.readCompactions == nil {
		return nil
	}
	for env.readCompactionEnv.readCompactions.size > 0 {
		rc := env.readCompactionEnv.readCompactions.remove()
		if pc = pickReadTriggeredCompactionHelper(p, rc, env); pc != nil {
			break
		}
	}
	return pc
}

func pickReadTriggeredCompactionHelper(
	p *compactionPickerByScore, rc *readCompaction, env compactionEnv,
) (pc *pickedCompaction) {
	cmp := p.opts.Comparer.Compare
	overlapSlice := p.vers.Overlaps(rc.level, cmp, rc.start, rc.end, false /* exclusiveEnd */)
	if overlapSlice.Empty() {
		// If there is no overlap, then the file with the key range
		// must have been compacted away. So, we don't proceed to
		// compact the same key range again.
		return nil
	}

	iter := overlapSlice.Iter()
	var fileMatches bool
	for f := iter.First(); f != nil; f = iter.Next() {
		if f.FileNum == rc.fileNum {
			fileMatches = true
			break
		}
	}
	if !fileMatches {
		return nil
	}

	pc = newPickedCompaction(p.opts, p.vers, rc.level, defaultOutputLevel(rc.level, p.baseLevel), p.baseLevel)

	pc.startLevel.files = overlapSlice
	if !pc.setupInputs(p.opts, env.diskAvailBytes, pc.startLevel) {
		return nil
	}
	if inputRangeAlreadyCompacting(env, pc) {
		return nil
	}
	pc.kind = compactionKindRead

	// Prevent read compactions which are too wide.
	outputOverlaps := pc.version.Overlaps(
		pc.outputLevel.level, pc.cmp, pc.smallest.UserKey,
		pc.largest.UserKey, pc.largest.IsExclusiveSentinel())
	if outputOverlaps.SizeSum() > pc.maxReadCompactionBytes {
		return nil
	}

	// Prevent compactions which start with a small seed file X, but overlap
	// with over allowedCompactionWidth * X file sizes in the output layer.
	const allowedCompactionWidth = 35
	if outputOverlaps.SizeSum() > overlapSlice.SizeSum()*allowedCompactionWidth {
		return nil
	}

	return pc
}

func (p *compactionPickerByScore) forceBaseLevel1() {
	p.baseLevel = 1
}

func inputRangeAlreadyCompacting(env compactionEnv, pc *pickedCompaction) bool {
	for _, cl := range pc.inputs {
		iter := cl.files.Iter()
		for f := iter.First(); f != nil; f = iter.Next() {
			if f.IsCompacting() {
				return true
			}
		}
	}

	// Look for active compactions outputting to the same region of the key
	// space in the same output level. Two potential compactions may conflict
	// without sharing input files if there are no files in the output level
	// that overlap with the intersection of the compactions' key spaces.
	//
	// Consider an active L0->Lbase compaction compacting two L0 files one
	// [a-f] and the other [t-z] into Lbase.
	//
	// L0
	//     ↦ 000100  ↤                           ↦  000101   ↤
	// L1
	//     ↦ 000004  ↤
	//     a b c d e f g h i j k l m n o p q r s t u v w x y z
	//
	// If a new file 000102 [j-p] is flushed while the existing compaction is
	// still ongoing, new file would not be in any compacting sublevel
	// intervals and would not overlap with any Lbase files that are also
	// compacting. However, this compaction cannot be picked because the
	// compaction's output key space [j-p] would overlap the existing
	// compaction's output key space [a-z].
	//
	// L0
	//     ↦ 000100* ↤       ↦   000102  ↤       ↦  000101*  ↤
	// L1
	//     ↦ 000004* ↤
	//     a b c d e f g h i j k l m n o p q r s t u v w x y z
	//
	// * - currently compacting
	if pc.outputLevel != nil && pc.outputLevel.level != 0 {
		for _, c := range env.inProgressCompactions {
			if pc.outputLevel.level != c.outputLevel {
				continue
			}
			if base.InternalCompare(pc.cmp, c.largest, pc.smallest) < 0 ||
				base.InternalCompare(pc.cmp, c.smallest, pc.largest) > 0 {
				continue
			}

			// The picked compaction and the in-progress compaction c are
			// outputting to the same region of the key space of the same
			// level.
			return true
		}
	}
	return false
}

// conflictsWithInProgress checks if there are any in-progress compactions with overlapping keyspace.
func conflictsWithInProgress(
	manual *manualCompaction, outputLevel int, inProgressCompactions []compactionInfo, cmp Compare,
) bool {
	for _, c := range inProgressCompactions {
		if (c.outputLevel == manual.level || c.outputLevel == outputLevel) &&
			isUserKeysOverlapping(manual.start, manual.end, c.smallest.UserKey, c.largest.UserKey, cmp) {
			return true
		}
		for _, in := range c.inputs {
			if in.files.Empty() {
				continue
			}
			iter := in.files.Iter()
			smallest := iter.First().Smallest.UserKey
			largest := iter.Last().Largest.UserKey
			if (in.level == manual.level || in.level == outputLevel) &&
				isUserKeysOverlapping(manual.start, manual.end, smallest, largest, cmp) {
				return true
			}
		}
	}
	return false
}

func isUserKeysOverlapping(x1, x2, y1, y2 []byte, cmp Compare) bool {
	return cmp(x1, y2) <= 0 && cmp(y1, x2) <= 0
}
