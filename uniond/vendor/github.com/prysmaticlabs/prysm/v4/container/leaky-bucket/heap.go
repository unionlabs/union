package leakybucket

import "fmt"

// Based on the example implementation of priority queue found in the
// container/heap package docs: https://golang.org/pkg/container/heap/
type priorityQueue []*LeakyBucket

func (pq priorityQueue) Len() int {
	return len(pq)
}

func (pq priorityQueue) Peak() *LeakyBucket {
	if len(pq) <= 0 {
		return nil
	}
	return pq[0]
}

func (pq priorityQueue) Less(i, j int) bool {
	return pq[i].p.Before(pq[j].p)
}

func (pq priorityQueue) Swap(i, j int) {
	pq[i], pq[j] = pq[j], pq[i]
	pq[i].index = i
	pq[j].index = j
}

func (pq *priorityQueue) Push(x interface{}) {
	n := len(*pq)
	b, ok := x.(*LeakyBucket)
	if !ok {
		panic(fmt.Sprintf("%T", x))
	}
	b.index = n
	*pq = append(*pq, b)
}

func (pq *priorityQueue) Pop() interface{} {
	old := *pq
	n := len(old)
	b := old[n-1]
	b.index = -1 // for safety
	*pq = old[0 : n-1]
	return b
}
