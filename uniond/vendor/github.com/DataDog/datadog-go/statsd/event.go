package statsd

import (
	"fmt"
	"time"
)

// Events support
// EventAlertType and EventAlertPriority became exported types after this issue was submitted: https://github.com/DataDog/datadog-go/issues/41
// The reason why they got exported is so that client code can directly use the types.

// EventAlertType is the alert type for events
type EventAlertType string

const (
	// Info is the "info" AlertType for events
	Info EventAlertType = "info"
	// Error is the "error" AlertType for events
	Error EventAlertType = "error"
	// Warning is the "warning" AlertType for events
	Warning EventAlertType = "warning"
	// Success is the "success" AlertType for events
	Success EventAlertType = "success"
)

// EventPriority is the event priority for events
type EventPriority string

const (
	// Normal is the "normal" Priority for events
	Normal EventPriority = "normal"
	// Low is the "low" Priority for events
	Low EventPriority = "low"
)

// An Event is an object that can be posted to your DataDog event stream.
type Event struct {
	// Title of the event.  Required.
	Title string
	// Text is the description of the event.  Required.
	Text string
	// Timestamp is a timestamp for the event.  If not provided, the dogstatsd
	// server will set this to the current time.
	Timestamp time.Time
	// Hostname for the event.
	Hostname string
	// AggregationKey groups this event with others of the same key.
	AggregationKey string
	// Priority of the event.  Can be statsd.Low or statsd.Normal.
	Priority EventPriority
	// SourceTypeName is a source type for the event.
	SourceTypeName string
	// AlertType can be statsd.Info, statsd.Error, statsd.Warning, or statsd.Success.
	// If absent, the default value applied by the dogstatsd server is Info.
	AlertType EventAlertType
	// Tags for the event.
	Tags []string
}

// NewEvent creates a new event with the given title and text.  Error checking
// against these values is done at send-time, or upon running e.Check.
func NewEvent(title, text string) *Event {
	return &Event{
		Title: title,
		Text:  text,
	}
}

// Check verifies that an event is valid.
func (e Event) Check() error {
	if len(e.Title) == 0 {
		return fmt.Errorf("statsd.Event title is required")
	}
	if len(e.Text) == 0 {
		return fmt.Errorf("statsd.Event text is required")
	}
	return nil
}

// Encode returns the dogstatsd wire protocol representation for an event.
// Tags may be passed which will be added to the encoded output but not to
// the Event's list of tags, eg. for default tags.
func (e Event) Encode(tags ...string) (string, error) {
	err := e.Check()
	if err != nil {
		return "", err
	}
	var buffer []byte
	buffer = appendEvent(buffer, e, tags)
	return string(buffer), nil
}
