/*
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
 */

package authz

import (
	"bytes"
	"context"
	"fmt"
	"os"
	"sync/atomic"
	"time"
	"unsafe"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/internal/xds/rbac"
	"google.golang.org/grpc/status"
)

var logger = grpclog.Component("authz")

// StaticInterceptor contains engines used to make authorization decisions. It
// either contains two engines deny engine followed by an allow engine or only
// one allow engine.
type StaticInterceptor struct {
	engines rbac.ChainEngine
}

// NewStatic returns a new StaticInterceptor from a static authorization policy
// JSON string.
func NewStatic(authzPolicy string) (*StaticInterceptor, error) {
	rbacs, policyName, err := translatePolicy(authzPolicy)
	if err != nil {
		return nil, err
	}
	chainEngine, err := rbac.NewChainEngine(rbacs, policyName)
	if err != nil {
		return nil, err
	}
	return &StaticInterceptor{*chainEngine}, nil
}

// UnaryInterceptor intercepts incoming Unary RPC requests.
// Only authorized requests are allowed to pass. Otherwise, an unauthorized
// error is returned to the client.
func (i *StaticInterceptor) UnaryInterceptor(ctx context.Context, req any, _ *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
	err := i.engines.IsAuthorized(ctx)
	if err != nil {
		if status.Code(err) == codes.PermissionDenied {
			if logger.V(2) {
				logger.Infof("unauthorized RPC request rejected: %v", err)
			}
			return nil, status.Errorf(codes.PermissionDenied, "unauthorized RPC request rejected")
		}
		return nil, err
	}
	return handler(ctx, req)
}

// StreamInterceptor intercepts incoming Stream RPC requests.
// Only authorized requests are allowed to pass. Otherwise, an unauthorized
// error is returned to the client.
func (i *StaticInterceptor) StreamInterceptor(srv any, ss grpc.ServerStream, _ *grpc.StreamServerInfo, handler grpc.StreamHandler) error {
	err := i.engines.IsAuthorized(ss.Context())
	if err != nil {
		if status.Code(err) == codes.PermissionDenied {
			if logger.V(2) {
				logger.Infof("unauthorized RPC request rejected: %v", err)
			}
			return status.Errorf(codes.PermissionDenied, "unauthorized RPC request rejected")
		}
		return err
	}
	return handler(srv, ss)
}

// FileWatcherInterceptor contains details used to make authorization decisions
// by watching a file path that contains authorization policy in JSON format.
type FileWatcherInterceptor struct {
	internalInterceptor unsafe.Pointer // *StaticInterceptor
	policyFile          string
	policyContents      []byte
	refreshDuration     time.Duration
	cancel              context.CancelFunc
}

// NewFileWatcher returns a new FileWatcherInterceptor from a policy file
// that contains JSON string of authorization policy and a refresh duration to
// specify the amount of time between policy refreshes.
func NewFileWatcher(file string, duration time.Duration) (*FileWatcherInterceptor, error) {
	if file == "" {
		return nil, fmt.Errorf("authorization policy file path is empty")
	}
	if duration <= time.Duration(0) {
		return nil, fmt.Errorf("requires refresh interval(%v) greater than 0s", duration)
	}
	i := &FileWatcherInterceptor{policyFile: file, refreshDuration: duration}
	if err := i.updateInternalInterceptor(); err != nil {
		return nil, err
	}
	ctx, cancel := context.WithCancel(context.Background())
	i.cancel = cancel
	// Create a background go routine for policy refresh.
	go i.run(ctx)
	return i, nil
}

func (i *FileWatcherInterceptor) run(ctx context.Context) {
	ticker := time.NewTicker(i.refreshDuration)
	for {
		if err := i.updateInternalInterceptor(); err != nil {
			logger.Warningf("authorization policy reload status err: %v", err)
		}
		select {
		case <-ctx.Done():
			ticker.Stop()
			return
		case <-ticker.C:
		}
	}
}

// updateInternalInterceptor checks if the policy file that is watching has changed,
// and if so, updates the internalInterceptor with the policy. Unlike the
// constructor, if there is an error in reading the file or parsing the policy, the
// previous internalInterceptors will not be replaced.
func (i *FileWatcherInterceptor) updateInternalInterceptor() error {
	policyContents, err := os.ReadFile(i.policyFile)
	if err != nil {
		return fmt.Errorf("policyFile(%s) read failed: %v", i.policyFile, err)
	}
	if bytes.Equal(i.policyContents, policyContents) {
		return nil
	}
	i.policyContents = policyContents
	policyContentsString := string(policyContents)
	interceptor, err := NewStatic(policyContentsString)
	if err != nil {
		return err
	}
	atomic.StorePointer(&i.internalInterceptor, unsafe.Pointer(interceptor))
	logger.Infof("authorization policy reload status: successfully loaded new policy %v", policyContentsString)
	return nil
}

// Close cleans up resources allocated by the interceptor.
func (i *FileWatcherInterceptor) Close() {
	i.cancel()
}

// UnaryInterceptor intercepts incoming Unary RPC requests.
// Only authorized requests are allowed to pass. Otherwise, an unauthorized
// error is returned to the client.
func (i *FileWatcherInterceptor) UnaryInterceptor(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
	return ((*StaticInterceptor)(atomic.LoadPointer(&i.internalInterceptor))).UnaryInterceptor(ctx, req, info, handler)
}

// StreamInterceptor intercepts incoming Stream RPC requests.
// Only authorized requests are allowed to pass. Otherwise, an unauthorized
// error is returned to the client.
func (i *FileWatcherInterceptor) StreamInterceptor(srv any, ss grpc.ServerStream, info *grpc.StreamServerInfo, handler grpc.StreamHandler) error {
	return ((*StaticInterceptor)(atomic.LoadPointer(&i.internalInterceptor))).StreamInterceptor(srv, ss, info, handler)
}
