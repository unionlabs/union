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

// Package main establishes a connection with an Echo service.
package main

import (
	"context"
	"flag"
	"log"
	"time"

	"github.com/google/s2a-go"
	"google.golang.org/grpc"

	pb "github.com/google/s2a-go/example/proto/echo_go_proto"
)

const (
	defaultTimeout = 10.0 * time.Second
)

var (
	serverAddr = flag.String("server_addr", "0.0.0.0:8080", "Echo service address.")
	s2aAddr    = flag.String("s2a_addr", "0.0.0.0:61365", "S2A service address.")
)

func runClient(serverAddr *string) {
	creds, err := s2a.NewClientCreds(&s2a.ClientOptions{
		S2AAddress:       *s2aAddr,
		VerificationMode: s2a.ConnectToGoogle,
		LocalIdentity:    s2a.NewHostname("test_rsa_client_identity"),
	})
	if err != nil {
		log.Fatalf("NewClientCreds() failed: %v", err)
	}
	opts := []grpc.DialOption{
		grpc.WithTransportCredentials(creds),
		grpc.WithReturnConnectionError(),
		grpc.WithDisableRetry(),
		grpc.WithBlock(),
	}
	conn, err := grpc.Dial(*serverAddr, opts...)
	if err != nil {
		log.Fatalf("Client: failed to connect: %v", err)
	}
	defer conn.Close()
	c := pb.NewEchoClient(conn)
	log.Printf("Client: connected to: %s", *serverAddr)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTimeout)
	defer cancel()
	msg := "Hello S2Av2 user!"
	r, err := c.Echo(ctx, &pb.EchoRequest{Msg: msg})
	if err != nil {
		log.Fatalf("Client: failed to send echo message: %v", err)
	}
	log.Printf("Client: received message from server: %s", r.GetMsg())
}

func main() {
	runClient(serverAddr)
}
