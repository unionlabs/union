package statsd

import (
	"fmt"
	"math"
	"strings"
	"time"
)

var (
	// DefaultNamespace is the default value for the Namespace option
	DefaultNamespace = ""
	// DefaultTags is the default value for the Tags option
	DefaultTags = []string{}
	// DefaultMaxBytesPerPayload is the default value for the MaxBytesPerPayload option
	DefaultMaxBytesPerPayload = 0
	// DefaultMaxMessagesPerPayload is the default value for the MaxMessagesPerPayload option
	DefaultMaxMessagesPerPayload = math.MaxInt32
	// DefaultBufferPoolSize is the default value for the DefaultBufferPoolSize option
	DefaultBufferPoolSize = 0
	// DefaultBufferFlushInterval is the default value for the BufferFlushInterval option
	DefaultBufferFlushInterval = 100 * time.Millisecond
	// DefaultBufferShardCount is the default value for the BufferShardCount option
	DefaultBufferShardCount = 32
	// DefaultSenderQueueSize is the default value for the DefaultSenderQueueSize option
	DefaultSenderQueueSize = 0
	// DefaultWriteTimeoutUDS is the default value for the WriteTimeoutUDS option
	DefaultWriteTimeoutUDS = 100 * time.Millisecond
	// DefaultTelemetry is the default value for the Telemetry option
	DefaultTelemetry = true
	// DefaultReceivingMode is the default behavior when sending metrics
	DefaultReceivingMode = MutexMode
	// DefaultChannelModeBufferSize is the default size of the channel holding incoming metrics
	DefaultChannelModeBufferSize = 4096
	// DefaultAggregationFlushInterval is the default interval for the aggregator to flush metrics.
	// This should divide the Agent reporting period (default=10s) evenly to reduce "aliasing" that
	// can cause values to appear irregular.
	DefaultAggregationFlushInterval = 2 * time.Second
	// DefaultAggregation
	DefaultAggregation = false
	// DefaultExtendedAggregation
	DefaultExtendedAggregation = false
	// DefaultDevMode
	DefaultDevMode = false
)

// Options contains the configuration options for a client.
type Options struct {
	// Namespace to prepend to all metrics, events and service checks name.
	Namespace string
	// Tags are global tags to be applied to every metrics, events and service checks.
	Tags []string
	// MaxBytesPerPayload is the maximum number of bytes a single payload will contain.
	// The magic value 0 will set the option to the optimal size for the transport
	// protocol used when creating the client: 1432 for UDP and 8192 for UDS.
	MaxBytesPerPayload int
	// MaxMessagesPerPayload is the maximum number of metrics, events and/or service checks a single payload will contain.
	// This option can be set to `1` to create an unbuffered client.
	MaxMessagesPerPayload int
	// BufferPoolSize is the size of the pool of buffers in number of buffers.
	// The magic value 0 will set the option to the optimal size for the transport
	// protocol used when creating the client: 2048 for UDP and 512 for UDS.
	BufferPoolSize int
	// BufferFlushInterval is the interval after which the current buffer will get flushed.
	BufferFlushInterval time.Duration
	// BufferShardCount is the number of buffer "shards" that will be used.
	// Those shards allows the use of multiple buffers at the same time to reduce
	// lock contention.
	BufferShardCount int
	// SenderQueueSize is the size of the sender queue in number of buffers.
	// The magic value 0 will set the option to the optimal size for the transport
	// protocol used when creating the client: 2048 for UDP and 512 for UDS.
	SenderQueueSize int
	// WriteTimeoutUDS is the timeout after which a UDS packet is dropped.
	WriteTimeoutUDS time.Duration
	// Telemetry is a set of metrics automatically injected by the client in the
	// dogstatsd stream to be able to monitor the client itself.
	Telemetry bool
	// ReceiveMode determins the behavior of the client when receiving to many
	// metrics. The client will either drop the metrics if its buffers are
	// full (ChannelMode mode) or block the caller until the metric can be
	// handled (MutexMode mode). By default the client will MutexMode. This
	// option should be set to ChannelMode only when use under very high
	// load.
	//
	// MutexMode uses a mutex internally which is much faster than
	// channel but causes some lock contention when used with a high number
	// of threads. Mutex are sharded based on the metrics name which
	// limit mutex contention when goroutines send different metrics.
	//
	// ChannelMode: uses channel (of ChannelModeBufferSize size) to send
	// metrics and drop metrics if the channel is full. Sending metrics in
	// this mode is slower that MutexMode (because of the channel), but
	// will not block the application. This mode is made for application
	// using many goroutines, sending the same metrics at a very high
	// volume. The goal is to not slow down the application at the cost of
	// dropping metrics and having a lower max throughput.
	ReceiveMode ReceivingMode
	// ChannelModeBufferSize is the size of the channel holding incoming metrics
	ChannelModeBufferSize int
	// AggregationFlushInterval is the interval for the aggregator to flush metrics
	AggregationFlushInterval time.Duration
	// [beta] Aggregation enables/disables client side aggregation for
	// Gauges, Counts and Sets (compatible with every Agent's version).
	Aggregation bool
	// [beta] Extended aggregation enables/disables client side aggregation
	// for all types. This feature is only compatible with Agent's versions
	// >=7.25.0 or Agent's version >=6.25.0 && < 7.0.0.
	ExtendedAggregation bool
	// TelemetryAddr specify a different endpoint for telemetry metrics.
	TelemetryAddr string
	// DevMode enables the "dev" mode where the client sends much more
	// telemetry metrics to help troubleshooting the client behavior.
	DevMode bool
}

