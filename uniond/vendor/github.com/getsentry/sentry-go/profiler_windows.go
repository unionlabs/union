package sentry

import (
	"sync"
	"syscall"
)

// This works around the ticker resolution on Windows being ~15ms by default.
// See https://github.com/golang/go/issues/44343
func setTimeTickerResolution() {
	var winmmDLL = syscall.NewLazyDLL("winmm.dll")
	if winmmDLL != nil {
		var timeBeginPeriod = winmmDLL.NewProc("timeBeginPeriod")
		if timeBeginPeriod != nil {
			timeBeginPeriod.Call(uintptr(1))
		}
	}
}

var runOnce sync.Once

func onProfilerStart() {
	runOnce.Do(setTimeTickerResolution)
}
