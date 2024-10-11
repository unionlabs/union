package mock

import "github.com/prysmaticlabs/prysm/v4/async/event"

type MockStreamer struct {
	logs [][]byte
	feed *event.Feed
}

// NewMockStreamer creates a new instance of MockStreamer.
// It's useful to set up the default state for the mock, like initializing the feed.
func NewMockStreamer(logs [][]byte) *MockStreamer {
	return &MockStreamer{
		logs: logs,
		feed: new(event.Feed),
	}
}

// GetLastFewLogs returns the predefined logs.
func (m *MockStreamer) GetLastFewLogs() [][]byte {
	return m.logs
}

// LogsFeed returns the predefined event feed.
func (m *MockStreamer) LogsFeed() *event.Feed {
	return m.feed
}
