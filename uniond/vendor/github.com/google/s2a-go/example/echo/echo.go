/*
 *
 * Copyright 2022 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

// Package echo contains the libraries for running an Echo server.
package echo

import (
	"context"

	pb "github.com/google/s2a-go/example/proto/echo_go_proto"
)

// Server is an echo server used for testing.
type Server struct {
	pb.UnimplementedEchoServer
}

// Echo uses the message, Msg, in EchoRequest to formulate EchoResponse.
func (s *Server) Echo(ctx context.Context, req *pb.EchoRequest) (*pb.EchoResponse, error) {
	return &pb.EchoResponse{Msg: req.GetMsg()}, nil
}
