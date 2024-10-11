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

// Package main runs an S2Av2 service.
package main

import (
	"flag"
	"log"
	"net"

	"github.com/google/s2a-go/internal/v2/fakes2av2"
	"google.golang.org/grpc"

	s2av2pb "github.com/google/s2a-go/internal/proto/v2/s2a_go_proto"
)

var (
	port = flag.String("port", ":8008", "Fake S2Av2 server address port.")
)

func runFakeS2Av2Server(listenPort *string) {
	listener, err := net.Listen("tcp", *port)
	if err != nil {
		log.Fatalf("Failed to listen on port %s: %v", listener.Addr().String(), err)
	}
	s := grpc.NewServer()
	log.Printf("Server: started gRPC Fake S2Av2 Server at port: %s", listener.Addr())
	s2av2pb.RegisterS2AServiceServer(s, &fakes2av2.Server{})
	if err := s.Serve(listener); err != nil {
		log.Fatalf("Failed to serve: %v", err)
	}
}

func main() {
	runFakeS2Av2Server(port)
}
