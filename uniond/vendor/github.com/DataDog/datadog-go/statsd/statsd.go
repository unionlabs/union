// Copyright 2013 Ooyala, Inc.

/*
Package statsd provides a Go dogstatsd client. Dogstatsd extends the popular statsd,
adding tags and histograms and pushing upstream to Datadog.

Refer to http://docs.datadoghq.com/guides/dogstatsd/ for information about DogStatsD.

Example Usage:

    // Create the client
    c, err := statsd.New("127.0.0.1:8125")
    if err != nil {
        log.Fatal(err)
    }
    // Prefix every metric with the app name
    c.Namespace = "flubber."
    // Send the EC2 availability zone as a tag with every metric
    c.Tags = append(c.Tags, "us-east-1a")
    err = c.Gauge("request.duration", 1.2, nil, 1)

statsd is based on go-statsd-client.
*/
package statsd

import (
	"fmt"
	"math/rand"
	"os"
	"strings"
	"sync"
	"time"
)

/*
OptimalUDPPayloadSize defines the optimal payload size for a UDP datagram, 1432 bytes
is optimal for regular networks with an MTU of 1500 so datagrams don't get
fragmented. It's generally recommended not to fragment UDP datagrams as losing
a single fragment will cause the entire datagram to be lost.
*/
const OptimalUDPPayloadSize = 1432

/*
MaxUDPPayloadSize defines the maximum payload size for a UDP datagram.
Its value comes from the calculation: 65535 bytes Max UDP datagram size -
8byte UDP header - 60byte max IP headers
any number greater than that will see frames being cut out.
*/
const MaxUDPPayloadSize = 65467

// DefaultUDPBufferPoolSize is the default size of the buffer pool for UDP clients.
const DefaultUDPBufferPoolSize = 2048

// DefaultUDSBufferPoolSize is the default size of the buffer pool for UDS clients.
const DefaultUDSBufferPoolSize = 512

/*
DefaultMaxAgentPayloadSize is the default maximum payload size the agent
can receive. This can be adjusted by changing dogstatsd_buffer_size in the
agent configuration file datadog.yaml.
*/
const DefaultMaxAgentPayloadSize = 8192

/*
TelemetryInterval is the interval at which telemetry will be sent by the client.
*/
const TelemetryInterval = 10 * time.Second

/*
clientTelemetryTag is a tag identifying this specific client.
*/
var clientTelemetryTag = "client:go"

/*
UnixAddressPrefix holds the prefix to use to enable Unix Domain Socket
traffic instead of UDP.
*/
const UnixAddressPrefix = "unix://"

// Client-side entity ID injection for container tagging
const (
	entityIDEnvName = "DD_ENTITY_ID"
	entityIDTagName = "dd.internal.entity_id"
)

type metricType int

const (
	gauge metricType = iota
	count
	histogram
	distribution
	set
	timing
	event
	serviceCheck
)

type metric struct {
	metricType metricType
	namespace  string
	globalTags []string
	name       string
	fvalue     float64
	ivalue     int64
	svalue     string
	evalue     *Event
	scvalue    *ServiceCheck
	tags       []string
	rate       float64
}

type noClientErr string

// ErrNoClient is returned if statsd reporting methods are invoked on
// a nil client.
const ErrNoClient = noClientErr("statsd client is nil")

func (e noClientErr) Error() string {
	return string(e)
}

// ClientInterface is an interface that exposes the common client functions for the
// purpose of being able to provide a no-op client or even mocking. This can aid
// downstream users' with their testing.
type ClientInterface interface {
	// Gauge measures the value of a metric at a particular time.
	Gauge(name string, value float64, tags []string, rate float64) error

	// Count tracks how many times something happened per second.
	Count(name string, value int64, tags []string, rate float64) error

	// Histogram tracks the statistical distribution of a set of values on each host.
	Histogram(name string, value float64, tags []string, rate float64) error

	// Distribution tracks the statistical distribution of a set of values across your infrastructure.
	Distribution(name string, value float64, tags []string, rate float64) error

	// Decr is just Count of -1
	Decr(name string, tags []string, rate float64) error

	// Incr is just Count of 1
	Incr(name string, tags []string, rate float64) error

	// Set counts the number of unique elements in a group.
	Set(name string, value string, tags []string, rate float64) error

	// Timing sends timing information, it is an alias for TimeInMilliseconds
	Timing(name string, value time.Duration, tags []string, rate float64) error

	// TimeInMilliseconds sends timing information in milliseconds.
	// It is flushed by statsd with percentiles, mean and other info (https://github.com/etsy/statsd/blob/master/docs/metric_types.md#timing)
	TimeInMilliseconds(name string, value float64, tags []string, rate float64) error

	// Event sends the provided Event.
	Event(e *Event) error

	// SimpleEvent sends an event with the provided title and text.
	SimpleEvent(title, text string) error

	// ServiceCheck sends the provided ServiceCheck.
	ServiceCheck(sc *ServiceCheck) error

	// SimpleServiceCheck sends an serviceCheck with the provided name and status.
	SimpleServiceCheck(name string, status ServiceCheckStatus) error

	// Close the client connection.
	Close() error

	// Flush forces a flush of all the queued dogstatsd payloads.
	Flush() error

	// SetWriteTimeout allows the user to set a custom write timeout.
	SetWriteTimeout(d time.Duration) error
}

