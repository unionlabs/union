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

package grpc

import (
	"fmt"
	"strings"
	"sync"
	"testing"

	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/balancer/stub"
	"google.golang.org/grpc/internal/grpcsync"
)

// TestBalancer_StateListenerBeforeConnect tries to stimulate a race between
// NewSubConn and ClientConn.Close.  In no cases should the SubConn's
// StateListener be invoked, because Connect was never called.
func (s) TestBalancer_StateListenerBeforeConnect(t *testing.T) {
	// started is fired after cc is set so cc can be used in the balancer.
	started := grpcsync.NewEvent()
	var cc *ClientConn

	wg := sync.WaitGroup{}
	wg.Add(2)

	// Create a balancer that calls NewSubConn and cc.Close at approximately the
	// same time.
	bf := stub.BalancerFuncs{
		UpdateClientConnState: func(bd *stub.BalancerData, ccs balancer.ClientConnState) error {
			go func() {
				// Wait for cc to be valid after the channel is created.
				<-started.Done()
				// In a goroutine, create the subconn.
				go func() {
					_, err := bd.ClientConn.NewSubConn(ccs.ResolverState.Addresses, balancer.NewSubConnOptions{
						StateListener: func(scs balancer.SubConnState) {
							t.Error("Unexpected call to StateListener with:", scs)
						},
					})
					if err != nil && !strings.Contains(err.Error(), "connection is closing") && !strings.Contains(err.Error(), "is deleted") && !strings.Contains(err.Error(), "is closed or idle") && !strings.Contains(err.Error(), "balancer is being closed") {
						t.Error("Unexpected error creating subconn:", err)
					}
					wg.Done()
				}()
				// At approximately the same time, close the channel.
				cc.Close()
				wg.Done()
			}()
			return nil
		},
	}
	stub.Register(t.Name(), bf)
	svcCfg := fmt.Sprintf(`{ "loadBalancingConfig": [{%q: {}}] }`, t.Name())

	cc, err := Dial("fake", WithTransportCredentials(insecure.NewCredentials()), WithDefaultServiceConfig(svcCfg))
	if err != nil {
		t.Fatal("Error dialing:", err)
	}
	started.Fire()

	// Wait for the LB policy to call NewSubConn and cc.Close.
	wg.Wait()
}
