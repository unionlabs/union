// Copyright 2019 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package rpc

import (
	"context"
	"encoding/binary"
	"errors"
	"strings"
	"sync"
	"time"
)

func newTestServer() *Server {
	server := NewServer()
	server.idgen = sequentialIDGenerator()
	if err := server.RegisterName("test", new(testService)); err != nil {
		panic(err)
	}
	if err := server.RegisterName("nftest", new(notificationTestService)); err != nil {
		panic(err)
	}
	return server
}

func sequentialIDGenerator() func() ID {
	var (
		mu      sync.Mutex
		counter uint64
	)
	return func() ID {
		mu.Lock()
		defer mu.Unlock()
		counter++
		id := make([]byte, 8)
		binary.BigEndian.PutUint64(id, counter)
		return encodeID(id)
	}
}

type testService struct{}

type echoArgs struct {
	S string
}

type echoResult struct {
	String string
	Int    int
	Args   *echoArgs
}

type testError struct{}

func (testError) Error() string          { return "testError" }
func (testError) ErrorCode() int         { return 444 }
func (testError) ErrorData() interface{} { return "testError data" }

type MarshalErrObj struct{}

func (o *MarshalErrObj) MarshalText() ([]byte, error) {
	return nil, errors.New("marshal error")
}

func (s *testService) NoArgsRets() {}

func (s *testService) Null() any {
	return nil
}

func (s *testService) Echo(str string, i int, args *echoArgs) echoResult {
	return echoResult{str, i, args}
}

func (s *testService) EchoWithCtx(ctx context.Context, str string, i int, args *echoArgs) echoResult {
	return echoResult{str, i, args}
}

func (s *testService) Repeat(msg string, i int) string {
	return strings.Repeat(msg, i)
}

func (s *testService) PeerInfo(ctx context.Context) PeerInfo {
	return PeerInfoFromContext(ctx)
}

func (s *testService) Sleep(ctx context.Context, duration time.Duration) {
	time.Sleep(duration)
}

func (s *testService) Block(ctx context.Context) error {
	<-ctx.Done()
	return errors.New("context canceled in testservice_block")
}

func (s *testService) Rets() (string, error) {
	return "", nil
}

//lint:ignore ST1008 returns error first on purpose.
func (s *testService) InvalidRets1() (error, string) {
	return nil, ""
}

func (s *testService) InvalidRets2() (string, string) {
	return "", ""
}

func (s *testService) InvalidRets3() (string, string, error) {
	return "", "", nil
}

func (s *testService) ReturnError() error {
	return testError{}
}

func (s *testService) MarshalError() *MarshalErrObj {
	return &MarshalErrObj{}
}

func (s *testService) Panic() string {
	panic("service panic")
}

func (s *testService) CallMeBack(ctx context.Context, method string, args []interface{}) (interface{}, error) {
	c, ok := ClientFromContext(ctx)
	if !ok {
		return nil, errors.New("no client")
	}
	var result interface{}
	err := c.Call(&result, method, args...)
	return result, err
}

func (s *testService) CallMeBackLater(ctx context.Context, method string, args []interface{}) error {
	c, ok := ClientFromContext(ctx)
	if !ok {
		return errors.New("no client")
	}
	go func() {
		<-ctx.Done()
		var result interface{}
		c.Call(&result, method, args...)
	}()
	return nil
}

func (s *testService) Subscription(ctx context.Context) (*Subscription, error) {
	return nil, nil
}

type notificationTestService struct {
	unsubscribed            chan string
	gotHangSubscriptionReq  chan struct{}
	unblockHangSubscription chan struct{}
}

func (s *notificationTestService) Echo(i int) int {
	return i
}

func (s *notificationTestService) Unsubscribe(subid string) {
	if s.unsubscribed != nil {
		s.unsubscribed <- subid
	}
}

func (s *notificationTestService) SomeSubscription(ctx context.Context, n, val int) (*Subscription, error) {
	notifier, supported := NotifierFromContext(ctx)
	if !supported {
		return nil, ErrNotificationsUnsupported
	}

	// By explicitly creating an subscription we make sure that the subscription id is send
	// back to the client before the first subscription.Notify is called. Otherwise the
	// events might be send before the response for the *_subscribe method.
	subscription := notifier.CreateSubscription()
	go func() {
		for i := 0; i < n; i++ {
			if err := notifier.Notify(subscription.ID, val+i); err != nil {
				return
			}
		}
		select {
		case <-notifier.Closed():
		case <-subscription.Err():
		}
		if s.unsubscribed != nil {
			s.unsubscribed <- string(subscription.ID)
		}
	}()
	return subscription, nil
}

// HangSubscription blocks on s.unblockHangSubscription before sending anything.
func (s *notificationTestService) HangSubscription(ctx context.Context, val int) (*Subscription, error) {
	notifier, supported := NotifierFromContext(ctx)
	if !supported {
		return nil, ErrNotificationsUnsupported
	}
	s.gotHangSubscriptionReq <- struct{}{}
	<-s.unblockHangSubscription
	subscription := notifier.CreateSubscription()

	go func() {
		notifier.Notify(subscription.ID, val)
	}()
	return subscription, nil
}

// largeRespService generates arbitrary-size JSON responses.
type largeRespService struct {
	length int
}

func (x largeRespService) LargeResp() string {
	return strings.Repeat("x", x.length)
}
