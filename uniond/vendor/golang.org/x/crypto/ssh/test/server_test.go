// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package test

import (
	"net"

	"golang.org/x/crypto/ssh"
)

type exitStatusMsg struct {
	Status uint32
}

// goTestServer is a test Go SSH server that accepts public key and certificate
// authentication and replies with a 0 exit status to any exec request without
// running any commands.
type goTestServer struct {
	listener net.Listener
	config   *ssh.ServerConfig
	done     <-chan struct{}
}

func newTestServer(config *ssh.ServerConfig) (*goTestServer, error) {
	server := &goTestServer{
		config: config,
	}
	listener, err := net.Listen("tcp", "127.0.0.1:")
	if err != nil {
		return nil, err
	}
	server.listener = listener
	done := make(chan struct{}, 1)
	server.done = done
	go server.acceptConnections(done)

	return server, nil
}

func (s *goTestServer) port() (string, error) {
	_, port, err := net.SplitHostPort(s.listener.Addr().String())
	return port, err
}

func (s *goTestServer) acceptConnections(done chan<- struct{}) {
	defer close(done)

	for {
		c, err := s.listener.Accept()
		if err != nil {
			return
		}
		_, chans, reqs, err := ssh.NewServerConn(c, s.config)
		if err != nil {
			return
		}
		go ssh.DiscardRequests(reqs)
		defer c.Close()

		for newChannel := range chans {
			if newChannel.ChannelType() != "session" {
				newChannel.Reject(ssh.UnknownChannelType, "unknown channel type")
				continue
			}

			channel, requests, err := newChannel.Accept()
			if err != nil {
				continue
			}

			go func(in <-chan *ssh.Request) {
				for req := range in {
					ok := false
					switch req.Type {
					case "exec":
						ok = true
						go func() {
							channel.SendRequest("exit-status", false, ssh.Marshal(&exitStatusMsg{Status: 0}))
							channel.Close()
						}()
					}
					if req.WantReply {
						req.Reply(ok, nil)
					}
				}
			}(requests)
		}
	}
}

func (s *goTestServer) Close() error {
	err := s.listener.Close()
	// wait for the accept loop to exit
	<-s.done
	return err
}
