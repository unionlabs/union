package eureka

import (
	"testing"
	"time"

	"github.com/hudl/fargo"

	"github.com/go-kit/kit/sd"
)

var _ sd.Instancer = (*Instancer)(nil) // API check

func TestInstancer(t *testing.T) {
	connection := &testConnection{
		instances:      []*fargo.Instance{instanceTest1, instanceTest2},
		errApplication: nil,
	}

	instancer := NewInstancer(connection, appNameTest, loggerTest)
	defer instancer.Stop()

	state := instancer.state()
	if state.Err != nil {
		t.Fatal(state.Err)
	}

	if want, have := 2, len(state.Instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}

func TestInstancerReceivesUpdates(t *testing.T) {
	connection := &testConnection{
		instances:      []*fargo.Instance{instanceTest1},
		errApplication: nil,
	}

	instancer := NewInstancer(connection, appNameTest, loggerTest)
	defer instancer.Stop()

	verifyCount := func(want int) (have int, converged bool) {
		const maxPollAttempts = 5
		const delayPerAttempt = 200 * time.Millisecond
		for i := 1; ; i++ {
			state := instancer.state()
			if have := len(state.Instances); want == have {
				return have, true
			} else if i == maxPollAttempts {
				return have, false
			}
			time.Sleep(delayPerAttempt)
		}
	}

	if have, converged := verifyCount(1); !converged {
		t.Fatalf("initial: want %d, have %d", 1, have)
	}

	if err := connection.RegisterInstance(instanceTest2); err != nil {
		t.Fatalf("failed to register an instance: %v", err)
	}
	if have, converged := verifyCount(2); !converged {
		t.Fatalf("after registration: want %d, have %d", 2, have)
	}

	if err := connection.DeregisterInstance(instanceTest1); err != nil {
		t.Fatalf("failed to unregister an instance: %v", err)
	}
	if have, converged := verifyCount(1); !converged {
		t.Fatalf("after deregistration: want %d, have %d", 1, have)
	}
}

func TestBadInstancerScheduleUpdates(t *testing.T) {
	connection := &testConnection{
		instances:      []*fargo.Instance{instanceTest1},
		errApplication: errTest,
	}

	instancer := NewInstancer(connection, appNameTest, loggerTest)
	defer instancer.Stop()

	state := instancer.state()
	if state.Err == nil {
		t.Fatal("expecting error")
	}

	if want, have := 0, len(state.Instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}
