// Copyright 2019 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package mclock

import (
	"testing"
	"time"
)

var _ Clock = System{}
var _ Clock = new(Simulated)

func TestSimulatedAfter(t *testing.T) {
	var (
		timeout = 30 * time.Minute
		offset  = 99 * time.Hour
		adv     = 11 * time.Minute
		c       Simulated
	)
	c.Run(offset)

	end := c.Now().Add(timeout)
	ch := c.After(timeout)
	for c.Now() < end.Add(-adv) {
		c.Run(adv)
		select {
		case <-ch:
			t.Fatal("Timer fired early")
		default:
		}
	}

	c.Run(adv)
	select {
	case stamp := <-ch:
		want := AbsTime(0).Add(offset).Add(timeout)
		if stamp != want {
			t.Errorf("Wrong time sent on timer channel: got %v, want %v", stamp, want)
		}
	default:
		t.Fatal("Timer didn't fire")
	}
}

func TestSimulatedAfterFunc(t *testing.T) {
	var c Simulated

	called1 := false
	timer1 := c.AfterFunc(100*time.Millisecond, func() { called1 = true })
	if c.ActiveTimers() != 1 {
		t.Fatalf("%d active timers, want one", c.ActiveTimers())
	}
	if fired := timer1.Stop(); !fired {
		t.Fatal("Stop returned false even though timer didn't fire")
	}
	if c.ActiveTimers() != 0 {
		t.Fatalf("%d active timers, want zero", c.ActiveTimers())
	}
	if called1 {
		t.Fatal("timer 1 called")
	}
	if fired := timer1.Stop(); fired {
		t.Fatal("Stop returned true after timer was already stopped")
	}

	called2 := false
	timer2 := c.AfterFunc(100*time.Millisecond, func() { called2 = true })
	c.Run(50 * time.Millisecond)
	if called2 {
		t.Fatal("timer 2 called")
	}
	c.Run(51 * time.Millisecond)
	if !called2 {
		t.Fatal("timer 2 not called")
	}
	if fired := timer2.Stop(); fired {
		t.Fatal("Stop returned true after timer has fired")
	}
}

func TestSimulatedSleep(t *testing.T) {
	var (
		c       Simulated
		timeout = 1 * time.Hour
		done    = make(chan AbsTime, 1)
	)
	go func() {
		c.Sleep(timeout)
		done <- c.Now()
	}()

	c.WaitForTimers(1)
	c.Run(2 * timeout)
	select {
	case stamp := <-done:
		want := AbsTime(2 * timeout)
		if stamp != want {
			t.Errorf("Wrong time after sleep: got %v, want %v", stamp, want)
		}
	case <-time.After(5 * time.Second):
		t.Fatal("Sleep didn't return in time")
	}
}

func TestSimulatedTimerReset(t *testing.T) {
	var (
		c       Simulated
		timeout = 1 * time.Hour
	)
	timer := c.NewTimer(timeout)
	c.Run(2 * timeout)
	select {
	case ftime := <-timer.C():
		if ftime != AbsTime(timeout) {
			t.Fatalf("wrong time %v sent on timer channel, want %v", ftime, AbsTime(timeout))
		}
	default:
		t.Fatal("timer didn't fire")
	}

	timer.Reset(timeout)
	c.Run(2 * timeout)
	select {
	case ftime := <-timer.C():
		if ftime != AbsTime(3*timeout) {
			t.Fatalf("wrong time %v sent on timer channel, want %v", ftime, AbsTime(3*timeout))
		}
	default:
		t.Fatal("timer didn't fire again")
	}
}

func TestSimulatedTimerStop(t *testing.T) {
	var (
		c       Simulated
		timeout = 1 * time.Hour
	)
	timer := c.NewTimer(timeout)
	c.Run(2 * timeout)
	if timer.Stop() {
		t.Errorf("Stop returned true for fired timer")
	}
	select {
	case <-timer.C():
	default:
		t.Fatal("timer didn't fire")
	}
}
