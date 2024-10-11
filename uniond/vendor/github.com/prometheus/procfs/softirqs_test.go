// Copyright 2022 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package procfs

import "testing"

func TestSoftirqs(t *testing.T) {
	s, err := getProcFixtures(t).Softirqs()
	if err != nil {
		t.Fatal(err)
	}

	// hi
	if want, have := uint64(3), s.Hi[0]; want != have {
		t.Errorf("want softirq HI count %d, have %d", want, have)
	}
	// timer
	if want, have := uint64(247490), s.Timer[1]; want != have {
		t.Errorf("want softirq TIMER count %d, have %d", want, have)
	}
	// net_tx
	if want, have := uint64(2419), s.NetTx[0]; want != have {
		t.Errorf("want softirq NET_TX count %d, have %d", want, have)
	}
	// net_rx
	if want, have := uint64(28694), s.NetRx[1]; want != have {
		t.Errorf("want softirq NET_RX count %d, have %d", want, have)
	}
	// block
	if want, have := uint64(262755), s.Block[1]; want != have {
		t.Errorf("want softirq BLOCK count %d, have %d", want, have)
	}
	// irq_poll
	if want, have := uint64(0), s.IRQPoll[0]; want != have {
		t.Errorf("want softirq IRQ_POLL count %d, have %d", want, have)
	}
	// tasklet
	if want, have := uint64(209), s.Tasklet[0]; want != have {
		t.Errorf("want softirq TASKLET count %d, have %d", want, have)
	}
	// sched
	if want, have := uint64(2278692), s.Sched[0]; want != have {
		t.Errorf("want softirq SCHED count %d, have %d", want, have)
	}
	// hrtimer
	if want, have := uint64(1281), s.HRTimer[0]; want != have {
		t.Errorf("want softirq HRTIMER count %d, have %d", want, have)
	}
	// rcu
	if want, have := uint64(532783), s.RCU[1]; want != have {
		t.Errorf("want softirq RCU count %d, have %d", want, have)
	}
}