// A Client is a handle for sending messages to dogstatsd.  It is safe to
// use one Client from multiple goroutines simultaneously.
type Client struct {
	// Sender handles the underlying networking protocol
	sender *sender
	// Namespace to prepend to all statsd calls
	Namespace string
	// Tags are global tags to be added to every statsd call
	Tags []string
	// skipErrors turns off error passing and allows UDS to emulate UDP behaviour
	SkipErrors    bool
	flushTime     time.Duration
	bufferPool    *bufferPool
	buffer        *statsdBuffer
	telemetryTags []string
	stop          chan struct{}
	sync.Mutex
}

// Verify that Client implements the ClientInterface.
// https://golang.org/doc/faq#guarantee_satisfies_interface
var _ ClientInterface = &Client{}

// New returns a pointer to a new Client given an addr in the format "hostname:port" or
// "unix:///path/to/socket".
func New(addr string, options ...Option) (*Client, error) {
	var w statsdWriter
	o, err := resolveOptions(options)
	if err != nil {
		return nil, err
	}

	var writerType string
	optimalPayloadSize := OptimalUDPPayloadSize
	defaultBufferPoolSize := DefaultUDPBufferPoolSize
	if !strings.HasPrefix(addr, UnixAddressPrefix) {
		w, err = newUDPWriter(addr)
		writerType = "udp"
	} else {
		// FIXME: The agent has a performance pitfall preventing us from using better defaults here.
		// Once it's fixed, use `DefaultMaxAgentPayloadSize` and `DefaultUDSBufferPoolSize` instead.
		optimalPayloadSize = OptimalUDPPayloadSize
		defaultBufferPoolSize = DefaultUDPBufferPoolSize
		w, err = newUDSWriter(addr[len(UnixAddressPrefix)-1:])
		writerType = "uds"
	}
	if err != nil {
		return nil, err
	}

	if o.MaxBytesPerPayload == 0 {
		o.MaxBytesPerPayload = optimalPayloadSize
	}
	if o.BufferPoolSize == 0 {
		o.BufferPoolSize = defaultBufferPoolSize
	}
	if o.SenderQueueSize == 0 {
		o.SenderQueueSize = defaultBufferPoolSize
	}
	return newWithWriter(w, o, writerType)
}

// NewWithWriter creates a new Client with given writer. Writer is a
// io.WriteCloser + SetWriteTimeout(time.Duration) error
func NewWithWriter(w statsdWriter, options ...Option) (*Client, error) {
	o, err := resolveOptions(options)
	if err != nil {
		return nil, err
	}
	return newWithWriter(w, o, "custom")
}

func newWithWriter(w statsdWriter, o *Options, writerName string) (*Client, error) {

	w.SetWriteTimeout(o.WriteTimeoutUDS)

	c := Client{
		Namespace:     o.Namespace,
		Tags:          o.Tags,
		telemetryTags: []string{clientTelemetryTag, "transport:" + writerName},
	}

	// Inject DD_ENTITY_ID as a constant tag if found
	entityID := os.Getenv(entityIDEnvName)
	if entityID != "" {
		entityTag := fmt.Sprintf("%s:%s", entityIDTagName, entityID)
		c.Tags = append(c.Tags, entityTag)
	}

	if o.MaxBytesPerPayload == 0 {
		o.MaxBytesPerPayload = OptimalUDPPayloadSize
	}

	c.bufferPool = newBufferPool(o.BufferPoolSize, o.MaxBytesPerPayload, o.MaxMessagesPerPayload)
	c.buffer = c.bufferPool.borrowBuffer()
	c.sender = newSender(w, o.SenderQueueSize, c.bufferPool)
	c.flushTime = o.BufferFlushInterval
	c.stop = make(chan struct{}, 1)
	go c.watch()
	go c.telemetry()

	return &c, nil
}

