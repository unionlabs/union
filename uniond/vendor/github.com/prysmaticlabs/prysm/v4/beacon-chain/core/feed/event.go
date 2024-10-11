// Package feed defines event feed types for inter-service communication
// during a beacon node's runtime.
package feed

// How to add a new event to the feed:
//   1. Add a file for the new type of feed.
//   2. Add a constant describing the list of events.
//   3. Add a structure with the name `<event>Data` containing any data fields that should be supplied with the event.
//
// Note that the same event is supplied to all subscribers, so the event received by subscribers should be considered read-only.

// EventType is the type that defines the type of event.
type EventType int

// Event is the event that is sent with operation feed updates.
type Event struct {
	// Type is the type of event.
	Type EventType
	// Data is event-specific data.
	Data interface{}
}
