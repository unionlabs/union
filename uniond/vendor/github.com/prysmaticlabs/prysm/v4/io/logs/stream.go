package logs

import (
	"io"

	lru "github.com/hashicorp/golang-lru"
	"github.com/prysmaticlabs/prysm/v4/async/event"
	lruwrpr "github.com/prysmaticlabs/prysm/v4/cache/lru"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
)

const (
	// The number of log entries to keep in memory.
	logCacheSize = 20
)

var (
	// Compile time interface checks.
	_ = io.Writer(&StreamServer{})
	_ = Streamer(&StreamServer{})
)

// Streamer defines a struct which can retrieve and stream process logs.
type Streamer interface {
	GetLastFewLogs() [][]byte
	LogsFeed() *event.Feed
}

// StreamServer defines a websocket server which can receive events from
// a feed and write them to open websocket connections.
type StreamServer struct {
	feed  *event.Feed
	cache *lru.Cache
}

// NewStreamServer initializes a new stream server capable of
// streaming log events.
func NewStreamServer() *StreamServer {
	ss := &StreamServer{
		feed:  new(event.Feed),
		cache: lruwrpr.New(logCacheSize),
	}
	addLogWriter(ss)
	return ss
}

// GetLastFewLogs returns the last few entries of logs stored in an LRU cache.
func (ss *StreamServer) GetLastFewLogs() [][]byte {
	messages := make([][]byte, 0)
	for _, k := range ss.cache.Keys() {
		d, ok := ss.cache.Get(k)
		if ok {
			messages = append(messages, d.([]byte))
		}
	}
	return messages
}

// LogsFeed returns a feed callers can subscribe to receive logs via a channel.
func (ss *StreamServer) LogsFeed() *event.Feed {
	return ss.feed
}

// Write a binary message and send over the event feed.
func (ss *StreamServer) Write(p []byte) (n int, err error) {
	ss.feed.Send(p)
	ss.cache.Add(rand.NewGenerator().Uint64(), p)
	return len(p), nil
}
