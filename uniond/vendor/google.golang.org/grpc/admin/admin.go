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

// Package admin provides a convenient method for registering a collection of
// administration services to a gRPC server. The services registered are:
//
// - Channelz: https://github.com/grpc/proposal/blob/master/A14-channelz.md
//
// - CSDS: https://github.com/grpc/proposal/blob/master/A40-csds-support.md
//
// # Experimental
//
// Notice: All APIs in this package are experimental and may be removed in a
// later release.
package admin

import (
	"google.golang.org/grpc"
	channelzservice "google.golang.org/grpc/channelz/service"
	internaladmin "google.golang.org/grpc/internal/admin"
)

func init() {
	// Add a list of default services to admin here. Optional services, like
	// CSDS, will be added by other packages.
	internaladmin.AddService(func(registrar grpc.ServiceRegistrar) (func(), error) {
		channelzservice.RegisterChannelzServiceToServer(registrar)
		return nil, nil
	})
}

// Register registers the set of admin services to the given server.
//
// The returned cleanup function should be called to clean up the resources
// allocated for the service handlers after the server is stopped.
//
// Note that if `s` is not a *grpc.Server or a *xds.GRPCServer, CSDS will not be
// registered because CSDS generated code is old and doesn't support interface
// `grpc.ServiceRegistrar`.
// https://github.com/envoyproxy/go-control-plane/issues/403
func Register(s grpc.ServiceRegistrar) (cleanup func(), _ error) {
	return internaladmin.Register(s)
}
