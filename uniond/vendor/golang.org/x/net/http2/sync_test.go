// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package http2

import (
	"context"
	"fmt"
	"runtime"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"
)

// A synctestGroup synchronizes between a set of cooperating goroutines.
type synctestGroup struct {
	mu     sync.Mutex
	gids   map[int]bool
	now    time.Time
	timers map[*fakeTimer]struct{}
}

type goroutine struct {
	id     int
	parent int
	state  string
}

// newSynctest creates a new group with the synthetic clock set the provided time.
func newSynctest(now time.Time) *synctestGroup {
	return &synctestGroup{
		gids: map[int]bool{
			currentGoroutine(): true,
		},
		now: now,
	}
}

// Join adds the current goroutine to the group.
func (g *synctestGroup) Join() {
	g.mu.Lock()
	defer g.mu.Unlock()
	g.gids[currentGoroutine()] = true
}

// Count returns the number of goroutines in the group.
func (g *synctestGroup) Count() int {
	gs := stacks(true)
	count := 0
	for _, gr := range gs {
		if !g.gids[gr.id] && !g.gids[gr.parent] {
			continue
		}
		count++
	}
	return count
}

// Close calls t.Fatal if the group contains any running goroutines.
func (g *synctestGroup) Close(t testing.TB) {
	if count := g.Count(); count != 1 {
		buf := make([]byte, 16*1024)
		n := runtime.Stack(buf, true)
		t.Logf("stacks:\n%s", buf[:n])
		t.Fatalf("%v goroutines still running after test completed, expect 1", count)
	}
}

// Wait blocks until every goroutine in the group and their direct children are idle.
func (g *synctestGroup) Wait() {
	for i := 0; ; i++ {
		if g.idle() {
			return
		}
		runtime.Gosched()
	}
}

func (g *synctestGroup) idle() bool {
	gs := stacks(true)
	g.mu.Lock()
	defer g.mu.Unlock()
	for _, gr := range gs[1:] {
		if !g.gids[gr.id] && !g.gids[gr.parent] {
			continue
		}
		// From runtime/runtime2.go.
		switch gr.state {
		case "IO wait":
		case "chan receive (nil chan)":
		case "chan send (nil chan)":
		case "select":
		case "select (no cases)":
		case "chan receive":
		case "chan send":
		case "sync.Cond.Wait":
		case "sync.Mutex.Lock":
		case "sync.RWMutex.RLock":
		case "sync.RWMutex.Lock":
		default:
			return false
		}
	}
	return true
}

func currentGoroutine() int {
	s := stacks(false)
	return s[0].id
}

func stacks(all bool) []goroutine {
	buf := make([]byte, 16*1024)
	for {
		n := runtime.Stack(buf, all)
		if n < len(buf) {
			buf = buf[:n]
			break
		}
		buf = make([]byte, len(buf)*2)
	}

	var goroutines []goroutine
	for _, gs := range strings.Split(string(buf), "\n\n") {
		skip, rest, ok := strings.Cut(gs, "goroutine ")
		if skip != "" || !ok {
			panic(fmt.Errorf("1 unparsable goroutine stack:\n%s", gs))
		}
		ids, rest, ok := strings.Cut(rest, " [")
		if !ok {
			panic(fmt.Errorf("2 unparsable goroutine stack:\n%s", gs))
		}
		id, err := strconv.Atoi(ids)
		if err != nil {
			panic(fmt.Errorf("3 unparsable goroutine stack:\n%s", gs))
		}
		state, rest, ok := strings.Cut(rest, "]")
		var parent int
		_, rest, ok = strings.Cut(rest, "\ncreated by ")
		if ok && strings.Contains(rest, " in goroutine ") {
			_, rest, ok := strings.Cut(rest, " in goroutine ")
			if !ok {
				panic(fmt.Errorf("4 unparsable goroutine stack:\n%s", gs))
			}
			parents, rest, ok := strings.Cut(rest, "\n")
			if !ok {
				panic(fmt.Errorf("5 unparsable goroutine stack:\n%s", gs))
			}
			parent, err = strconv.Atoi(parents)
			if err != nil {
				panic(fmt.Errorf("6 unparsable goroutine stack:\n%s", gs))
			}
		}
		goroutines = append(goroutines, goroutine{
			id:     id,
			parent: parent,
			state:  state,
		})
	}
	return goroutines
}

