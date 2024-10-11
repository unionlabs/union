package async

import (
	"context"
	"time"
)

// Debounce events fired over a channel by a specified duration, ensuring no events
// are handled until a certain interval of time has passed.
func Debounce(ctx context.Context, interval time.Duration, eventsChan <-chan interface{}, handler func(interface{})) {
	var timer *time.Timer
	defer func() {
		if timer != nil {
			timer.Stop()
		}
	}()
	for {
		select {
		// Wait until an event is triggered.
		case event := <-eventsChan:
			timer = time.NewTimer(interval)
		loop:
			for {
				// If an event is received, wait the specified interval before calling the handler.
				// If another event is received before the interval has passed, store
				// it and reset the timer.
				select {
				case event = <-eventsChan:
					// Reset timer.
					timer.Stop()
					timer = time.NewTimer(interval)
				case <-timer.C:
					// Stop the current timer, handle the request, and wait for more events.
					timer.Stop()
					handler(event)
					break loop
				case <-ctx.Done():
					return
				}
			}
		case <-ctx.Done():
			return
		}
	}
}