func resolveOptions(options []Option) (*Options, error) {
	o := &Options{
		Namespace:                DefaultNamespace,
		Tags:                     DefaultTags,
		MaxBytesPerPayload:       DefaultMaxBytesPerPayload,
		MaxMessagesPerPayload:    DefaultMaxMessagesPerPayload,
		BufferPoolSize:           DefaultBufferPoolSize,
		BufferFlushInterval:      DefaultBufferFlushInterval,
		BufferShardCount:         DefaultBufferShardCount,
		SenderQueueSize:          DefaultSenderQueueSize,
		WriteTimeoutUDS:          DefaultWriteTimeoutUDS,
		Telemetry:                DefaultTelemetry,
		ReceiveMode:              DefaultReceivingMode,
		ChannelModeBufferSize:    DefaultChannelModeBufferSize,
		AggregationFlushInterval: DefaultAggregationFlushInterval,
		Aggregation:              DefaultAggregation,
		ExtendedAggregation:      DefaultExtendedAggregation,
		DevMode:                  DefaultDevMode,
	}

	for _, option := range options {
		err := option(o)
		if err != nil {
			return nil, err
		}
	}

	return o, nil
}

// Option is a client option. Can return an error if validation fails.
type Option func(*Options) error

// WithNamespace sets the Namespace option.
func WithNamespace(namespace string) Option {
	return func(o *Options) error {
		if strings.HasSuffix(namespace, ".") {
			o.Namespace = namespace
		} else {
			o.Namespace = namespace + "."
		}
		return nil
	}
}

// WithTags sets the Tags option.
func WithTags(tags []string) Option {
	return func(o *Options) error {
		o.Tags = tags
		return nil
	}
}

// WithMaxMessagesPerPayload sets the MaxMessagesPerPayload option.
func WithMaxMessagesPerPayload(maxMessagesPerPayload int) Option {
	return func(o *Options) error {
		o.MaxMessagesPerPayload = maxMessagesPerPayload
		return nil
	}
}

// WithMaxBytesPerPayload sets the MaxBytesPerPayload option.
func WithMaxBytesPerPayload(MaxBytesPerPayload int) Option {
	return func(o *Options) error {
		o.MaxBytesPerPayload = MaxBytesPerPayload
		return nil
	}
}

