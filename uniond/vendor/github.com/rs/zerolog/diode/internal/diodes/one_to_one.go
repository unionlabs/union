package diodes

import (
	"sync/atomic"
	"unsafe"
)

// GenericDataType is the data type the diodes operate on.
type GenericDataType unsafe.Pointer

// Alerter is used to report how many values were overwritten since the
// last write.
type Alerter interface {
	Alert(missed int)
}

// AlertFunc type is an adapter to allow the use of ordinary functions as
// Alert handlers.
type AlertFunc func(missed int)

// Alert calls f(missed)
func (f AlertFunc) Alert(missed int) {
	f(missed)
}

type bucket struct {
	data GenericDataType
	seq  uint64 // seq is the recorded write index at the time of writing
}

// OneToOne diode is meant to be used by a single reader and a single writer.
// It is not thread safe if used otherwise.
type OneToOne struct {
	writeIndex uint64
	readIndex  uint64
	buffer     []unsafe.Pointer
	alerter    Alerter
}

// NewOneToOne creates a new diode is meant to be used by a single reader and
// a single writer. The alerter is invoked on the read's go-routine. It is
// called when it notices that the writer go-routine has passed it and wrote
// over data. A nil can be used to ignore alerts.
func NewOneToOne(size int, alerter Alerter) *OneToOne {
	if alerter == nil {
		alerter = AlertFunc(func(int) {})
	}

	return &OneToOne{
		buffer:  make([]unsafe.Pointer, size),
		alerter: alerter,
	}
}

// Set sets the data in the next slot of the ring buffer.
func (d *OneToOne) Set(data GenericDataType) {
	idx := d.writeIndex % uint64(len(d.buffer))

	newBucket := &bucket{
		data: data,
		seq:  d.writeIndex,
	}
	d.writeIndex++

	atomic.StorePointer(&d.buffer[idx], unsafe.Pointer(newBucket))
}

// TryNext will attempt to read from the next slot of the ring buffer.
// If there is no data available, it will return (nil, false).
func (d *OneToOne) TryNext() (data GenericDataType, ok bool) {
	// Read a value from the ring buffer based on the readIndex.
	idx := d.readIndex % uint64(len(d.buffer))
	result := (*bucket)(atomic.SwapPointer(&d.buffer[idx], nil))

	// When the result is nil that means the writer has not had the
	// opportunity to write a value into the diode. This value must be ignored
	// and the read head must not increment.
	if result == nil {
		return nil, false
	}

	// When the seq value is less than the current read index that means a
	// value was read from idx that was previously written but since has
	// been dropped. This value must be ignored and the read head must not
	// increment.
	//
	// The simulation for this scenario assumes the fast forward occurred as
	// detailed below.
	//
	// 5. The reader reads again getting seq 5. It then reads again expecting
	//    seq 6 but gets seq 2. This is a read of a stale value that was
	//    effectively "dropped" so the read fails and the read head stays put.
	//    `| 4 | 5 | 2 | 3 |` r: 7, w: 6
	//
	if result.seq < d.readIndex {
		return nil, false
	}

	// When the seq value is greater than the current read index that means a
	// value was read from idx that overwrote the value that was expected to
	// be at this idx. This happens when the writer has lapped the reader. The
	// reader needs to catch up to the writer so it moves its write head to
	// the new seq, effectively dropping the messages that were not read in
	// between the two values.
	//
	// Here is a simulation of this scenario:
	//
	// 1. Both the read and write heads start at 0.
	//    `| nil | nil | nil | nil |` r: 0, w: 0
	// 2. The writer fills the buffer.
	//    `| 0 | 1 | 2 | 3 |` r: 0, w: 4
	// 3. The writer laps the read head.
	//    `| 4 | 5 | 2 | 3 |` r: 0, w: 6
	// 4. The reader reads the first value, expecting a seq of 0 but reads 4,
	//    this forces the reader to fast forward to 5.
	//    `| 4 | 5 | 2 | 3 |` r: 5, w: 6
	//
	if result.seq > d.readIndex {
		dropped := result.seq - d.readIndex
		d.readIndex = result.seq
		d.alerter.Alert(int(dropped))
	}

	// Only increment read index if a regular read occurred (where seq was
	// equal to readIndex) or a value was read that caused a fast forward
	// (where seq was greater than readIndex).
	d.readIndex++
	return result.data, true
}
