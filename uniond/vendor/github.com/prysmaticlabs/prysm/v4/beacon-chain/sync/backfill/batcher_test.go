package backfill

import (
	"fmt"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestBatcherBefore(t *testing.T) {
	cases := []struct {
		name   string
		b      batcher
		upTo   []primitives.Slot
		expect []batch
	}{
		{
			name: "size 10",
			b:    batcher{min: 0, size: 10},
			upTo: []primitives.Slot{33, 30, 10, 6},
			expect: []batch{
				{begin: 23, end: 33, state: batchInit},
				{begin: 20, end: 30, state: batchInit},
				{begin: 0, end: 10, state: batchInit},
				{begin: 0, end: 6, state: batchInit},
			},
		},
		{
			name: "size 4",
			b:    batcher{min: 0, size: 4},
			upTo: []primitives.Slot{33, 6, 4},
			expect: []batch{
				{begin: 29, end: 33, state: batchInit},
				{begin: 2, end: 6, state: batchInit},
				{begin: 0, end: 4, state: batchInit},
			},
		},
		{
			name: "trigger end",
			b:    batcher{min: 20, size: 10},
			upTo: []primitives.Slot{33, 30, 25, 21, 20, 19},
			expect: []batch{
				{begin: 23, end: 33, state: batchInit},
				{begin: 20, end: 30, state: batchInit},
				{begin: 20, end: 25, state: batchInit},
				{begin: 20, end: 21, state: batchInit},
				{begin: 20, end: 20, state: batchEndSequence},
				{begin: 19, end: 19, state: batchEndSequence},
			},
		},
	}
	for _, c := range cases {
		for i := range c.upTo {
			upTo := c.upTo[i]
			expect := c.expect[i]
			t.Run(fmt.Sprintf("%s upTo %d", c.name, upTo), func(t *testing.T) {
				got := c.b.before(upTo)
				require.Equal(t, expect.begin, got.begin)
				require.Equal(t, expect.end, got.end)
				require.Equal(t, expect.state, got.state)
			})
		}
	}
}

func TestBatchSequencer(t *testing.T) {
	var min, max, size primitives.Slot
	seqLen := 8
	min = 0
	max = 11235
	size = 64
	seq := newBatchSequencer(seqLen, min, max, size)
	expected := []batch{
		{begin: 11171, end: 11235},
		{begin: 11107, end: 11171},
		{begin: 11043, end: 11107},
		{begin: 10979, end: 11043},
		{begin: 10915, end: 10979},
		{begin: 10851, end: 10915},
		{begin: 10787, end: 10851},
		{begin: 10723, end: 10787},
	}
	got, err := seq.sequence()
	require.Equal(t, seqLen, len(got))
	for i := 0; i < seqLen; i++ {
		g := got[i]
		exp := expected[i]
		require.NoError(t, err)
		require.Equal(t, exp.begin, g.begin)
		require.Equal(t, exp.end, g.end)
		require.Equal(t, batchSequenced, g.state)
	}
	// This should give us the error indicating there are too many outstanding batches.
	_, err = seq.sequence()
	require.ErrorIs(t, err, errMaxBatches)

	// mark the last batch completed so we can call sequence again.
	last := seq.seq[len(seq.seq)-1]
	// With this state, the batch should get served back to us as the next batch.
	last.state = batchErrRetryable
	seq.update(last)
	nextS, err := seq.sequence()
	require.Equal(t, 1, len(nextS))
	next := nextS[0]
	require.NoError(t, err)
	require.Equal(t, last.begin, next.begin)
	require.Equal(t, last.end, next.end)
	// sequence() should replace the batchErrRetryable state with batchSequenced.
	require.Equal(t, batchSequenced, next.state)

	// No batches have been marked importable.
	require.Equal(t, 0, len(seq.importable()))

	// Mark our batch importable and make sure it shows up in the list of importable batches.
	next.state = batchImportable
	seq.update(next)
	require.Equal(t, 0, len(seq.importable()))
	first := seq.seq[0]
	first.state = batchImportable
	seq.update(first)
	require.Equal(t, 1, len(seq.importable()))
	require.Equal(t, len(seq.seq), seqLen)
	// change the last element back to batchInit so that the importable test stays simple
	last = seq.seq[len(seq.seq)-1]
	last.state = batchInit
	seq.update(last)
	// ensure that the number of importable elements grows as the list is marked importable
	for i := 0; i < len(seq.seq); i++ {
		seq.seq[i].state = batchImportable
		require.Equal(t, i+1, len(seq.importable()))
	}
	// reset everything to init
	for i := 0; i < len(seq.seq); i++ {
		seq.seq[i].state = batchInit
		require.Equal(t, 0, len(seq.importable()))
	}
	// loop backwards and make sure importable is zero until the first element is importable
	for i := len(seq.seq) - 1; i > 0; i-- {
		seq.seq[i].state = batchImportable
		require.Equal(t, 0, len(seq.importable()))
	}
	seq.seq[0].state = batchImportable
	require.Equal(t, len(seq.seq), len(seq.importable()))

	// reset everything to init again
	for i := 0; i < len(seq.seq); i++ {
		seq.seq[i].state = batchInit
		require.Equal(t, 0, len(seq.importable()))
	}
	// set first 3 elements to importable. we should see them in the result for importable()
	// and be able to use update to cycle them away.
	seq.seq[0].state, seq.seq[1].state, seq.seq[2].state = batchImportable, batchImportable, batchImportable
	require.Equal(t, 3, len(seq.importable()))
	a, b, c, z := seq.seq[0], seq.seq[1], seq.seq[2], seq.seq[3]
	require.NotEqual(t, z.begin, seq.seq[2].begin)
	require.NotEqual(t, z.begin, seq.seq[1].begin)
	require.NotEqual(t, z.begin, seq.seq[0].begin)
	a.state, b.state, c.state = batchImportComplete, batchImportComplete, batchImportComplete
	seq.update(a)

	// follow z as it moves down  the chain to the first spot
	require.Equal(t, z.begin, seq.seq[2].begin)
	require.NotEqual(t, z.begin, seq.seq[1].begin)
	require.NotEqual(t, z.begin, seq.seq[0].begin)
	seq.update(b)
	require.NotEqual(t, z.begin, seq.seq[2].begin)
	require.Equal(t, z.begin, seq.seq[1].begin)
	require.NotEqual(t, z.begin, seq.seq[0].begin)
	seq.update(c)
	require.NotEqual(t, z.begin, seq.seq[2].begin)
	require.NotEqual(t, z.begin, seq.seq[1].begin)
	require.Equal(t, z.begin, seq.seq[0].begin)

	// Check integrity of begin/end alignment across the sequence.
	// Also update all the states to sequenced for the convenience of the next test.
	for i := 1; i < len(seq.seq); i++ {
		require.Equal(t, seq.seq[i].end, seq.seq[i-1].begin)
		// won't touch the first element, which is fine because it is marked complete below.
		seq.seq[i].state = batchSequenced
	}

	// set the min for the batcher close to the lowest slot. This will force the next batch to be partial and the batch
	// after that to be the final batch.
	newMin := seq.seq[len(seq.seq)-1].begin - 30
	seq.batcher.min = newMin
	first = seq.seq[0]
	first.state = batchImportComplete
	// update() with a complete state will cause the sequence to be extended with an additional batch
	seq.update(first)
	lastS, err := seq.sequence()
	last = lastS[0]
	require.NoError(t, err)
	require.Equal(t, newMin, last.begin)
	require.Equal(t, seq.seq[len(seq.seq)-2].begin, last.end)

	// Mark first batch done again, this time check that sequence() gives errEndSequence.
	first = seq.seq[0]
	first.state = batchImportComplete
	// update() with a complete state will cause the sequence to be extended with an additional batch
	seq.update(first)
	endExp, err := seq.sequence()
	require.NoError(t, err)
	require.Equal(t, 1, len(endExp))
	end := endExp[0]
	//require.ErrorIs(t, err, errEndSequence)
	require.Equal(t, batchEndSequence, end.state)
}