// WithBufferPoolSize sets the BufferPoolSize option.
func WithBufferPoolSize(bufferPoolSize int) Option {
	return func(o *Options) error {
		o.BufferPoolSize = bufferPoolSize
		return nil
	}
}

// WithBufferFlushInterval sets the BufferFlushInterval option.
func WithBufferFlushInterval(bufferFlushInterval time.Duration) Option {
	return func(o *Options) error {
		o.BufferFlushInterval = bufferFlushInterval
		return nil
	}
}

// WithBufferShardCount sets the BufferShardCount option.
func WithBufferShardCount(bufferShardCount int) Option {
	return func(o *Options) error {
		if bufferShardCount < 1 {
			return fmt.Errorf("BufferShardCount must be a positive integer")
		}
		o.BufferShardCount = bufferShardCount
		return nil
	}
}

// WithSenderQueueSize sets the SenderQueueSize option.
func WithSenderQueueSize(senderQueueSize int) Option {
	return func(o *Options) error {
		o.SenderQueueSize = senderQueueSize
		return nil
	}
}

// WithWriteTimeoutUDS sets the WriteTimeoutUDS option.
func WithWriteTimeoutUDS(writeTimeoutUDS time.Duration) Option {
	return func(o *Options) error {
		o.WriteTimeoutUDS = writeTimeoutUDS
		return nil
	}
}

// WithoutTelemetry disables the telemetry
func WithoutTelemetry() Option {
	return func(o *Options) error {
		o.Telemetry = false
		return nil
	}
}

// WithChannelMode will use channel to receive metrics
func WithChannelMode() Option {
	return func(o *Options) error {
		o.ReceiveMode = ChannelMode
		return nil
	}
}

// WithMutexMode will use mutex to receive metrics
func WithMutexMode() Option {
	return func(o *Options) error {
		o.ReceiveMode = MutexMode
		return nil
	}
}

// WithChannelModeBufferSize the channel buffer size when using "drop mode"
func WithChannelModeBufferSize(bufferSize int) Option {
	return func(o *Options) error {
		o.ChannelModeBufferSize = bufferSize
		return nil
	}
}

// WithAggregationInterval set the aggregation interval
func WithAggregationInterval(interval time.Duration) Option {
	return func(o *Options) error {
		o.AggregationFlushInterval = interval
		return nil
	}
}

// WithClientSideAggregation enables client side aggregation for Gauges, Counts
// and Sets. Client side aggregation is a beta feature.
func WithClientSideAggregation() Option {
	return func(o *Options) error {
		o.Aggregation = true
		return nil
	}
}

// WithoutClientSideAggregation disables client side aggregation.
func WithoutClientSideAggregation() Option {
	return func(o *Options) error {
		o.Aggregation = false
		o.ExtendedAggregation = false
		return nil
	}
}

// WithExtendedClientSideAggregation enables client side aggregation for all
// types. This feature is only compatible with Agent's version >=6.25.0 &&
// <7.0.0 or Agent's versions >=7.25.0. Client side aggregation is a beta
// feature.
func WithExtendedClientSideAggregation() Option {
	return func(o *Options) error {
		o.Aggregation = true
		o.ExtendedAggregation = true
		return nil
	}
}

// WithTelemetryAddr specify a different address for telemetry metrics.
func WithTelemetryAddr(addr string) Option {
	return func(o *Options) error {
		o.TelemetryAddr = addr
		return nil
	}
}

// WithDevMode enables client "dev" mode, sending more Telemetry metrics to
// help troubleshoot client behavior.
func WithDevMode() Option {
	return func(o *Options) error {
		o.DevMode = true
		return nil
	}
}

// WithoutDevMode disables client "dev" mode, sending more Telemetry metrics to
// help troubleshoot client behavior.
func WithoutDevMode() Option {
	return func(o *Options) error {
		o.DevMode = false
		return nil
	}
}
