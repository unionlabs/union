/*
 *
 * Copyright 2021 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package ringhash

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/grpclog"
	igrpclog "google.golang.org/grpc/internal/grpclog"
	"google.golang.org/grpc/internal/testutils"
)

var testSubConns []*testutils.TestSubConn

func init() {
	for i := 0; i < 8; i++ {
		testSubConns = append(testSubConns, testutils.NewTestSubConn(fmt.Sprint(i)))
	}
}

func newTestRing(cStats []connectivity.State) *ring {
	var items []*ringEntry
	for i, st := range cStats {
		testSC := testSubConns[i]
		items = append(items, &ringEntry{
			idx:  i,
			hash: uint64((i + 1) * 10),
			sc: &subConn{
				addr:  testSC.String(),
				sc:    testSC,
				state: st,
			},
		})
	}
	return &ring{items: items}
}

func (s) TestPickerPickFirstTwo(t *testing.T) {
	tests := []struct {
		name            string
		ring            *ring
		hash            uint64
		wantSC          balancer.SubConn
		wantErr         error
		wantSCToConnect balancer.SubConn
	}{
		{
			name:   "picked is Ready",
			ring:   newTestRing([]connectivity.State{connectivity.Ready, connectivity.Idle}),
			hash:   5,
			wantSC: testSubConns[0],
		},
		{
			name:    "picked is connecting, queue",
			ring:    newTestRing([]connectivity.State{connectivity.Connecting, connectivity.Idle}),
			hash:    5,
			wantErr: balancer.ErrNoSubConnAvailable,
		},
		{
			name:            "picked is Idle, connect and queue",
			ring:            newTestRing([]connectivity.State{connectivity.Idle, connectivity.Idle}),
			hash:            5,
			wantErr:         balancer.ErrNoSubConnAvailable,
			wantSCToConnect: testSubConns[0],
		},
		{
			name:   "picked is TransientFailure, next is ready, return",
			ring:   newTestRing([]connectivity.State{connectivity.TransientFailure, connectivity.Ready}),
			hash:   5,
			wantSC: testSubConns[1],
		},
		{
			name:    "picked is TransientFailure, next is connecting, queue",
			ring:    newTestRing([]connectivity.State{connectivity.TransientFailure, connectivity.Connecting}),
			hash:    5,
			wantErr: balancer.ErrNoSubConnAvailable,
		},
		{
			name:            "picked is TransientFailure, next is Idle, connect and queue",
			ring:            newTestRing([]connectivity.State{connectivity.TransientFailure, connectivity.Idle}),
			hash:            5,
			wantErr:         balancer.ErrNoSubConnAvailable,
			wantSCToConnect: testSubConns[1],
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p := newPicker(tt.ring, igrpclog.NewPrefixLogger(grpclog.Component("xds"), "rh_test"))
			got, err := p.Pick(balancer.PickInfo{
				Ctx: SetRequestHash(context.Background(), tt.hash),
			})
			if err != tt.wantErr {
				t.Errorf("Pick() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got.SubConn != tt.wantSC {
				t.Errorf("Pick() got = %v, want picked SubConn: %v", got, tt.wantSC)
			}
			if sc := tt.wantSCToConnect; sc != nil {
				select {
				case <-sc.(*testutils.TestSubConn).ConnectCh:
				case <-time.After(defaultTestShortTimeout):
					t.Errorf("timeout waiting for Connect() from SubConn %v", sc)
				}
			}
		})
	}
}

// TestPickerPickTriggerTFConnect covers that if the picked SubConn is
// TransientFailures, all SubConns until a non-TransientFailure are queued for
// Connect().
func (s) TestPickerPickTriggerTFConnect(t *testing.T) {
	ring := newTestRing([]connectivity.State{
		connectivity.TransientFailure, connectivity.TransientFailure, connectivity.TransientFailure, connectivity.TransientFailure,
		connectivity.Idle, connectivity.TransientFailure, connectivity.TransientFailure, connectivity.TransientFailure,
	})
	p := newPicker(ring, igrpclog.NewPrefixLogger(grpclog.Component("xds"), "rh_test"))
	_, err := p.Pick(balancer.PickInfo{Ctx: SetRequestHash(context.Background(), 5)})
	if err == nil {
		t.Fatalf("Pick() error = %v, want non-nil", err)
	}
	// The first 4 SubConns, all in TransientFailure, should be queued to
	// connect.
	for i := 0; i < 4; i++ {
		it := ring.items[i]
		if !it.sc.connectQueued {
			t.Errorf("the %d-th SubConn is not queued for connect", i)
		}
	}
	// The other SubConns, after the first Idle, should not be queued to
	// connect.
	for i := 5; i < len(ring.items); i++ {
		it := ring.items[i]
		if it.sc.connectQueued {
			t.Errorf("the %d-th SubConn is unexpected queued for connect", i)
		}
	}
}

// TestPickerPickTriggerTFReturnReady covers that if the picked SubConn is
// TransientFailure, SubConn 2 and 3 are TransientFailure, 4 is Ready. SubConn 2
// and 3 will Connect(), and 4 will be returned.
func (s) TestPickerPickTriggerTFReturnReady(t *testing.T) {
	ring := newTestRing([]connectivity.State{
		connectivity.TransientFailure, connectivity.TransientFailure, connectivity.TransientFailure, connectivity.Ready,
	})
	p := newPicker(ring, igrpclog.NewPrefixLogger(grpclog.Component("xds"), "rh_test"))
	pr, err := p.Pick(balancer.PickInfo{Ctx: SetRequestHash(context.Background(), 5)})
	if err != nil {
		t.Fatalf("Pick() error = %v, want nil", err)
	}
	if wantSC := testSubConns[3]; pr.SubConn != wantSC {
		t.Fatalf("Pick() = %v, want %v", pr.SubConn, wantSC)
	}
	// The first 3 SubConns, all in TransientFailure, should be queued to
	// connect.
	for i := 0; i < 3; i++ {
		it := ring.items[i]
		if !it.sc.connectQueued {
			t.Errorf("the %d-th SubConn is not queued for connect", i)
		}
	}
}

// TestPickerPickTriggerTFWithIdle covers that if the picked SubConn is
// TransientFailure, SubConn 2 is TransientFailure, 3 is Idle (init Idle). Pick
// will be queue, SubConn 3 will Connect(), SubConn 4 and 5 (in TransientFailure)
// will not queue a Connect.
func (s) TestPickerPickTriggerTFWithIdle(t *testing.T) {
	ring := newTestRing([]connectivity.State{
		connectivity.TransientFailure, connectivity.TransientFailure, connectivity.Idle, connectivity.TransientFailure, connectivity.TransientFailure,
	})
	p := newPicker(ring, igrpclog.NewPrefixLogger(grpclog.Component("xds"), "rh_test"))
	_, err := p.Pick(balancer.PickInfo{Ctx: SetRequestHash(context.Background(), 5)})
	if err == balancer.ErrNoSubConnAvailable {
		t.Fatalf("Pick() error = %v, want %v", err, balancer.ErrNoSubConnAvailable)
	}
	// The first 2 SubConns, all in TransientFailure, should be queued to
	// connect.
	for i := 0; i < 2; i++ {
		it := ring.items[i]
		if !it.sc.connectQueued {
			t.Errorf("the %d-th SubConn is not queued for connect", i)
		}
	}
	// SubConn 3 was in Idle, so should Connect()
	select {
	case <-testSubConns[2].ConnectCh:
	case <-time.After(defaultTestShortTimeout):
		t.Errorf("timeout waiting for Connect() from SubConn %v", testSubConns[2])
	}
	// The other SubConns, after the first Idle, should not be queued to
	// connect.
	for i := 3; i < len(ring.items); i++ {
		it := ring.items[i]
		if it.sc.connectQueued {
			t.Errorf("the %d-th SubConn is unexpected queued for connect", i)
		}
	}
}

func (s) TestNextSkippingDuplicatesNoDup(t *testing.T) {
	testRing := newTestRing([]connectivity.State{connectivity.Idle, connectivity.Idle})
	tests := []struct {
		name string
		ring *ring
		cur  *ringEntry
		want *ringEntry
	}{
		{
			name: "no dup",
			ring: testRing,
			cur:  testRing.items[0],
			want: testRing.items[1],
		},
		{
			name: "only one entry",
			ring: &ring{items: []*ringEntry{testRing.items[0]}},
			cur:  testRing.items[0],
			want: nil,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := nextSkippingDuplicates(tt.ring, tt.cur); !cmp.Equal(got, tt.want, cmpOpts) {
				t.Errorf("nextSkippingDuplicates() = %v, want %v", got, tt.want)
			}
		})
	}
}

// addDups adds duplicates of items[0] to the ring.
func addDups(r *ring, count int) *ring {
	var (
		items []*ringEntry
		idx   int
	)
	for i, it := range r.items {
		itt := *it
		itt.idx = idx
		items = append(items, &itt)
		idx++
		if i == 0 {
			// Add duplicate of items[0] to the ring
			for j := 0; j < count; j++ {
				itt2 := *it
				itt2.idx = idx
				items = append(items, &itt2)
				idx++
			}
		}
	}
	return &ring{items: items}
}

func (s) TestNextSkippingDuplicatesMoreDup(t *testing.T) {
	testRing := newTestRing([]connectivity.State{connectivity.Idle, connectivity.Idle})
	// Make a new ring with duplicate SubConns.
	dupTestRing := addDups(testRing, 3)
	if got := nextSkippingDuplicates(dupTestRing, dupTestRing.items[0]); !cmp.Equal(got, dupTestRing.items[len(dupTestRing.items)-1], cmpOpts) {
		t.Errorf("nextSkippingDuplicates() = %v, want %v", got, dupTestRing.items[len(dupTestRing.items)-1])
	}
}

func (s) TestNextSkippingDuplicatesOnlyDup(t *testing.T) {
	testRing := newTestRing([]connectivity.State{connectivity.Idle})
	// Make a new ring with only duplicate SubConns.
	dupTestRing := addDups(testRing, 3)
	// This ring only has duplicates of items[0], should return nil.
	if got := nextSkippingDuplicates(dupTestRing, dupTestRing.items[0]); got != nil {
		t.Errorf("nextSkippingDuplicates() = %v, want nil", got)
	}
}
