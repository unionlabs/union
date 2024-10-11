package leakybucket

import (
	"testing"
	"time"
)

func TestLen(t *testing.T) {
	q := make(priorityQueue, 0, 4096)

	if q.Len() != 0 {
		t.Fatal("Queue should be empty?!")
	}

	for i := 1; i <= 5; i++ {
		b := NewLeakyBucket(1.0, 5, time.Second)
		q.Push(b)

		l := q.Len()
		if l != i {
			t.Fatalf("Expected length %d, got %d", i, l)
		}
	}
	for i := 4; i >= 0; i-- {
		q.Pop()

		l := q.Len()
		if l != i {
			t.Fatalf("Expected length %d, got %d", i, l)
		}
	}
}

func TestPeak(t *testing.T) {
	q := make(priorityQueue, 0, 4096)

	for i := 0; i < 5; i++ {
		b := NewLeakyBucket(1.0, 5, time.Second)
		q.Push(b)
	}
}

func TestLess(t *testing.T) {
	q := make(priorityQueue, 0, 4096)

	for i := 0; i < 5; i++ {
		b := NewLeakyBucket(1.0, 5, time.Second)
		b.p = now().Add(time.Duration(i))
		q.Push(b)
	}

	for i, j := 0, 4; i < 5; i, j = i+1, j-1 {
		if i < j && !q.Less(i, j) {
			t.Fatal("Less is more?!")
		}
	}
}

func TestSwap(t *testing.T) {
	q := make(priorityQueue, 0, 4096)

	for i := 0; i < 5; i++ {
		b := NewLeakyBucket(1.0, 5, time.Second)
		q.Push(b)
	}

	i := 2
	j := 4

	bi := q[i]
	bj := q[j]

	q.Swap(i, j)

	if bi != q[j] || bj != q[i] {
		t.Fatal("Element weren't swapped?!")
	}
}

func TestPush(t *testing.T) {
	q := make(priorityQueue, 0, 4096)

	for i := 0; i < 5; i++ {
		b := NewLeakyBucket(1.0, 5, time.Second)
		q.Push(b)

		if b != q[len(q)-1] {
			t.Fatal("Push should append to queue.")
		}
	}
}

func TestPop(t *testing.T) {
	q := make(priorityQueue, 0, 4096)

	for i := 1; i <= 5; i++ {
		b := NewLeakyBucket(1.0, 5, time.Second)
		q.Push(b)
	}

	for i := 1; i <= 5; i++ {
		b := q[len(q)-1]
		if b != q.Pop() {
			t.Fatal("Pop should remove from end of queue.")
		}
	}
}
