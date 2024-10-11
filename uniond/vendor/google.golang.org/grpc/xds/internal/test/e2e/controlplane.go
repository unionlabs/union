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
 */

package e2e

import (
	"testing"

	"github.com/google/uuid"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
)

type controlPlane struct {
	server           *e2e.ManagementServer
	nodeID           string
	bootstrapContent string
}

func newControlPlane(t *testing.T) (*controlPlane, error) {
	// Spin up an xDS management server on a local port.
	server := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, server.Address)

	return &controlPlane{
		server:           server,
		nodeID:           nodeID,
		bootstrapContent: string(bootstrapContents),
	}, nil
}
