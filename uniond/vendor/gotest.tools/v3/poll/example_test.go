package poll_test

import (
	"fmt"
	"time"

	"gotest.tools/v3/poll"
)

var t poll.TestingT

func numOfProcesses() (int, error) {
	return 0, nil
}

func ExampleWaitOn() {
	desired := 10

	check := func(t poll.LogT) poll.Result {
		actual, err := numOfProcesses()
		if err != nil {
			return poll.Error(fmt.Errorf("failed to get number of processes: %w", err))
		}
		if actual == desired {
			return poll.Success()
		}
		t.Logf("waiting on process count to be %d...", desired)
		return poll.Continue("number of processes is %d, not %d", actual, desired)
	}

	poll.WaitOn(t, check)
}

func isDesiredState() bool { return false }
func getState() string     { return "" }

func ExampleSettingOp() {
	check := func(t poll.LogT) poll.Result {
		if isDesiredState() {
			return poll.Success()
		}
		return poll.Continue("state is: %s", getState())
	}
	poll.WaitOn(t, check, poll.WithTimeout(30*time.Second), poll.WithDelay(15*time.Millisecond))
}