// NewBuffered returns a Client that buffers its output and sends it in chunks.
// Buflen is the length of the buffer in number of commands.
//
// When addr is empty, the client will default to a UDP client and use the DD_AGENT_HOST
// and (optionally) the DD_DOGSTATSD_PORT environment variables to build the target address.
func NewBuffered(addr string, buflen int) (*Client, error) {
	return New(addr, WithMaxMessagesPerPayload(buflen))
}

// SetWriteTimeout allows the user to set a custom UDS write timeout. Not supported for UDP.
func (c *Client) SetWriteTimeout(d time.Duration) error {
	if c == nil {
		return ErrNoClient
	}
	return c.sender.transport.SetWriteTimeout(d)
}

func (c *Client) watch() {
	ticker := time.NewTicker(c.flushTime)

	for {
		select {
		case <-ticker.C:
			c.Lock()
			c.flushUnsafe()
			c.Unlock()
		case <-c.stop:
			ticker.Stop()
			return
		}
	}
}

func (c *Client) telemetry() {
	ticker := time.NewTicker(TelemetryInterval)
	for {
		select {
		case <-ticker.C:
			metrics := c.sender.flushMetrics()
			c.telemetryCount("datadog.dogstatsd.client.packets_sent", int64(metrics.TotalSentPayloads), c.telemetryTags, 1)
			c.telemetryCount("datadog.dogstatsd.client.bytes_sent", int64(metrics.TotalSentBytes), c.telemetryTags, 1)
			c.telemetryCount("datadog.dogstatsd.client.packets_dropped", int64(metrics.TotalDroppedPayloads), c.telemetryTags, 1)
			c.telemetryCount("datadog.dogstatsd.client.bytes_dropped", int64(metrics.TotalDroppedBytes), c.telemetryTags, 1)
			c.telemetryCount("datadog.dogstatsd.client.packets_dropped_queue", int64(metrics.TotalDroppedPayloadsQueueFull), c.telemetryTags, 1)
			c.telemetryCount("datadog.dogstatsd.client.bytes_dropped_queue", int64(metrics.TotalDroppedBytesQueueFull), c.telemetryTags, 1)
			c.telemetryCount("datadog.dogstatsd.client.packets_dropped_writer", int64(metrics.TotalDroppedPayloadsWriter), c.telemetryTags, 1)
			c.telemetryCount("datadog.dogstatsd.client.bytes_dropped_writer", int64(metrics.TotalDroppedBytesWriter), c.telemetryTags, 1)
		case <-c.stop:
			ticker.Stop()
			return
		}
	}
}

// same as Count but without global namespace / tags
func (c *Client) telemetryCount(name string, value int64, tags []string, rate float64) {
	c.addMetric(metric{metricType: count, name: name, ivalue: value, tags: tags, rate: rate})
}

// Flush forces a flush of all the queued dogstatsd payloads
// This method is blocking and will not return until everything is sent
// through the network
func (c *Client) Flush() error {
	if c == nil {
		return ErrNoClient
	}
	c.Lock()
	defer c.Unlock()
	c.flushUnsafe()
	c.sender.flush()
	return nil
}

// flush the current buffer. Lock must be held by caller.
// flushed buffer written to the network asynchronously.
func (c *Client) flushUnsafe() {
	if len(c.buffer.bytes()) > 0 {
		c.sender.send(c.buffer)
		c.buffer = c.bufferPool.borrowBuffer()
	}
}

func (c *Client) shouldSample(rate float64) bool {
	if rate < 1 && rand.Float64() > rate {
		return true
	}
	return false
}

func (c *Client) globalTags() []string {
	if c != nil {
		return c.Tags
	}
	return nil
}

func (c *Client) namespace() string {
	if c != nil {
		return c.Namespace
	}
	return ""
}

func (c *Client) writeMetricUnsafe(m metric) error {
	switch m.metricType {
	case gauge:
		return c.buffer.writeGauge(m.namespace, m.globalTags, m.name, m.fvalue, m.tags, m.rate)
	case count:
		return c.buffer.writeCount(m.namespace, m.globalTags, m.name, m.ivalue, m.tags, m.rate)
	case histogram:
		return c.buffer.writeHistogram(m.namespace, m.globalTags, m.name, m.fvalue, m.tags, m.rate)
	case distribution:
		return c.buffer.writeDistribution(m.namespace, m.globalTags, m.name, m.fvalue, m.tags, m.rate)
	case set:
		return c.buffer.writeSet(m.namespace, m.globalTags, m.name, m.svalue, m.tags, m.rate)
	case timing:
		return c.buffer.writeTiming(m.namespace, m.globalTags, m.name, m.fvalue, m.tags, m.rate)
	case event:
		return c.buffer.writeEvent(*m.evalue, m.globalTags)
	case serviceCheck:
		return c.buffer.writeServiceCheck(*m.scvalue, m.globalTags)
	default:
		return nil
	}
}

