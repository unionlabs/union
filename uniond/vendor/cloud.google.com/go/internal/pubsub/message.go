// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and

package pubsub

import (
	"context"
	"time"
)

// AckHandler implements ack/nack handling.
type AckHandler interface {
	// OnAck processes a message ack.
	OnAck()

	// OnNack processes a message nack.
	OnNack()

	// OnAckWithResult processes a message ack and returns
	// a result that shows if it succeeded.
	OnAckWithResult() *AckResult

	// OnNackWithResult processes a message nack and returns
	// a result that shows if it succeeded.
	OnNackWithResult() *AckResult
}

// Message represents a Pub/Sub message.
type Message struct {
	// ID identifies this message. This ID is assigned by the server and is
	// populated for Messages obtained from a subscription.
	//
	// This field is read-only.
	ID string

	// Data is the actual data in the message.
	Data []byte

	// Attributes represents the key-value pairs the current message is
	// labelled with.
	Attributes map[string]string

	// PublishTime is the time at which the message was published. This is
	// populated by the server for Messages obtained from a subscription.
	//
	// This field is read-only.
	PublishTime time.Time

	// DeliveryAttempt is the number of times a message has been delivered.
	// This is part of the dead lettering feature that forwards messages that
	// fail to be processed (from nack/ack deadline timeout) to a dead letter topic.
	// If dead lettering is enabled, this will be set on all attempts, starting
	// with value 1. Otherwise, the value will be nil.
	// This field is read-only.
	DeliveryAttempt *int

	// OrderingKey identifies related messages for which publish order should
	// be respected. If empty string is used, message will be sent unordered.
	OrderingKey string

	// ackh handles Ack() or Nack().
	ackh AckHandler
}

// Ack indicates successful processing of a Message passed to the Subscriber.Receive callback.
// It should not be called on any other Message value.
// If message acknowledgement fails, the Message will be redelivered.
// Client code must call Ack or Nack when finished for each received Message.
// Calls to Ack or Nack have no effect after the first call.
func (m *Message) Ack() {
	if m.ackh != nil {
		m.ackh.OnAck()
	}
}

// Nack indicates that the client will not or cannot process a Message passed to the Subscriber.Receive callback.
// It should not be called on any other Message value.
// Nack will result in the Message being redelivered more quickly than if it were allowed to expire.
// Client code must call Ack or Nack when finished for each received Message.
// Calls to Ack or Nack have no effect after the first call.
func (m *Message) Nack() {
	if m.ackh != nil {
		m.ackh.OnNack()
	}
}

// AcknowledgeStatus represents the status of an Ack or Nack request.
type AcknowledgeStatus int

const (
	// AcknowledgeStatusSuccess indicates the request was a success.
	AcknowledgeStatusSuccess AcknowledgeStatus = iota
	// AcknowledgeStatusPermissionDenied indicates the caller does not have sufficient permissions.
	AcknowledgeStatusPermissionDenied
	// AcknowledgeStatusFailedPrecondition indicates the request encountered a FailedPrecondition error.
	AcknowledgeStatusFailedPrecondition
	// AcknowledgeStatusInvalidAckID indicates one or more of the ack IDs sent were invalid.
	AcknowledgeStatusInvalidAckID
	// AcknowledgeStatusOther indicates another unknown error was returned.
	AcknowledgeStatusOther
)

// AckResult holds the result from a call to Ack or Nack.
type AckResult struct {
	ready chan struct{}
	res   AcknowledgeStatus
	err   error
}

// Ready returns a channel that is closed when the result is ready.
// When the Ready channel is closed, Get is guaranteed not to block.
func (r *AckResult) Ready() <-chan struct{} { return r.ready }

// Get returns the status and/or error result of a Ack, Nack, or Modack call.
// Get blocks until the Ack/Nack completes or the context is done.
func (r *AckResult) Get(ctx context.Context) (res AcknowledgeStatus, err error) {
	// If the result is already ready, return it even if the context is done.
	select {
	case <-r.Ready():
		return r.res, r.err
	default:
	}
	select {
	case <-ctx.Done():
		// Explicitly return AcknowledgeStatusOther for context cancelled cases,
		// since the default is success.
		return AcknowledgeStatusOther, ctx.Err()
	case <-r.Ready():
		return r.res, r.err
	}
}

// NewAckResult creates a AckResult.
func NewAckResult() *AckResult {
	return &AckResult{
		ready: make(chan struct{}),
	}
}

// SetAckResult sets the ack response and error for a ack result and closes
// the Ready channel. Any call after the first for the same AckResult
// is a no-op.
func SetAckResult(r *AckResult, res AcknowledgeStatus, err error) {
	select {
	case <-r.Ready():
		return
	default:
		r.res = res
		r.err = err
		close(r.ready)
	}
}

// AckWithResult acknowledges a message in Pub/Sub and it will not be
// delivered to this subscription again.
//
// You should avoid acknowledging messages until you have
// *finished* processing them, so that in the event of a failure,
// you receive the message again.
//
// If exactly-once delivery is enabled on the subscription, the
// AckResult returned by this method tracks the state of acknowledgement
// operation. If the operation completes successfully, the message is
// guaranteed NOT to be re-delivered. Otherwise, the result will
// contain an error with more details about the failure and the
// message may be re-delivered.
//
// If exactly-once delivery is NOT enabled on the subscription, or
// if using Pub/Sub Lite, AckResult readies immediately with a AcknowledgeStatus.Success.
// Since acks in Cloud Pub/Sub are best effort when exactly-once
// delivery is disabled, the message may be re-delivered. Because
// re-deliveries are possible, you should ensure that your processing
// code is idempotent, as you may receive any given message more than
// once.
func (m *Message) AckWithResult() *AckResult {
	if m.ackh != nil {
		return m.ackh.OnAckWithResult()
	}
	// When the message was constructed directly rather passed in the callback in `sub.Receive`,
	// ready the message with success so calling `AckResult.Get` doesn't panic.
	return newSuccessAckResult()
}

// NackWithResult declines to acknowledge the message which indicates that
// the client will not or cannot process a Message. This will cause the message
// to be re-delivered to subscribers. Re-deliveries may take place immediately
// or after a delay.
//
// If exactly-once delivery is enabled on the subscription, the
// AckResult returned by this method tracks the state of nack
// operation. If the operation completes successfully, the result will
// contain AckResponse.Success. Otherwise, the result will contain an error
// with more details about the failure.
//
// If exactly-once delivery is NOT enabled on the subscription, or
// if using Pub/Sub Lite, AckResult readies immediately with a AcknowledgeStatus.Success.
func (m *Message) NackWithResult() *AckResult {
	if m.ackh != nil {
		return m.ackh.OnNackWithResult()
	}
	// When the message was constructed directly rather passed in the callback in `sub.Receive`,
	// ready the message with success so calling `AckResult.Get` doesn't panic.
	return newSuccessAckResult()
}

// NewMessage creates a message with an AckHandler implementation, which should
// not be nil.
func NewMessage(ackh AckHandler) *Message {
	return &Message{ackh: ackh}
}

// MessageAckHandler provides access to the internal field Message.ackh.
func MessageAckHandler(m *Message) AckHandler {
	return m.ackh
}

func newSuccessAckResult() *AckResult {
	ar := NewAckResult()
	SetAckResult(ar, AcknowledgeStatusSuccess, nil)
	return ar
}
