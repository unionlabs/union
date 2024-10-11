// Package async includes helpers for scheduling runnable, periodic functions and contains useful helpers for converting multi-processor computation.
package async

import (
	"context"
	"reflect"
	"runtime"
	"time"

	log "github.com/sirupsen/logrus"
)

// RunEvery runs the provided command periodically.
// It runs in a goroutine, and can be cancelled by finishing the supplied context.
func RunEvery(ctx context.Context, period time.Duration, f func()) {
	funcName := runtime.FuncForPC(reflect.ValueOf(f).Pointer()).Name()
	ticker := time.NewTicker(period)
	go func() {
		for {
			select {
			case <-ticker.C:
				log.WithField("function", funcName).Trace("running")
				f()
			case <-ctx.Done():
				log.WithField("function", funcName).Debug("context is closed, exiting")
				ticker.Stop()
				return
			}
		}
	}()
}
