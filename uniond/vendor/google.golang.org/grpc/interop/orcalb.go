/*
 *
 * Copyright 2023 gRPC authors.
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

package interop

import (
	"context"
	"fmt"
	"sync"
	"time"

	v3orcapb "github.com/cncf/xds/go/xds/data/orca/v3"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/balancer/base"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/orca"
)

func init() {
	balancer.Register(orcabb{})
}

type orcabb struct{}

func (orcabb) Build(cc balancer.ClientConn, _ balancer.BuildOptions) balancer.Balancer {
	return &orcab{cc: cc}
}

func (orcabb) Name() string {
	return "test_backend_metrics_load_balancer"
}

type orcab struct {
	cc          balancer.ClientConn
	sc          balancer.SubConn
	cancelWatch func()

	reportMu sync.Mutex
	report   *v3orcapb.OrcaLoadReport
}

func (o *orcab) UpdateClientConnState(s balancer.ClientConnState) error {
	if o.sc != nil {
		o.sc.UpdateAddresses(s.ResolverState.Addresses)
		return nil
	}

	if len(s.ResolverState.Addresses) == 0 {
		o.ResolverError(fmt.Errorf("produced no addresses"))
		return fmt.Errorf("resolver produced no addresses")
	}
	var err error
	o.sc, err = o.cc.NewSubConn(s.ResolverState.Addresses, balancer.NewSubConnOptions{StateListener: o.updateSubConnState})
	if err != nil {
		o.cc.UpdateState(balancer.State{ConnectivityState: connectivity.TransientFailure, Picker: base.NewErrPicker(fmt.Errorf("error creating subconn: %v", err))})
		return nil
	}
	o.cancelWatch = orca.RegisterOOBListener(o.sc, o, orca.OOBListenerOptions{ReportInterval: time.Second})
	o.sc.Connect()
	o.cc.UpdateState(balancer.State{ConnectivityState: connectivity.Connecting, Picker: base.NewErrPicker(balancer.ErrNoSubConnAvailable)})
	return nil
}

func (o *orcab) ResolverError(err error) {
	if o.sc == nil {
		o.cc.UpdateState(balancer.State{ConnectivityState: connectivity.TransientFailure, Picker: base.NewErrPicker(fmt.Errorf("resolver error: %v", err))})
	}
}

func (o *orcab) UpdateSubConnState(sc balancer.SubConn, state balancer.SubConnState) {
	logger.Errorf("UpdateSubConnState(%v, %+v) called unexpectedly", sc, state)
}

func (o *orcab) updateSubConnState(state balancer.SubConnState) {
	switch state.ConnectivityState {
	case connectivity.Ready:
		o.cc.UpdateState(balancer.State{ConnectivityState: connectivity.Ready, Picker: &orcaPicker{o: o}})
	case connectivity.TransientFailure:
		o.cc.UpdateState(balancer.State{ConnectivityState: connectivity.TransientFailure, Picker: base.NewErrPicker(fmt.Errorf("all subchannels in transient failure: %v", state.ConnectionError))})
	case connectivity.Connecting:
		// Ignore; picker already set to "connecting".
	case connectivity.Idle:
		o.sc.Connect()
		o.cc.UpdateState(balancer.State{ConnectivityState: connectivity.Connecting, Picker: base.NewErrPicker(balancer.ErrNoSubConnAvailable)})
	case connectivity.Shutdown:
		// Ignore; we are closing but handle that in Close instead.
	}
}

func (o *orcab) Close() {
	o.cancelWatch()
}

func (o *orcab) OnLoadReport(r *v3orcapb.OrcaLoadReport) {
	o.reportMu.Lock()
	defer o.reportMu.Unlock()
	logger.Infof("received OOB load report: %v", r)
	o.report = r
}

type orcaPicker struct {
	o *orcab
}

func (p *orcaPicker) Pick(info balancer.PickInfo) (balancer.PickResult, error) {
	doneCB := func(di balancer.DoneInfo) {
		if lr, _ := di.ServerLoad.(*v3orcapb.OrcaLoadReport); lr != nil &&
			(lr.CpuUtilization != 0 || lr.MemUtilization != 0 || len(lr.Utilization) > 0 || len(lr.RequestCost) > 0) {
			// Since all RPCs will respond with a load report due to the
			// presence of the DialOption, we need to inspect every field and
			// use the out-of-band report instead if all are unset/zero.
			setContextCMR(info.Ctx, lr)
		} else {
			p.o.reportMu.Lock()
			defer p.o.reportMu.Unlock()
			if lr := p.o.report; lr != nil {
				setContextCMR(info.Ctx, lr)
			}
		}
	}
	return balancer.PickResult{SubConn: p.o.sc, Done: doneCB}, nil
}

func setContextCMR(ctx context.Context, lr *v3orcapb.OrcaLoadReport) {
	if r := orcaResultFromContext(ctx); r != nil {
		*r = lr
	}
}

type orcaKey string

var orcaCtxKey = orcaKey("orcaResult")

// contextWithORCAResult sets a key in ctx with a pointer to an ORCA load
// report that is to be filled in by the "test_backend_metrics_load_balancer"
// LB policy's Picker's Done callback.
//
// If a per-call load report is provided from the server for the call, result
// will be filled with that, otherwise the most recent OOB load report is used.
// If no OOB report has been received, result is not modified.
func contextWithORCAResult(ctx context.Context, result **v3orcapb.OrcaLoadReport) context.Context {
	return context.WithValue(ctx, orcaCtxKey, result)
}

// orcaResultFromContext returns the ORCA load report stored in the context.
// The LB policy uses this to communicate the load report back to the interop
// client application.
func orcaResultFromContext(ctx context.Context) **v3orcapb.OrcaLoadReport {
	v := ctx.Value(orcaCtxKey)
	if v == nil {
		return nil
	}
	return v.(**v3orcapb.OrcaLoadReport)
}
