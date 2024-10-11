package amqp

import (
	"context"
	"encoding/json"
	"time"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/transport"
	"github.com/go-kit/log"
	amqp "github.com/rabbitmq/amqp091-go"
)

// Subscriber wraps an endpoint and provides a handler for AMQP Delivery messages.
type Subscriber struct {
	e                 endpoint.Endpoint
	dec               DecodeRequestFunc
	enc               EncodeResponseFunc
	before            []RequestFunc
	after             []SubscriberResponseFunc
	responsePublisher ResponsePublisher
	errorEncoder      ErrorEncoder
	errorHandler      transport.ErrorHandler
}

// NewSubscriber constructs a new subscriber, which provides a handler
// for AMQP Delivery messages.
func NewSubscriber(
	e endpoint.Endpoint,
	dec DecodeRequestFunc,
	enc EncodeResponseFunc,
	options ...SubscriberOption,
) *Subscriber {
	s := &Subscriber{
		e:                 e,
		dec:               dec,
		enc:               enc,
		responsePublisher: DefaultResponsePublisher,
		errorEncoder:      DefaultErrorEncoder,
		errorHandler:      transport.NewLogErrorHandler(log.NewNopLogger()),
	}
	for _, option := range options {
		option(s)
	}
	return s
}

// SubscriberOption sets an optional parameter for subscribers.
type SubscriberOption func(*Subscriber)

// SubscriberBefore functions are executed on the publisher delivery object
// before the request is decoded.
func SubscriberBefore(before ...RequestFunc) SubscriberOption {
	return func(s *Subscriber) { s.before = append(s.before, before...) }
}

// SubscriberAfter functions are executed on the subscriber reply after the
// endpoint is invoked, but before anything is published to the reply.
func SubscriberAfter(after ...SubscriberResponseFunc) SubscriberOption {
	return func(s *Subscriber) { s.after = append(s.after, after...) }
}

// SubscriberResponsePublisher is used by the subscriber to deliver response
// objects to the original sender.
// By default, the DefaultResponsePublisher is used.
func SubscriberResponsePublisher(rp ResponsePublisher) SubscriberOption {
	return func(s *Subscriber) { s.responsePublisher = rp }
}

// SubscriberErrorEncoder is used to encode errors to the subscriber reply
// whenever they're encountered in the processing of a request. Clients can
// use this to provide custom error formatting. By default,
// errors will be published with the DefaultErrorEncoder.
func SubscriberErrorEncoder(ee ErrorEncoder) SubscriberOption {
	return func(s *Subscriber) { s.errorEncoder = ee }
}

// SubscriberErrorLogger is used to log non-terminal errors. By default, no errors
// are logged. This is intended as a diagnostic measure. Finer-grained control
// of error handling, including logging in more detail, should be performed in a
// custom SubscriberErrorEncoder which has access to the context.
// Deprecated: Use SubscriberErrorHandler instead.
func SubscriberErrorLogger(logger log.Logger) SubscriberOption {
	return func(s *Subscriber) { s.errorHandler = transport.NewLogErrorHandler(logger) }
}

// SubscriberErrorHandler is used to handle non-terminal errors. By default, non-terminal errors
// are ignored. This is intended as a diagnostic measure. Finer-grained control
// of error handling, including logging in more detail, should be performed in a
// custom SubscriberErrorEncoder which has access to the context.
func SubscriberErrorHandler(errorHandler transport.ErrorHandler) SubscriberOption {
	return func(s *Subscriber) { s.errorHandler = errorHandler }
}

// ServeDelivery handles AMQP Delivery messages
// It is strongly recommended to use *amqp.Channel as the
// Channel interface implementation.
func (s Subscriber) ServeDelivery(ch Channel) func(deliv *amqp.Delivery) {
	return func(deliv *amqp.Delivery) {
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()

		pub := amqp.Publishing{}

		for _, f := range s.before {
			ctx = f(ctx, &pub, deliv)
		}

		request, err := s.dec(ctx, deliv)
		if err != nil {
			s.errorHandler.Handle(ctx, err)
			s.errorEncoder(ctx, err, deliv, ch, &pub)
			return
		}

		response, err := s.e(ctx, request)
		if err != nil {
			s.errorHandler.Handle(ctx, err)
			s.errorEncoder(ctx, err, deliv, ch, &pub)
			return
		}

		for _, f := range s.after {
			ctx = f(ctx, deliv, ch, &pub)
		}

		if err := s.enc(ctx, &pub, response); err != nil {
			s.errorHandler.Handle(ctx, err)
			s.errorEncoder(ctx, err, deliv, ch, &pub)
			return
		}

		if err := s.responsePublisher(ctx, deliv, ch, &pub); err != nil {
			s.errorHandler.Handle(ctx, err)
			s.errorEncoder(ctx, err, deliv, ch, &pub)
			return
		}
	}

}

