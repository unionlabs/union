package leakybucket

import (
	"fmt"
	"os"
	"sync/atomic"
	"testing"
	"time"
)

// Arbitrary start time.
var start = time.Date(1990, 1, 2, 0, 0, 0, 0, time.UTC).Round(0)
var elapsed int64

// We provide atomic access to elapsed to avoid data races between multiple
// concurrent goroutines during the tests.
func getElapsed() time.Duration {
	return time.Duration(atomic.LoadInt64(&elapsed))
}
func setElapsed(v time.Duration) {
	atomic.StoreInt64(&elapsed, int64(v))
}
func addToElapsed(v time.Duration) {
	atomic.AddInt64(&elapsed, int64(v))
}

func reset(t *testing.T, c *Collector) {
	c.Reset()
	setElapsed(0)
}

func TestNewLeakyBucket(t *testing.T) {
	rate := 1.0
	capacity := int64(5)
	b := NewLeakyBucket(rate, capacity, time.Second)

	if b.p != now() {
		t.Fatal("Didn't initialize priority?!")
	}
	if b.rate != rate || b.Rate() != rate {
		t.Fatal("Wrong rate?!")
	}
	if b.capacity != capacity || b.Capacity() != capacity {
		t.Fatal("Wrong capacity?!")
	}
}

type actionSet struct {
	count  int64
	action string
	value  interface{}
}

type testSet struct {
	capacity int64
	rate     float64
	set      []actionSet
}

var oneAtaTime = testSet{
	capacity: 5,
	rate:     1.0,
	set: []actionSet{
		{},
		{1, "add", 1},
		{1, "time-set", time.Nanosecond},
		{1, "till", time.Second - time.Nanosecond},
		{1, "time-set", time.Second - time.Nanosecond},
		{1, "till", time.Nanosecond},
		{0, "time-set", time.Second},
		{0, "till", time.Duration(0)},

		// Add a couple.
		{1, "add", 1},
		{1, "time-add", time.Second / 2},
		{1, "till", time.Second / 2},
		{2, "add", 1},
		{2, "time-add", time.Second/2 - time.Nanosecond},

		// Monkey with the capacity and make sure Count()/TillEmpty() are
		// adjusted as needed.
		{2, "cap", 5 + 1},
		{2, "till", time.Second + time.Nanosecond},
		{2, "cap", 5 - 1},
		{2, "till", time.Second + time.Nanosecond},
		{1, "cap", 1},
		{1, "till", time.Second},
		{1, "cap", 4},
		{1, "till", time.Second},

		// Test the full cases.
		{0, "time-add", time.Second * time.Duration(5)},
		{1, "add", 1},
		{2, "add", 1},
		{3, "add", 1},
		{4, "add", 1},
		{4, "add", 1},
		{4, "till", time.Second * 4},
	},
}

var varied = testSet{
	capacity: 1000,
	rate:     60.0,
	set: []actionSet{
		{},
		{100, "add", 100},
		{100, "time-set", time.Nanosecond},
		{1000, "add", 1000},
		{1000, "add", 1},
		{940, "time-set", time.Second},
	},
}

func runTest(t *testing.T, test *testSet) {
	setElapsed(0)
	b := NewLeakyBucket(test.rate, test.capacity, time.Second)

	for i, v := range test.set {
		switch v.action {
		case "add":
			count := b.Count()
			remaining := test.capacity - count
			amount := int64(v.value.(int))
			n := b.Add(amount)
			if n < amount {
				// The bucket should be full now.
				if n < remaining {
					t.Fatalf("Test %d: Bucket should have been filled by this Add()?!", i)
				}
			}
		case "time-set":
			setElapsed(v.value.(time.Duration))
		case "cap":
			b.ChangeCapacity(int64(v.value.(int)))
			test.capacity = b.Capacity()
		case "time-add":
			addToElapsed(v.value.(time.Duration))
		case "till":
			dt := b.TillEmpty()
			if dt != v.value.(time.Duration) {
				t.Fatalf("Test %d: Bad TillEmpty(). Expected %v, got %v", i, v.value, dt)
			}
		case "debug":
			fmt.Println("elapsed:", getElapsed())
			fmt.Println("tillEmpty:", b.TillEmpty())
			fmt.Println("count:", b.Count())
		}
		c := b.Count()
		if c != v.count {
			t.Fatalf("Test %d: Bad count. Expected %d, got %d", i, v.count, c)
		}
		if c > test.capacity {
			t.Fatalf("Test %d: Went over capacity?!", i)
		}
		if b.Remaining() != test.capacity-v.count {
			t.Fatalf("Test %d: Expected remaining value %d, got %d",
				i, test.capacity-v.count, b.Remaining())
		}
	}
}

func TestLeakyBucket(t *testing.T) {
	tests := []testSet{
		oneAtaTime,
		varied,
	}

	for i, test := range tests {
		fmt.Println("Running testSet:", i)
		runTest(t, &test)
	}
}

func TestMain(m *testing.M) {
	// Override what now() function the leakybucket library uses.
	// This greatly increases testability!
	now = func() time.Time { return start.Add(getElapsed()) }

	os.Exit(m.Run())
}
