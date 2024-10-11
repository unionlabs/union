package consensus

import (
	"time"

	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/service"
)

var tickTockBufferSize = 10

// TimeoutTicker is a timer that schedules timeouts
// conditional on the height/round/step in the timeoutInfo.
// The timeoutInfo.Duration may be non-positive.
type TimeoutTicker interface {
	Start() error
	Stop() error
	Chan() <-chan timeoutInfo       // on which to receive a timeout
	ScheduleTimeout(ti timeoutInfo) // reset the timer

	SetLogger(l log.Logger)
}

// timeoutTicker wraps time.Timer,
// scheduling timeouts only for greater height/round/step
// than what it's already seen.
// Timeouts are scheduled along the tickChan,
// and fired on the tockChan.
type timeoutTicker struct {
	service.BaseService

	timerActive bool
	timer       *time.Timer
	tickChan    chan timeoutInfo // for scheduling timeouts
	tockChan    chan timeoutInfo // for notifying about them
}

// NewTimeoutTicker returns a new TimeoutTicker.
func NewTimeoutTicker() TimeoutTicker {
	tt := &timeoutTicker{
		timer: time.NewTimer(0),
		// An indicator variable to check if the timer is active or not.
		// Concurrency safe because the timer is only accessed by a single goroutine.
		timerActive: true,
		tickChan:    make(chan timeoutInfo, tickTockBufferSize),
		tockChan:    make(chan timeoutInfo, tickTockBufferSize),
	}
	tt.BaseService = *service.NewBaseService(nil, "TimeoutTicker", tt)
	tt.stopTimer() // don't want to fire until the first scheduled timeout
	return tt
}

// OnStart implements service.Service. It starts the timeout routine.
func (t *timeoutTicker) OnStart() error {
	go t.timeoutRoutine()

	return nil
}

// OnStop implements service.Service. It stops the timeout routine.
func (t *timeoutTicker) OnStop() {
	t.BaseService.OnStop()
}

// Chan returns a channel on which timeouts are sent.
func (t *timeoutTicker) Chan() <-chan timeoutInfo {
	return t.tockChan
}

// ScheduleTimeout schedules a new timeout by sending on the internal tickChan.
// The timeoutRoutine is always available to read from tickChan, so this won't block.
// The scheduling may fail if the timeoutRoutine has already scheduled a timeout for a later height/round/step.
func (t *timeoutTicker) ScheduleTimeout(ti timeoutInfo) {
	t.tickChan <- ti
}

// -------------------------------------------------------------

// if the timer is active, stop it and drain the channel.
func (t *timeoutTicker) stopTimer() {
	if !t.timerActive {
		return
	}
	// Stop() returns false if it was already fired or was stopped
	if !t.timer.Stop() {
		<-t.timer.C
	}
	t.timerActive = false
}

// send on tickChan to start a new timer.
// timers are interrupted and replaced by new ticks from later steps
// timeouts of 0 on the tickChan will be immediately relayed to the tockChan.
// NOTE: timerActive is not concurrency safe, but it's only accessed in NewTimer and timeoutRoutine,
// making it single-threaded access.
func (t *timeoutTicker) timeoutRoutine() {
	t.Logger.Debug("Starting timeout routine")
	var ti timeoutInfo
	for {
		select {
		case newti := <-t.tickChan:
			t.Logger.Debug("Received tick", "old_ti", ti, "new_ti", newti)

			if shouldSkipTick(newti, ti) {
				continue
			}

			// stop the last timer if it exists
			t.stopTimer()

			// update timeoutInfo, reset timer, and mark timer as active
			ti = newti
			t.timer.Reset(ti.Duration)
			t.timerActive = true

			t.Logger.Debug("Scheduled timeout", "dur", ti.Duration, "height", ti.Height, "round", ti.Round, "step", ti.Step)
		case <-t.timer.C:
			t.timerActive = false
			t.Logger.Info("Timed out", "dur", ti.Duration, "height", ti.Height, "round", ti.Round, "step", ti.Step)
			// go routine here guarantees timeoutRoutine doesn't block.
			// Determinism comes from playback in the receiveRoutine.
			// We can eliminate it by merging the timeoutRoutine into receiveRoutine
			//  and managing the timeouts ourselves with a millisecond ticker
			go func(toi timeoutInfo) { t.tockChan <- toi }(ti)
		case <-t.Quit():
			t.stopTimer()
			return
		}
	}
}

// shouldSkipTick returns true if the new timeoutInfo should be skipped.
func shouldSkipTick(newti, ti timeoutInfo) bool {
	if newti.Height < ti.Height {
		return true
	}
	if newti.Height == ti.Height && ((newti.Round < ti.Round) || (newti.Round == ti.Round && ti.Step > 0 && newti.Step <= ti.Step)) {
		return true
	}
	return false
}
