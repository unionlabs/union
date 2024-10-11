package p2p

import (
	"fmt"
	"reflect"
	"regexp"
	"sync"

	"github.com/go-kit/kit/metrics"
)

const (
	// MetricsSubsystem is a subsystem shared by all metrics exposed by this
	// package.
	MetricsSubsystem = "p2p"
)

// valueToLabelRegexp is used to find the golang package name and type name
// so that the name can be turned into a prometheus label where the characters
// in the label do not include prometheus special characters such as '*' and '.'.
var valueToLabelRegexp = regexp.MustCompile(`\*?(\w+)\.(.*)`)

//go:generate go run ../scripts/metricsgen -struct=Metrics

// Metrics contains metrics exposed by this package.
type Metrics struct {
	// Number of peers.
	Peers metrics.Gauge
	// Pending bytes to be sent to a given peer.
	PeerPendingSendBytes metrics.Gauge `metrics_labels:"peer_id"`
	// Number of transactions submitted by each peer.
	NumTxs metrics.Gauge `metrics_labels:"peer_id"`
	// Number of bytes of each message type received.
	MessageReceiveBytesTotal metrics.Counter `metrics_labels:"message_type"`
	// Number of bytes of each message type sent.
	MessageSendBytesTotal metrics.Counter `metrics_labels:"message_type"`
}

type peerPendingMetricsCache struct {
	mtx             sync.Mutex
	perMessageCache map[reflect.Type]*peerPendingMetricsCacheEntry
}

type peerPendingMetricsCacheEntry struct {
	label            string
	pendingSendBytes int
	pendingRecvBytes int
}

func newPeerPendingMetricsCache() *peerPendingMetricsCache {
	return &peerPendingMetricsCache{
		perMessageCache: make(map[reflect.Type]*peerPendingMetricsCacheEntry),
	}
}

func (c *peerPendingMetricsCache) AddPendingSendBytes(msgType reflect.Type, addBytes int) {
	c.mtx.Lock()
	defer c.mtx.Unlock()
	if entry, ok := c.perMessageCache[msgType]; ok {
		entry.pendingSendBytes += addBytes
	} else {
		c.perMessageCache[msgType] = &peerPendingMetricsCacheEntry{
			label:            buildLabel(msgType),
			pendingSendBytes: addBytes,
		}
	}
}

func (c *peerPendingMetricsCache) AddPendingRecvBytes(msgType reflect.Type, addBytes int) {
	c.mtx.Lock()
	defer c.mtx.Unlock()
	if entry, ok := c.perMessageCache[msgType]; ok {
		entry.pendingRecvBytes += addBytes
	} else {
		c.perMessageCache[msgType] = &peerPendingMetricsCacheEntry{
			label:            buildLabel(msgType),
			pendingRecvBytes: addBytes,
		}
	}
}

func buildLabel(msgType reflect.Type) string {
	s := msgType.String()
	ss := valueToLabelRegexp.FindStringSubmatch(s)
	return fmt.Sprintf("%s_%s", ss[1], ss[2])
}

func getMsgType(i any) reflect.Type {
	return reflect.TypeOf(i)
}