// EncodeJSONResponse marshals the response as JSON as part of the
// payload of the AMQP Publishing object.
func EncodeJSONResponse(
	ctx context.Context,
	pub *amqp.Publishing,
	response interface{},
) error {
	b, err := json.Marshal(response)
	if err != nil {
		return err
	}
	pub.Body = b
	return nil
}

// EncodeNopResponse is a response function that does nothing.
func EncodeNopResponse(
	ctx context.Context,
	pub *amqp.Publishing,
	response interface{},
) error {
	return nil
}

// ResponsePublisher functions are executed by the subscriber to
// publish response object to the original sender.
// Please note that the word "publisher" does not refer
// to the publisher of pub/sub.
// Rather, publisher is merely a function that publishes, or sends responses.
type ResponsePublisher func(
	context.Context,
	*amqp.Delivery,
	Channel,
	*amqp.Publishing,
) error

// DefaultResponsePublisher extracts the reply exchange and reply key
// from the request, and sends the response object to that destination.
func DefaultResponsePublisher(
	ctx context.Context,
	deliv *amqp.Delivery,
	ch Channel,
	pub *amqp.Publishing,
) error {
	if pub.CorrelationId == "" {
		pub.CorrelationId = deliv.CorrelationId
	}

	replyExchange := getPublishExchange(ctx)
	replyTo := getPublishKey(ctx)
	if replyTo == "" {
		replyTo = deliv.ReplyTo
	}

	return ch.Publish(
		replyExchange,
		replyTo,
		false, // mandatory
		false, // immediate
		*pub,
	)
}

// NopResponsePublisher does not deliver a response to the original sender.
// This response publisher is used when the user wants the subscriber to
// receive and forget.
func NopResponsePublisher(
	ctx context.Context,
	deliv *amqp.Delivery,
	ch Channel,
	pub *amqp.Publishing,
) error {
	return nil
}

// ErrorEncoder is responsible for encoding an error to the subscriber reply.
// Users are encouraged to use custom ErrorEncoders to encode errors to
// their replies, and will likely want to pass and check for their own error
// types.
type ErrorEncoder func(ctx context.Context,
	err error, deliv *amqp.Delivery, ch Channel, pub *amqp.Publishing)

// DefaultErrorEncoder simply ignores the message. It does not reply
// nor Ack/Nack the message.
func DefaultErrorEncoder(ctx context.Context,
	err error, deliv *amqp.Delivery, ch Channel, pub *amqp.Publishing) {
}

// SingleNackRequeueErrorEncoder issues a Nack to the delivery with multiple flag set as false
// and requeue flag set as true. It does not reply the message.
func SingleNackRequeueErrorEncoder(ctx context.Context,
	err error, deliv *amqp.Delivery, ch Channel, pub *amqp.Publishing) {
	deliv.Nack(
		false, //multiple
		true,  //requeue
	)
	duration := getNackSleepDuration(ctx)
	time.Sleep(duration)
}

// ReplyErrorEncoder serializes the error message as a DefaultErrorResponse
// JSON and sends the message to the ReplyTo address.
func ReplyErrorEncoder(
	ctx context.Context,
	err error,
	deliv *amqp.Delivery,
	ch Channel,
	pub *amqp.Publishing,
) {

	if pub.CorrelationId == "" {
		pub.CorrelationId = deliv.CorrelationId
	}

	replyExchange := getPublishExchange(ctx)
	replyTo := getPublishKey(ctx)
	if replyTo == "" {
		replyTo = deliv.ReplyTo
	}

	response := DefaultErrorResponse{err.Error()}

	b, err := json.Marshal(response)
	if err != nil {
		return
	}
	pub.Body = b

	ch.Publish(
		replyExchange,
		replyTo,
		false, // mandatory
		false, // immediate
		*pub,
	)
}

// ReplyAndAckErrorEncoder serializes the error message as a DefaultErrorResponse
// JSON and sends the message to the ReplyTo address then Acks the original
// message.
func ReplyAndAckErrorEncoder(ctx context.Context, err error, deliv *amqp.Delivery, ch Channel, pub *amqp.Publishing) {
	ReplyErrorEncoder(ctx, err, deliv, ch, pub)
	deliv.Ack(false)
}

// DefaultErrorResponse is the default structure of responses in the event
// of an error.
type DefaultErrorResponse struct {
	Error string `json:"err"`
}

// Channel is a channel interface to make testing possible.
// It is highly recommended to use *amqp.Channel as the interface implementation.
type Channel interface {
	Publish(exchange, key string, mandatory, immediate bool, msg amqp.Publishing) error
	Consume(queue, consumer string, autoAck, exclusive, noLocal, noWail bool, args amqp.Table) (<-chan amqp.Delivery, error)
}
