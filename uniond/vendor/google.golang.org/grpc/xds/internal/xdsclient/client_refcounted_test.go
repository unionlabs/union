/*
 *
 * Copyright 2022 gRPC authors.
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

package xdsclient

import (
	"context"
	"sync"
	"testing"

	"github.com/google/uuid"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
)

// Tests that multiple calls to New() with the same name returns the same
// client. Also verifies that only when all references to the newly created
// client are released, the underlying client is closed.
func (s) TestClientNew_Single(t *testing.T) {
	// Create a bootstrap configuration, place it in a file in the temp
	// directory, and set the bootstrap env vars to point to it.
	nodeID := uuid.New().String()
	contents := e2e.DefaultBootstrapContents(t, nodeID, "non-existent-server-address")
	testutils.CreateBootstrapFileForTesting(t, contents)

	// Override the client creation hook to get notified.
	origClientImplCreateHook := xdsClientImplCreateHook
	clientImplCreateCh := testutils.NewChannel()
	xdsClientImplCreateHook = func(name string) {
		clientImplCreateCh.Replace(name)
	}
	defer func() { xdsClientImplCreateHook = origClientImplCreateHook }()

	// Override the client close hook to get notified.
	origClientImplCloseHook := xdsClientImplCloseHook
	clientImplCloseCh := testutils.NewChannel()
	xdsClientImplCloseHook = func(name string) {
		clientImplCloseCh.Replace(name)
	}
	defer func() { xdsClientImplCloseHook = origClientImplCloseHook }()

	// The first call to New() should create a new client.
	_, closeFunc, err := New(t.Name())
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := clientImplCreateCh.Receive(ctx); err != nil {
		t.Fatalf("Timeout when waiting for xDS client to be created: %v", err)
	}

	// Calling New() again should not create new client implementations.
	const count = 9
	closeFuncs := make([]func(), count)
	for i := 0; i < count; i++ {
		func() {
			_, closeFuncs[i], err = New(t.Name())
			if err != nil {
				t.Fatalf("%d-th call to New() failed with error: %v", i, err)
			}

			sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
			defer sCancel()
			if _, err := clientImplCreateCh.Receive(sCtx); err == nil {
				t.Fatalf("%d-th call to New() created a new client", i)
			}
		}()
	}

	// Call Close() multiple times on each of the clients created in the above
	// for loop. Close() calls are idempotent, and the underlying client
	// implementation will not be closed until we release the first reference we
	// acquired above, via the first call to New().
	for i := 0; i < count; i++ {
		func() {
			closeFuncs[i]()
			closeFuncs[i]()

			sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
			defer sCancel()
			if _, err := clientImplCloseCh.Receive(sCtx); err == nil {
				t.Fatal("Client implementation closed before all references are released")
			}
		}()
	}

	// Call the last Close(). The underlying implementation should be closed.
	closeFunc()
	if _, err := clientImplCloseCh.Receive(ctx); err != nil {
		t.Fatalf("Timeout waiting for client implementation to be closed: %v", err)
	}

	// Calling New() again, after the previous Client was actually closed,
	// should create a new one.
	_, closeFunc, err = New(t.Name())
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer closeFunc()
	if _, err := clientImplCreateCh.Receive(ctx); err != nil {
		t.Fatalf("Timeout when waiting for xDS client to be created: %v", err)
	}
}

// Tests the scenario where there are multiple calls to New() with different
// names. Verifies that reference counts are tracked correctly for each client
// and that only when all references are released for a client, it is closed.
func (s) TestClientNew_Multiple(t *testing.T) {
	// Create a bootstrap configuration, place it in a file in the temp
	// directory, and set the bootstrap env vars to point to it.
	nodeID := uuid.New().String()
	contents := e2e.DefaultBootstrapContents(t, nodeID, "non-existent-server-address")
	testutils.CreateBootstrapFileForTesting(t, contents)

	// Override the client creation hook to get notified.
	origClientImplCreateHook := xdsClientImplCreateHook
	clientImplCreateCh := testutils.NewChannel()
	xdsClientImplCreateHook = func(name string) {
		clientImplCreateCh.Replace(name)
	}
	defer func() { xdsClientImplCreateHook = origClientImplCreateHook }()

	// Override the client close hook to get notified.
	origClientImplCloseHook := xdsClientImplCloseHook
	clientImplCloseCh := testutils.NewChannel()
	xdsClientImplCloseHook = func(name string) {
		clientImplCloseCh.Replace(name)
	}
	defer func() { xdsClientImplCloseHook = origClientImplCloseHook }()

	// Create two xDS clients.
	client1Name := t.Name() + "-1"
	_, closeFunc1, err := New(client1Name)
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	name, err := clientImplCreateCh.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when waiting for xDS client to be created: %v", err)
	}
	if name.(string) != client1Name {
		t.Fatalf("xDS client created for name %q, want %q", name.(string), client1Name)
	}

	client2Name := t.Name() + "-2"
	_, closeFunc2, err := New(client2Name)
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	name, err = clientImplCreateCh.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when waiting for xDS client to be created: %v", err)
	}
	if name.(string) != client2Name {
		t.Fatalf("xDS client created for name %q, want %q", name.(string), client1Name)
	}

	// Create N more references to each of these clients.
	const count = 9
	closeFuncs1 := make([]func(), count)
	closeFuncs2 := make([]func(), count)
	var wg sync.WaitGroup
	wg.Add(2)
	go func() {
		defer wg.Done()
		for i := 0; i < count; i++ {
			var err error
			_, closeFuncs1[i], err = New(client1Name)
			if err != nil {
				t.Errorf("%d-th call to New() failed with error: %v", i, err)
			}
		}
	}()
	go func() {
		defer wg.Done()
		for i := 0; i < count; i++ {
			var err error
			_, closeFuncs2[i], err = New(client2Name)
			if err != nil {
				t.Errorf("%d-th call to New() failed with error: %v", i, err)
			}
		}
	}()
	wg.Wait()
	if t.Failed() {
		t.FailNow()
	}

	// Ensure that none of the create hooks are called.
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if _, err := clientImplCreateCh.Receive(sCtx); err == nil {
		t.Fatalf("New xDS client created when expected to reuse an existing one")
	}

	// The close function returned by New() is idempotent and calling it
	// multiple times should not decrement the reference count multiple times.
	for i := 0; i < count; i++ {
		closeFuncs1[i]()
		closeFuncs1[i]()
	}
	sCtx, sCancel = context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if _, err := clientImplCloseCh.Receive(sCtx); err == nil {
		t.Fatal("Client implementation closed before all references are released")
	}

	// Release the last reference and verify that the client is closed
	// completely.
	closeFunc1()
	name, err = clientImplCloseCh.Receive(ctx)
	if err != nil {
		t.Fatal("Timeout when waiting for xDS client to be closed completely")
	}
	if name.(string) != client1Name {
		t.Fatalf("xDS client closed for name %q, want %q", name.(string), client1Name)
	}

	// Ensure that the close hook is not called for the second client.
	sCtx, sCancel = context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if _, err := clientImplCloseCh.Receive(sCtx); err == nil {
		t.Fatal("Client implementation closed before all references are released")
	}

	// The close function returned by New() is idempotent and calling it
	// multiple times should not decrement the reference count multiple times.
	for i := 0; i < count; i++ {
		closeFuncs2[i]()
		closeFuncs2[i]()
	}
	sCtx, sCancel = context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if _, err := clientImplCloseCh.Receive(sCtx); err == nil {
		t.Fatal("Client implementation closed before all references are released")
	}

	// Release the last reference and verify that the client is closed
	// completely.
	closeFunc2()
	name, err = clientImplCloseCh.Receive(ctx)
	if err != nil {
		t.Fatal("Timeout when waiting for xDS client to be closed completely")
	}
	if name.(string) != client2Name {
		t.Fatalf("xDS client closed for name %q, want %q", name.(string), client2Name)
	}
}
