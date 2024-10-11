package amqp

import (
	"context"
	"time"

	amqp "github.com/rabbitmq/amqp091-go"
)

// RequestFunc may take information from a publisher request and put it into a
// request context. In Subscribers, RequestFuncs are executed prior to invoking
// the endpoint.
type RequestFunc func(context.Context, *amqp.Publishing, *amqp.Delivery) context.Context

// SubscriberResponseFunc may take information from a request context and use it to
// manipulate a Publisher. SubscriberResponseFuncs are only executed in
// subscribers, after invoking the endpoint but prior to publishing a reply.
type SubscriberResponseFunc func(context.Context,
	*amqp.Delivery,
	Channel,
	*amqp.Publishing,
) context.Context

// PublisherResponseFunc may take information from an AMQP request and make the
// response available for consumption. PublisherResponseFunc are only executed
// in publishers, after a request has been made, but prior to it being decoded.
type PublisherResponseFunc func(context.Context, *amqp.Delivery) context.Context

// SetPublishExchange returns a RequestFunc that sets the Exchange field
// of an AMQP Publish call.
func SetPublishExchange(publishExchange string) RequestFunc {
	return func(ctx context.Context, pub *amqp.Publishing, _ *amqp.Delivery) context.Context {
		return context.WithValue(ctx, ContextKeyExchange, publishExchange)
	}
}

// SetPublishKey returns a RequestFunc that sets the Key field
// of an AMQP Publish call.
func SetPublishKey(publishKey string) RequestFunc {
	return func(ctx context.Context, pub *amqp.Publishing, _ *amqp.Delivery) context.Context {
		return context.WithValue(ctx, ContextKeyPublishKey, publishKey)
	}
}

// SetPublishDeliveryMode sets the delivery mode of a Publishing.
// Please refer to AMQP delivery mode constants in the AMQP package.
func SetPublishDeliveryMode(dmode uint8) RequestFunc {
	return func(ctx context.Context, pub *amqp.Publishing, _ *amqp.Delivery) context.Context {
		pub.DeliveryMode = dmode
		return ctx
	}
}

// SetNackSleepDuration returns a RequestFunc that sets the amount of time
// to sleep in the event of a Nack.
// This has to be used in conjunction with an error encoder that Nack and sleeps.
// One example is the SingleNackRequeueErrorEncoder.
// It is designed to be used by Subscribers.
func SetNackSleepDuration(duration time.Duration) RequestFunc {
	return func(ctx context.Context, pub *amqp.Publishing, _ *amqp.Delivery) context.Context {
		return context.WithValue(ctx, ContextKeyNackSleepDuration, duration)
	}
}

// SetConsumeAutoAck returns a RequestFunc that sets whether or not to autoAck
// messages when consuming.
// When set to false, the publisher will Ack the first message it receives with
// a matching correlationId.
// It is designed to be used by Publishers.
func SetConsumeAutoAck(autoAck bool) RequestFunc {
	return func(ctx context.Context, pub *amqp.Publishing, _ *amqp.Delivery) context.Context {
		return context.WithValue(ctx, ContextKeyAutoAck, autoAck)
	}
}

// SetConsumeArgs returns a RequestFunc that set the arguments for amqp Consume
// function.
// It is designed to be used by Publishers.
func SetConsumeArgs(args amqp.Table) RequestFunc {
	return func(ctx context.Context, pub *amqp.Publishing, _ *amqp.Delivery) context.Context {
		return context.WithValue(ctx, ContextKeyConsumeArgs, args)
	}
}

// SetContentType returns a RequestFunc that sets the ContentType field of
// an AMQP Publishing.
func SetContentType(contentType string) RequestFunc {
	return func(ctx context.Context, pub *amqp.Publishing, _ *amqp.Delivery) context.Context {
		pub.ContentType = contentType
		return ctx
	}
}

// SetContentEncoding returns a RequestFunc that sets the ContentEncoding field
// of an AMQP Publishing.
func SetContentEncoding(contentEncoding string) RequestFunc {
	return func(ctx context.Context, pub *amqp.Publishing, _ *amqp.Delivery) context.Context {
		pub.ContentEncoding = contentEncoding
		return ctx
	}
}

// SetCorrelationID returns a RequestFunc that sets the CorrelationId field
// of an AMQP Publishing.
func SetCorrelationID(cid string) RequestFunc {
	return func(ctx context.Context, pub *amqp.Publishing, _ *amqp.Delivery) context.Context {
		pub.CorrelationId = cid
		return ctx
	}
}

// SetAckAfterEndpoint returns a SubscriberResponseFunc that prompts the service
// to Ack the Delivery object after successfully evaluating the endpoint,
// and before it encodes the response.
// It is designed to be used by Subscribers.
func SetAckAfterEndpoint(multiple bool) SubscriberResponseFunc {
	return func(ctx context.Context,
		deliv *amqp.Delivery,
		ch Channel,
		pub *amqp.Publishing,
	) context.Context {
		deliv.Ack(multiple)
		return ctx
	}
}

func getPublishExchange(ctx context.Context) string {
	if exchange := ctx.Value(ContextKeyExchange); exchange != nil {
		return exchange.(string)
	}
	return ""
}

func getPublishKey(ctx context.Context) string {
	if publishKey := ctx.Value(ContextKeyPublishKey); publishKey != nil {
		return publishKey.(string)
	}
	return ""
}

func getNackSleepDuration(ctx context.Context) time.Duration {
	if duration := ctx.Value(ContextKeyNackSleepDuration); duration != nil {
		return duration.(time.Duration)
	}
	return 0
}

func getConsumeAutoAck(ctx context.Context) bool {
	if autoAck := ctx.Value(ContextKeyAutoAck); autoAck != nil {
		return autoAck.(bool)
	}
	return false
}

func getConsumeArgs(ctx context.Context) amqp.Table {
	if args := ctx.Value(ContextKeyConsumeArgs); args != nil {
		return args.(amqp.Table)
	}
	return nil
}

type contextKey int

const (
	// ContextKeyExchange is the value of the reply Exchange in
	// amqp.Publish.
	ContextKeyExchange contextKey = iota
	// ContextKeyPublishKey is the value of the ReplyTo field in
	// amqp.Publish.
	ContextKeyPublishKey
	// ContextKeyNackSleepDuration is the duration to sleep for if the
	// service Nack and requeues a message.
	// This is to prevent sporadic send-resending of message
	// when a message is constantly Nack'd and requeued.
	ContextKeyNackSleepDuration
	// ContextKeyAutoAck is the value of autoAck field when calling
	// amqp.Channel.Consume.
	ContextKeyAutoAck
	// ContextKeyConsumeArgs is the value of consumeArgs field when calling
	// amqp.Channel.Consume.
	ContextKeyConsumeArgs
)
