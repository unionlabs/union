package pebble

import "github.com/cockroachdb/pebble/internal/base"

// The maximum number of elements in the readCompactions queue.
// We want to limit the number of elements so that we only do
// compactions for ranges which are being read recently.
const readCompactionMaxQueueSize = 5

// The readCompactionQueue is a queue of read compactions with
// 0 overlapping ranges.
type readCompactionQueue struct {
	// Invariant: A contiguous prefix of the queue contains
	// all the elements in the queue, in order of insertion.
	// When we remove duplicates from the queue, we break
	// the invariant that a contiguous prefix of the queue
	// has all the elements in it. To fix this, we shift
	// the elements of the queue to the left. This is cheap
	// because the queue has a max length of 5.
	queue [readCompactionMaxQueueSize]*readCompaction

	// The size of the queue which is occupied.
	// A size of k, implies that the first k elements
	// of the queue are occupied.
	// The size will be <= readCompactionMaxQueueSize.
	size int
}

// combine should be used to combine an older queue with a newer
// queue.
func (qu *readCompactionQueue) combine(newQu *readCompactionQueue, cmp base.Compare) {

	for i := 0; i < newQu.size; i++ {
		qu.add(newQu.queue[i], cmp)
	}
}

// add adds read compactions to the queue, while maintaining the invariant
// that there are no overlapping ranges in the queue.
func (qu *readCompactionQueue) add(rc *readCompaction, cmp base.Compare) {
	sz := qu.size
	for i := 0; i < sz; i++ {
		left := qu.queue[i]
		right := rc
		if cmp(left.start, right.start) > 0 {
			left, right = right, left
		}
		if cmp(right.start, left.end) <= 0 {
			qu.queue[i] = nil
			qu.size--
		}
	}

	// Get rid of the holes which may have been formed
	// in the queue.
	qu.shiftLeft()

	if qu.size == readCompactionMaxQueueSize {
		// Make space at the end.
		copy(qu.queue[0:], qu.queue[1:])
		qu.queue[qu.size-1] = rc
	} else {
		qu.size++
		qu.queue[qu.size-1] = rc
	}
}

// Shifts the non-nil elements of the queue to the left so
// that a continguous prefix of the queue is non-nil.
func (qu *readCompactionQueue) shiftLeft() {
	nilPos := -1
	for i := 0; i < readCompactionMaxQueueSize; i++ {
		if qu.queue[i] == nil && nilPos == -1 {
			nilPos = i
		} else if qu.queue[i] != nil && nilPos != -1 {
			qu.queue[nilPos] = qu.queue[i]
			qu.queue[i] = nil
			nilPos++
		}
	}
}

// remove will remove the oldest element from the queue.
func (qu *readCompactionQueue) remove() *readCompaction {
	if qu.size == 0 {
		return nil
	}

	c := qu.queue[0]
	copy(qu.queue[0:], qu.queue[1:])
	qu.queue[qu.size-1] = nil
	qu.size--
	return c
}
