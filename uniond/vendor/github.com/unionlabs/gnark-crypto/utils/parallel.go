package utils

import (
	"runtime"
	"sync"
)

// Would normally put this in internal/parallel; but it may be desirable for it to be accessible from gnark

type Task func(start, end int)

type job struct {
	start, end int
	task       Task
	done       *sync.WaitGroup
}

type WorkerPool struct {
	chJobs    chan job
	nbWorkers int
}

func NewWorkerPool() *WorkerPool {
	p := &WorkerPool{}
	p.nbWorkers = runtime.NumCPU() + 2
	p.chJobs = make(chan job, 40*p.nbWorkers)
	for i := 0; i < p.nbWorkers; i++ {
		go func() {
			for j := range p.chJobs {
				j.task(j.start, j.end)
				j.done.Done()
			}
		}()
	}
	return p
}

// Stop (but does not wait) the pool. It frees the worker.
func (wp *WorkerPool) Stop() {
	close(wp.chJobs)
}

func (wp *WorkerPool) Submit(n int, work func(int, int), minBlock int) *sync.WaitGroup {
	var wg sync.WaitGroup

	// we have an interval [0,n)
	// that we split in minBlock sizes.
	for start := 0; start < n; start += minBlock {
		start := start
		end := start + minBlock
		if end > n {
			end = n
		}
		wg.Add(1)
		wp.chJobs <- job{
			task:  work,
			start: start,
			end:   end,
			done:  &wg,
		}
	}

	return &wg
}
