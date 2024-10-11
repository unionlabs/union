// Copyright 2024 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package fifo

import (
	"context"
	"fmt"
	"runtime"
	"testing"

	"golang.org/x/sync/errgroup"
)

// BenchmarkSemaphore runs benchmarks for the semaphore and for a simple channel
// semaphore as a baseline.
//
// Sample results on an Apple M1:
//
//	Semaphore/W1/C1/Semaphore-10        66.1ns ± 0%
//	Semaphore/W1/C1/Channel-10          91.0ns ± 0%
//	Semaphore/W2/C2/Semaphore-10         119ns ± 3%
//	Semaphore/W2/C2/Channel-10           123ns ± 7%
//	Semaphore/W8/C4/Semaphore-10         448ns ± 1%
//	Semaphore/W8/C4/Channel-10           497ns ± 4%
//	Semaphore/W128/C4/Semaphore-10       433ns ± 1%
//	Semaphore/W128/C4/Channel-10         515ns ±25%
//	Semaphore/W512/C128/Semaphore-10     862ns ±23%
//	Semaphore/W512/C128/Channel-10      1.54µs ±37%
//	Semaphore/W512/C513/Semaphore-10     374ns ±59%
//	Semaphore/W512/C513/Channel-10       376ns ±43%
//	Semaphore/W512/C511/Semaphore-10     499ns ±58%
//	Semaphore/W512/C511/Channel-10       316ns ±66%
//	Semaphore/W1024/C4/Semaphore-10      392ns ± 2%
//	Semaphore/W1024/C4/Channel-10        517ns ±18%
//	Semaphore/W1024/C4096/Semaphore-10   437ns ± 2%
//	Semaphore/W1024/C4096/Channel-10     386ns ±41%
func BenchmarkSemaphore(b *testing.B) {
	specs := []struct {
		workers, capacity int
	}{
		{workers: 1, capacity: 1},
		{workers: 2, capacity: 2},
		{workers: 8, capacity: 4},
		{workers: 128, capacity: 4},
		{workers: 512, capacity: 128},
		{workers: 512, capacity: 513},
		{workers: 512, capacity: 511},
		{workers: 1024, capacity: 4},
		{workers: 1024, capacity: 4096},
	}
	for _, s := range specs {
		b.Run(fmt.Sprintf("W%d/C%d", s.workers, s.capacity), func(b *testing.B) {
			b.Run("Semaphore", func(b *testing.B) {
				benchmarkSemaphore(b, s.capacity, s.workers)
			})
			b.Run("Channel", func(b *testing.B) {
				benchmarkChannelSem(b, s.capacity, s.workers)
			})
		})
	}
}

func benchmarkSemaphore(b *testing.B, capacity, workers int) {
	sem := NewSemaphore(int64(capacity))
	g, ctx := errgroup.WithContext(context.Background())
	runWorker := func(workerNum int) {
		g.Go(func() error {
			for i := workerNum; i < b.N; i += workers {
				err := sem.Acquire(ctx, 1)
				if err != nil {
					return err
				}
				runtime.Gosched()
				sem.Release(1)
			}
			return nil
		})
	}
	for i := 0; i < workers; i++ {
		runWorker(i)
	}
	if err := g.Wait(); err != nil {
		b.Fatal(err)
	}
}

func benchmarkChannelSem(b *testing.B, capacity, workers int) {
	sem := make(chan struct{}, capacity)
	g, ctx := errgroup.WithContext(context.Background())
	runWorker := func(workerNum int) {
		g.Go(func() error {
			for i := workerNum; i < b.N; i += workers {
				select {
				case <-ctx.Done():
				case sem <- struct{}{}:
				}
				runtime.Gosched()
				<-sem
			}
			return nil
		})
	}
	for i := 0; i < workers; i++ {
		runWorker(i)
	}
	if err := g.Wait(); err != nil {
		b.Fatal(err)
	}
	close(sem)
}
