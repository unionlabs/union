package statsd

import (
	"math"
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
	// DefaultSenderQueueSize is the default value for the DefaultSenderQueueSize option
	DefaultSenderQueueSize = 0
	// DefaultWriteTimeoutUDS is the default value for the WriteTimeoutUDS option
	DefaultWriteTimeoutUDS = 1 * time.Millisecond
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
	// SenderQueueSize is the size of the sender queue in number of buffers.
	// The magic value 0 will set the option to the optimal size for the transport
	// protocol used when creating the client: 2048 for UDP and 512 for UDS.
	SenderQueueSize int
	// WriteTimeoutUDS is the timeout after which a UDS packet is dropped.
	WriteTimeoutUDS time.Duration
}

func resolveOptions(options []Option) (*Options, error) {
	o := &Options{
		Namespace:             DefaultNamespace,
		Tags:                  DefaultTags,
		MaxBytesPerPayload:    DefaultMaxBytesPerPayload,
		MaxMessagesPerPayload: DefaultMaxMessagesPerPayload,
		BufferPoolSize:        DefaultBufferPoolSize,
		BufferFlushInterval:   DefaultBufferFlushInterval,
		SenderQueueSize:       DefaultSenderQueueSize,
		WriteTimeoutUDS:       DefaultWriteTimeoutUDS,
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
		o.Namespace = namespace
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
