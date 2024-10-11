package timepb

import (
	"fmt"
	"time"

	durpb "google.golang.org/protobuf/types/known/durationpb"
	tspb "google.golang.org/protobuf/types/known/timestamppb"
)

// IsZero returns true only when t is nil
func IsZero(t *tspb.Timestamp) bool {
	return t == nil
}

// Commpare t1 and t2 and returns -1 when t1 < t2, 0 when t1 == t2 and 1 otherwise.
// Returns false if t1 or t2 is nil
func Compare(t1, t2 *tspb.Timestamp) int {
	if t1 == nil || t2 == nil {
		panic(fmt.Sprint("Can't compare nil time, t1=", t1, "t2=", t2))
	}
	if t1.Seconds == t2.Seconds && t1.Nanos == t2.Nanos {
		return 0
	}
	if t1.Seconds < t2.Seconds || t1.Seconds == t2.Seconds && t1.Nanos < t2.Nanos {
		return -1
	}
	return 1
}

// DurationIsNegative returns true if the duration is negative. It assumes that d is valid
// (d..CheckValid() is nil).
func DurationIsNegative(d *durpb.Duration) bool {
	return d.Seconds < 0 || d.Seconds == 0 && d.Nanos < 0
}

// AddStd returns a new timestamp with value t + d, where d is stdlib Duration.
// If t is nil then nil is returned.
// Panics on overflow.
func AddStd(t *tspb.Timestamp, d time.Duration) *tspb.Timestamp {
	if t == nil {
		return nil
	}
	if d == 0 {
		t2 := *t
		return &t2
	}
	t2 := tspb.New(t.AsTime().Add(d))
	overflowPanic(t, t2, d < 0)
	return t2
}

func overflowPanic(t1, t2 *tspb.Timestamp, negative bool) {
	cmp := Compare(t1, t2)
	if negative {
		if cmp < 0 {
			panic("time overflow")
		}
	} else {
		if cmp > 0 {
			panic("time overflow")
		}
	}
}

const second = int32(time.Second)

// Add returns a new timestamp with value t + d, where d is protobuf Duration
// If t is nil then nil is returned. Panics on overflow.
// Note: d must be a valid PB Duration (d..CheckValid() is nil).
func Add(t *tspb.Timestamp, d *durpb.Duration) *tspb.Timestamp {
	if t == nil {
		return nil
	}
	if d.Seconds == 0 && d.Nanos == 0 {
		t2 := *t
		return &t2
	}
	t2 := tspb.Timestamp{
		Seconds: t.Seconds + d.Seconds,
		Nanos:   t.Nanos + d.Nanos,
	}
	if t2.Nanos >= second {
		t2.Nanos -= second
		t2.Seconds++
	} else if t2.Nanos <= -second {
		t2.Nanos += second
		t2.Seconds--
	}
	overflowPanic(t, &t2, DurationIsNegative(d))
	return &t2
}