func (c *Client) addMetric(m metric) error {
	if c == nil {
		return ErrNoClient
	}
	if c.shouldSample(m.rate) {
		return nil
	}
	c.Lock()
	var err error
	if err = c.writeMetricUnsafe(m); err == errBufferFull {
		c.flushUnsafe()
		err = c.writeMetricUnsafe(m)
	}
	c.Unlock()
	return err
}

// Gauge measures the value of a metric at a particular time.
func (c *Client) Gauge(name string, value float64, tags []string, rate float64) error {
	return c.addMetric(metric{namespace: c.namespace(), globalTags: c.globalTags(), metricType: gauge, name: name, fvalue: value, tags: tags, rate: rate})
}

// Count tracks how many times something happened per second.
func (c *Client) Count(name string, value int64, tags []string, rate float64) error {
	return c.addMetric(metric{namespace: c.namespace(), globalTags: c.globalTags(), metricType: count, name: name, ivalue: value, tags: tags, rate: rate})
}

// Histogram tracks the statistical distribution of a set of values on each host.
func (c *Client) Histogram(name string, value float64, tags []string, rate float64) error {
	return c.addMetric(metric{namespace: c.namespace(), globalTags: c.globalTags(), metricType: histogram, name: name, fvalue: value, tags: tags, rate: rate})
}

// Distribution tracks the statistical distribution of a set of values across your infrastructure.
func (c *Client) Distribution(name string, value float64, tags []string, rate float64) error {
	return c.addMetric(metric{namespace: c.namespace(), globalTags: c.globalTags(), metricType: distribution, name: name, fvalue: value, tags: tags, rate: rate})
}

// Decr is just Count of -1
func (c *Client) Decr(name string, tags []string, rate float64) error {
	return c.Count(name, -1, tags, rate)
}

// Incr is just Count of 1
func (c *Client) Incr(name string, tags []string, rate float64) error {
	return c.Count(name, 1, tags, rate)
}

// Set counts the number of unique elements in a group.
func (c *Client) Set(name string, value string, tags []string, rate float64) error {
	return c.addMetric(metric{namespace: c.namespace(), globalTags: c.globalTags(), metricType: set, name: name, svalue: value, tags: tags, rate: rate})
}

// Timing sends timing information, it is an alias for TimeInMilliseconds
func (c *Client) Timing(name string, value time.Duration, tags []string, rate float64) error {
	return c.TimeInMilliseconds(name, value.Seconds()*1000, tags, rate)
}

// TimeInMilliseconds sends timing information in milliseconds.
// It is flushed by statsd with percentiles, mean and other info (https://github.com/etsy/statsd/blob/master/docs/metric_types.md#timing)
func (c *Client) TimeInMilliseconds(name string, value float64, tags []string, rate float64) error {
	return c.addMetric(metric{namespace: c.namespace(), globalTags: c.globalTags(), metricType: timing, name: name, fvalue: value, tags: tags, rate: rate})
}

// Event sends the provided Event.
func (c *Client) Event(e *Event) error {
	return c.addMetric(metric{globalTags: c.globalTags(), metricType: event, evalue: e, rate: 1})
}

// SimpleEvent sends an event with the provided title and text.
func (c *Client) SimpleEvent(title, text string) error {
	e := NewEvent(title, text)
	return c.Event(e)
}

// ServiceCheck sends the provided ServiceCheck.
func (c *Client) ServiceCheck(sc *ServiceCheck) error {
	return c.addMetric(metric{globalTags: c.globalTags(), metricType: serviceCheck, scvalue: sc, rate: 1})
}

// SimpleServiceCheck sends an serviceCheck with the provided name and status.
func (c *Client) SimpleServiceCheck(name string, status ServiceCheckStatus) error {
	sc := NewServiceCheck(name, status)
	return c.ServiceCheck(sc)
}

// Close the client connection.
func (c *Client) Close() error {
	if c == nil {
		return ErrNoClient
	}
	select {
	case c.stop <- struct{}{}:
	default:
	}
	c.Flush()
	return c.sender.close()
}
