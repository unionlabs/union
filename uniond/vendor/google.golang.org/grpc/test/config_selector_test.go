/*
 *
 * Copyright 2020 gRPC authors.
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

package test

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"google.golang.org/grpc/codes"
	iresolver "google.golang.org/grpc/internal/resolver"
	"google.golang.org/grpc/internal/serviceconfig"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	testpb "google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/status"
)

type funcConfigSelector struct {
	f func(iresolver.RPCInfo) (*iresolver.RPCConfig, error)
}

func (f funcConfigSelector) SelectConfig(i iresolver.RPCInfo) (*iresolver.RPCConfig, error) {
	return f.f(i)
}

func (s) TestConfigSelector(t *testing.T) {
	gotContextChan := testutils.NewChannelWithSize(1)

	ss := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
			gotContextChan.SendContext(ctx, ctx)
			return &testpb.Empty{}, nil
		},
	}
	ss.R = manual.NewBuilderWithScheme("confSel")

	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	const normalTimeout = 10 * time.Second
	ctxDeadline := time.Now().Add(normalTimeout)
	ctx, cancel := context.WithTimeout(context.Background(), normalTimeout)
	defer cancel()

	const longTimeout = 30 * time.Second
	longCtxDeadline := time.Now().Add(longTimeout)
	longdeadlineCtx, cancel := context.WithTimeout(context.Background(), longTimeout)
	defer cancel()
	shorterTimeout := 3 * time.Second

	testMD := metadata.MD{"footest": []string{"bazbar"}}
	mdOut := metadata.MD{"handler": []string{"value"}}

	var onCommittedCalled bool

	testCases := []struct {
		name   string
		md     metadata.MD          // MD sent with RPC
		config *iresolver.RPCConfig // config returned by config selector
		csErr  error                // error returned by config selector

		wantMD       metadata.MD
		wantDeadline time.Time
		wantTimeout  time.Duration
		wantErr      error
	}{{
		name:         "basic",
		md:           testMD,
		config:       &iresolver.RPCConfig{},
		wantMD:       testMD,
		wantDeadline: ctxDeadline,
	}, {
		name: "alter MD",
		md:   testMD,
		config: &iresolver.RPCConfig{
			Context: metadata.NewOutgoingContext(ctx, mdOut),
		},
		wantMD:       mdOut,
		wantDeadline: ctxDeadline,
	}, {
		name:    "erroring SelectConfig",
		csErr:   status.Errorf(codes.Unavailable, "cannot send RPC"),
		wantErr: status.Errorf(codes.Unavailable, "cannot send RPC"),
	}, {
		name: "alter timeout; remove MD",
		md:   testMD,
		config: &iresolver.RPCConfig{
			Context: longdeadlineCtx, // no metadata
		},
		wantMD:       nil,
		wantDeadline: longCtxDeadline,
	}, {
		name:         "nil config",
		md:           metadata.MD{},
		config:       nil,
		wantMD:       nil,
		wantDeadline: ctxDeadline,
	}, {
		name: "alter timeout via method config; remove MD",
		md:   testMD,
		config: &iresolver.RPCConfig{
			MethodConfig: serviceconfig.MethodConfig{
				Timeout: &shorterTimeout,
			},
		},
		wantMD:      nil,
		wantTimeout: shorterTimeout,
	}, {
		name: "onCommitted callback",
		md:   testMD,
		config: &iresolver.RPCConfig{
			OnCommitted: func() {
				onCommittedCalled = true
			},
		},
		wantMD:       testMD,
		wantDeadline: ctxDeadline,
	}}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			var gotInfo *iresolver.RPCInfo
			state := iresolver.SetConfigSelector(resolver.State{
				Addresses:     []resolver.Address{{Addr: ss.Address}},
				ServiceConfig: parseServiceConfig(t, ss.R, "{}"),
			}, funcConfigSelector{
				f: func(i iresolver.RPCInfo) (*iresolver.RPCConfig, error) {
					gotInfo = &i
					cfg := tc.config
					if cfg != nil && cfg.Context == nil {
						cfg.Context = i.Context
					}
					return cfg, tc.csErr
				},
			})
			ss.R.UpdateState(state) // Blocks until config selector is applied

			onCommittedCalled = false
			ctx := metadata.NewOutgoingContext(ctx, tc.md)
			startTime := time.Now()
			if _, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}); fmt.Sprint(err) != fmt.Sprint(tc.wantErr) {
				t.Fatalf("client.EmptyCall(_, _) = _, %v; want _, %v", err, tc.wantErr)
			} else if err != nil {
				return // remaining checks are invalid
			}

			if gotInfo == nil {
				t.Fatalf("no config selector data")
			}

			if want := "/grpc.testing.TestService/EmptyCall"; gotInfo.Method != want {
				t.Errorf("gotInfo.Method = %q; want %q", gotInfo.Method, want)
			}

			gotContextI, ok := gotContextChan.ReceiveOrFail()
			if !ok {
				t.Fatalf("no context received")
			}
			gotContext := gotContextI.(context.Context)

			gotMD, _ := metadata.FromOutgoingContext(gotInfo.Context)
			if diff := cmp.Diff(tc.md, gotMD); diff != "" {
				t.Errorf("gotInfo.Context contains MD %v; want %v\nDiffs: %v", gotMD, tc.md, diff)
			}

			gotMD, _ = metadata.FromIncomingContext(gotContext)
			// Remove entries from gotMD not in tc.wantMD (e.g. authority header).
			for k := range gotMD {
				if _, ok := tc.wantMD[k]; !ok {
					delete(gotMD, k)
				}
			}
			if diff := cmp.Diff(tc.wantMD, gotMD, cmpopts.EquateEmpty()); diff != "" {
				t.Errorf("received md = %v; want %v\nDiffs: %v", gotMD, tc.wantMD, diff)
			}

			wantDeadline := tc.wantDeadline
			if wantDeadline.Equal(time.Time{}) {
				wantDeadline = startTime.Add(tc.wantTimeout)
			}
			deadlineGot, _ := gotContext.Deadline()
			if diff := deadlineGot.Sub(wantDeadline); diff > time.Second || diff < -time.Second {
				t.Errorf("received deadline = %v; want ~%v", deadlineGot, wantDeadline)
			}

			if tc.config != nil && tc.config.OnCommitted != nil && !onCommittedCalled {
				t.Errorf("OnCommitted callback not called")
			}
		})
	}
}
