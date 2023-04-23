package xsync

import (
	"fmt"
)

// Go allows running a function in another goroutine
// and waiting for its error.
func Go(fn func() error) <-chan error {
	errs := make(chan error, 1)
	go func() {
		defer func() {
			r := recover()
			if r != nil {
				select {
				case errs <- fmt.Errorf("panic in go fn: %v", r):
				default:
				}
			}
		}()
		errs <- fn()
	}()

	return errs
}
