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

package resolver

import (
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/serviceconfig"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

type fakeConfigSelector struct {
	selectConfig func(RPCInfo) (*RPCConfig, error)
}

func (f *fakeConfigSelector) SelectConfig(r RPCInfo) (*RPCConfig, error) {
	return f.selectConfig(r)
}

func (s) TestSafeConfigSelector(t *testing.T) {
	testRPCInfo := RPCInfo{Method: "test method"}

	retChan1 := make(chan *RPCConfig)
	retChan2 := make(chan *RPCConfig)
	defer close(retChan1)
	defer close(retChan2)

	one := 1
	two := 2

	resp1 := &RPCConfig{MethodConfig: serviceconfig.MethodConfig{MaxReqSize: &one}}
	resp2 := &RPCConfig{MethodConfig: serviceconfig.MethodConfig{MaxReqSize: &two}}

	cs1Called := make(chan struct{}, 1)
	cs2Called := make(chan struct{}, 1)

	cs1 := &fakeConfigSelector{
		selectConfig: func(r RPCInfo) (*RPCConfig, error) {
			cs1Called <- struct{}{}
			if diff := cmp.Diff(r, testRPCInfo); diff != "" {
				t.Errorf("SelectConfig(%v) called; want %v\n  Diffs:\n%s", r, testRPCInfo, diff)
			}
			return <-retChan1, nil
		},
	}
	cs2 := &fakeConfigSelector{
		selectConfig: func(r RPCInfo) (*RPCConfig, error) {
			cs2Called <- struct{}{}
			if diff := cmp.Diff(r, testRPCInfo); diff != "" {
				t.Errorf("SelectConfig(%v) called; want %v\n  Diffs:\n%s", r, testRPCInfo, diff)
			}
			return <-retChan2, nil
		},
	}

	scs := &SafeConfigSelector{}
	scs.UpdateConfigSelector(cs1)

	cs1Returned := make(chan struct{})
	go func() {
		got, err := scs.SelectConfig(testRPCInfo) // blocks until send to retChan1
		if err != nil || got != resp1 {
			t.Errorf("SelectConfig(%v) = %v, %v; want %v, nil", testRPCInfo, got, err, resp1)
		}
		close(cs1Returned)
	}()

	// cs1 is blocked but should be called
	select {
	case <-time.After(500 * time.Millisecond):
		t.Fatalf("timed out waiting for cs1 to be called")
	case <-cs1Called:
	}

	// swap in cs2 now that cs1 is called
	csSwapped := make(chan struct{})
	go func() {
		// wait awhile first to ensure cs1 could be called below.
		time.Sleep(50 * time.Millisecond)
		scs.UpdateConfigSelector(cs2) // Blocks until cs1 done
		close(csSwapped)
	}()

	// Allow cs1 to return and cs2 to eventually be swapped in.
	retChan1 <- resp1

	cs1Done := false // set when cs2 is first called
	for dl := time.Now().Add(150 * time.Millisecond); !time.Now().After(dl); {
		gotConfigChan := make(chan *RPCConfig, 1)
		go func() {
			cfg, _ := scs.SelectConfig(testRPCInfo)
			gotConfigChan <- cfg
		}()
		select {
		case <-time.After(500 * time.Millisecond):
			t.Fatalf("timed out waiting for cs1 or cs2 to be called")
		case <-cs1Called:
			// Initially, before swapping to cs2, cs1 should be called
			retChan1 <- resp1
			go func() { <-gotConfigChan }()
			if cs1Done {
				t.Fatalf("cs1 called after cs2")
			}
		case <-cs2Called:
			// Success! the new config selector is being called
			if !cs1Done {
				select {
				case <-csSwapped:
				case <-time.After(50 * time.Millisecond):
					t.Fatalf("timed out waiting for UpdateConfigSelector to return")
				}
				select {
				case <-cs1Returned:
				case <-time.After(50 * time.Millisecond):
					t.Fatalf("timed out waiting for cs1 to return")
				}
				cs1Done = true
			}
			retChan2 <- resp2
			got := <-gotConfigChan
			if diff := cmp.Diff(got, resp2); diff != "" {
				t.Fatalf("SelectConfig(%v) = %v; want %v\n  Diffs:\n%s", testRPCInfo, got, resp2, diff)
			}
		}
		time.Sleep(10 * time.Millisecond)
	}
	if !cs1Done {
		t.Fatalf("timed out waiting for cs2 to be called")
	}
}
