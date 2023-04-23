# Go Timer implementation with a fixed Reset behavior

[![GoDoc](https://godoc.org/github.com/desertbit/timer?status.svg)](https://godoc.org/github.com/desertbit/timer)
[![Go Report Card](https://goreportcard.com/badge/github.com/desertbit/timer)](https://goreportcard.com/report/github.com/desertbit/timer)

This is a lightweight timer implementation which is a drop-in replacement for
Go's Timer. Reset behaves as one would expect and drains the timer.C channel automatically.
The core design of this package is similar to the original runtime timer implementation.

These two lines are equivalent except for saving some garbage:

```go
t.Reset(x)

t := timer.NewTimer(x)
```

See issues:
- https://github.com/golang/go/issues/11513
- https://github.com/golang/go/issues/14383
- https://github.com/golang/go/issues/12721
- https://github.com/golang/go/issues/14038
- https://groups.google.com/forum/#!msg/golang-dev/c9UUfASVPoU/tlbK2BpFEwAJ
- http://grokbase.com/t/gg/golang-nuts/1571eh3tv7/go-nuts-reusing-time-timer

Quote from the [Timer Go doc reference](https://golang.org/pkg/time/#Timer):

>Reset changes the timer to expire after duration d.
It returns true if the timer had been active, false if the timer had
expired or been stopped.

> To reuse an active timer, always call its Stop method first and—if it had
expired—drain the value from its channel. For example: [...]
This should not be done concurrent to other receives from the Timer's channel.

> Note that it is not possible to use Reset's return value correctly, as there
is a race condition between draining the channel and the new timer expiring.
Reset should always be used in concert with Stop, as described above.
The return value exists to preserve compatibility with existing programs.

## Broken behavior sample

### Sample 1

```go
package main

import (
    "log"
    "time"
)

func main() {
	start := time.Now()

	// Start a new timer with a timeout of 1 second.
	timer := time.NewTimer(1 * time.Second)

	// Wait for 2 seconds.
	// Meanwhile the timer fired and filled the channel.
	time.Sleep(2 * time.Second)

	// Reset the timer. This should act exactly as creating a new timer.
	timer.Reset(1 * time.Second)

	// However this will fire immediately, because the channel was not drained.
	// See issue: https://github.com/golang/go/issues/11513
	<-timer.C

	if int(time.Since(start).Seconds()) != 3 {
		log.Fatalf("took ~%v seconds, should be ~3 seconds\n", int(time.Since(start).Seconds()))
	}
}
```

### Sample 2

```go
package main

import "time"

const (
	keepaliveInterval = 2 * time.Millisecond
)

var (
	resetC = make(chan struct{}, 1)
)

func main() {
	go keepaliveLoop()

	// Sample routine triggering the reset.
	// Example: this could be due to incoming peer requests and
	// a keepalive check should be reset to the max keepalive timeout.
	for i := 0; i < 1000; i++ {
		time.Sleep(time.Millisecond)
		resetKeepalive()
	}
}

func resetKeepalive() {
	// Don't block if there is already a reset request.
	select {
	case resetC <- struct{}{}:
	default:
	}
}

func keepaliveLoop() {
	t := time.NewTimer(keepaliveInterval)

	for {
		select {
		case <-resetC:
			time.Sleep(3 * time.Millisecond) // Simulate some reset work...
			t.Reset(keepaliveInterval)
		case <-t.C:
			ping()
			t.Reset(keepaliveInterval)
		}
	}
}

func ping() {
	panic("ping must not be called in this example")
}
```
