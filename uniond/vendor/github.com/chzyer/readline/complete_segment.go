package readline

type SegmentCompleter interface {
	// a
	// |- a1
	// |--- a11
	// |- a2
	// b
	// input:
	//   DoTree([], 0) [a, b]
	//   DoTree([a], 1) [a]
	//   DoTree([a, ], 0) [a1, a2]
	//   DoTree([a, a], 1) [a1, a2]
	//   DoTree([a, a1], 2) [a1]
	//   DoTree([a, a1, ], 0) [a11]
	//   DoTree([a, a1, a], 1) [a11]
	DoSegment([][]rune, int) [][]rune
}

type dumpSegmentCompleter struct {
	f func([][]rune, int) [][]rune
}

func (d *dumpSegmentCompleter) DoSegment(segment [][]rune, n int) [][]rune {
	return d.f(segment, n)
}

func SegmentFunc(f func([][]rune, int) [][]rune) AutoCompleter {
	return &SegmentComplete{&dumpSegmentCompleter{f}}
}

func SegmentAutoComplete(completer SegmentCompleter) *SegmentComplete {
	return &SegmentComplete{
		SegmentCompleter: completer,
	}
}

type SegmentComplete struct {
	SegmentCompleter
}

func RetSegment(segments [][]rune, cands [][]rune, idx int) ([][]rune, int) {
	ret := make([][]rune, 0, len(cands))
	lastSegment := segments[len(segments)-1]
	for _, cand := range cands {
		if !runes.HasPrefix(cand, lastSegment) {
			continue
		}
		ret = append(ret, cand[len(lastSegment):])
	}
	return ret, idx
}

func SplitSegment(line []rune, pos int) ([][]rune, int) {
	segs := [][]rune{}
	lastIdx := -1
	line = line[:pos]
	pos = 0
	for idx, l := range line {
		if l == ' ' {
			pos = 0
			segs = append(segs, line[lastIdx+1:idx])
			lastIdx = idx
		} else {
			pos++
		}
	}
	segs = append(segs, line[lastIdx+1:])
	return segs, pos
}

func (c *SegmentComplete) Do(line []rune, pos int) (newLine [][]rune, offset int) {

	segment, idx := SplitSegment(line, pos)

	cands := c.DoSegment(segment, idx)
	newLine, offset = RetSegment(segment, cands, idx)
	for idx := range newLine {
		newLine[idx] = append(newLine[idx], ' ')
	}
	return newLine, offset
}