// AdvanceTime advances the synthetic clock by d.
func (g *synctestGroup) AdvanceTime(d time.Duration) {
	defer g.Wait()
	g.mu.Lock()
	defer g.mu.Unlock()
	g.now = g.now.Add(d)
	for tm := range g.timers {
		if tm.when.After(g.now) {
			continue
		}
		tm.run()
		delete(g.timers, tm)
	}
}

// Now returns the current synthetic time.
func (g *synctestGroup) Now() time.Time {
	g.mu.Lock()
	defer g.mu.Unlock()
	return g.now
}

// TimeUntilEvent returns the amount of time until the next scheduled timer.
func (g *synctestGroup) TimeUntilEvent() (d time.Duration, scheduled bool) {
	g.mu.Lock()
	defer g.mu.Unlock()
	for tm := range g.timers {
		if dd := tm.when.Sub(g.now); !scheduled || dd < d {
			d = dd
			scheduled = true
		}
	}
	return d, scheduled
}

// Sleep is time.Sleep, but using synthetic time.
func (g *synctestGroup) Sleep(d time.Duration) {
	tm := g.NewTimer(d)
	<-tm.C()
}

// NewTimer is time.NewTimer, but using synthetic time.
func (g *synctestGroup) NewTimer(d time.Duration) Timer {
	return g.addTimer(d, &fakeTimer{
		ch: make(chan time.Time),
	})
}

// AfterFunc is time.AfterFunc, but using synthetic time.
func (g *synctestGroup) AfterFunc(d time.Duration, f func()) Timer {
	return g.addTimer(d, &fakeTimer{
		f: f,
	})
}

// ContextWithTimeout is context.WithTimeout, but using synthetic time.
func (g *synctestGroup) ContextWithTimeout(ctx context.Context, d time.Duration) (context.Context, context.CancelFunc) {
	ctx, cancel := context.WithCancel(ctx)
	tm := g.AfterFunc(d, cancel)
	return ctx, func() {
		tm.Stop()
		cancel()
	}
}

func (g *synctestGroup) addTimer(d time.Duration, tm *fakeTimer) *fakeTimer {
	g.mu.Lock()
	defer g.mu.Unlock()
	tm.g = g
	tm.when = g.now.Add(d)
	if g.timers == nil {
		g.timers = make(map[*fakeTimer]struct{})
	}
	if tm.when.After(g.now) {
		g.timers[tm] = struct{}{}
	} else {
		tm.run()
	}
	return tm
}

type Timer = interface {
	C() <-chan time.Time
	Reset(d time.Duration) bool
	Stop() bool
}

type fakeTimer struct {
	g    *synctestGroup
	when time.Time
	ch   chan time.Time
	f    func()
}

func (tm *fakeTimer) run() {
	if tm.ch != nil {
		tm.ch <- tm.g.now
	} else {
		go func() {
			tm.g.Join()
			tm.f()
		}()
	}
}

func (tm *fakeTimer) C() <-chan time.Time { return tm.ch }

func (tm *fakeTimer) Reset(d time.Duration) bool {
	tm.g.mu.Lock()
	defer tm.g.mu.Unlock()
	_, stopped := tm.g.timers[tm]
	if d <= 0 {
		delete(tm.g.timers, tm)
		tm.run()
	} else {
		tm.when = tm.g.now.Add(d)
		tm.g.timers[tm] = struct{}{}
	}
	return stopped
}

func (tm *fakeTimer) Stop() bool {
	tm.g.mu.Lock()
	defer tm.g.mu.Unlock()
	_, stopped := tm.g.timers[tm]
	delete(tm.g.timers, tm)
	return stopped
}
