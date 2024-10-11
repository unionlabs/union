// This is an example program that demonstrates an advanced use of the Sentry
// SDK using Hub, Scope and EventProcessor to recover from runtime panics,
// report to Sentry filtering specific frames from the stack trace and then
// letting the program crash as usual.
//
// Try it by running:
//
//	go run main.go
//
// To actually report events to Sentry, set the DSN either by editing the
// appropriate line below or setting the environment variable SENTRY_DSN to
// match the DSN of your Sentry project.
package main

import (
	"fmt"
	"log"
	"math/rand"
	"strings"
	"sync"
	"time"

	"github.com/getsentry/sentry-go"
)

func main() {
	err := sentry.Init(sentry.ClientOptions{
		// Either set your DSN here or set the SENTRY_DSN environment variable.
		Dsn: "",
		// Enable printing of SDK debug messages.
		// Useful when getting started or trying to figure something out.
		Debug: true,
		// This is an optional function with access to the event before it is
		// sent to Sentry. The event can be mutated, or sending can be aborted
		// by returning nil.
		BeforeSend: func(event *sentry.Event, hint *sentry.EventHint) *sentry.Event { return event },
	})
	if err != nil {
		log.Fatalf("sentry.Init: %s", err)
	}
	// Flush buffered events before the program terminates.
	// Set the timeout to the maximum duration the program can afford to wait.
	defer sentry.Flush(2 * time.Second)

	rand.Seed(time.Now().UnixNano())

	nWorkers := 8
	var wg sync.WaitGroup

	// Run some worker goroutines to simulate work.
	for i := 0; i < nWorkers; i++ {
		wg.Add(1)
		go func() {
			// Note that wg.Done() must be outside of RecoverRepanic, otherwise
			// it would unblock wg.Wait() before RecoverRepanic is done doing
			// its job (reporting panic to Sentry).
			defer wg.Done()
			RecoverRepanic(func() {
				// Sleep to simulate some work.
				//#nosec G404 -- We are fine using transparent, non-secure value here.
				time.Sleep(time.Duration(rand.Intn(1000)) * time.Millisecond)
				// Intentionally access an index out of bounds to trigger a runtime
				// panic.
				fmt.Println(make([]int, 3)[3])
			})
		}()
	}

	wg.Wait()
}

// RecoverRepanic calls f and, in case of a runtime panic, reports the panic to
// Sentry and repanics.
//
// Note that if RecoverRepanic is called from multiple goroutines and they panic
// concurrently, then the repanic initiated from RecoverRepanic, unless handled
// further down the call stack, will cause the program to crash without waiting
// for other goroutines to finish their work. That means that most likely only
// the first panic will be successfully reported to Sentry.
func RecoverRepanic(f func()) {
	// Clone the current hub so that modifications of the scope are visible only
	// within this function.
	hub := sentry.CurrentHub().Clone()

	// filterFrames removes frames from outgoing events that reference the
	// RecoverRepanic function and its subfunctions.
	filterFrames := func(event *sentry.Event) {
		for _, e := range event.Exception {
			if e.Stacktrace == nil {
				continue
			}
			frames := e.Stacktrace.Frames[:0]
			for _, frame := range e.Stacktrace.Frames {
				if frame.Module == "main" && strings.HasPrefix(frame.Function, "RecoverRepanic") {
					continue
				}
				frames = append(frames, frame)
			}
			e.Stacktrace.Frames = frames
		}
	}

	// Add an EventProcessor to the scope. The event processor is a function
	// that can change events before they are sent to Sentry.
	// Alternatively, see also ClientOptions.BeforeSend, which is a special
	// event processor applied to error events.
	hub.ConfigureScope(func(scope *sentry.Scope) {
		scope.AddEventProcessor(func(event *sentry.Event, hint *sentry.EventHint) *sentry.Event {
			filterFrames(event)
			return event
		})
	})

	// See https://golang.org/ref/spec#Handling_panics.
	// This will recover from runtime panics and then panic again after
	// reporting to Sentry.
	defer func() {
		if x := recover(); x != nil {
			// Create an event and enqueue it for reporting.
			hub.Recover(x)
			// Because the goroutine running this code is going to crash the
			// program, call Flush to send the event to Sentry before it is too
			// late. Set the timeout to an appropriate value depending on your
			// program. The value is the maximum time to wait before giving up
			// and dropping the event.
			hub.Flush(2 * time.Second)
			// Note that if multiple goroutines panic, possibly only the first
			// one to call Flush will succeed in sending the event. If you want
			// to capture multiple panics and still crash the program
			// afterwards, you need to coordinate error reporting and
			// termination differently.
			panic(x)
		}
	}()

	// Run the original function.
	f()
}
