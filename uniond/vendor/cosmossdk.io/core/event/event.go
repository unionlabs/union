package event

import "cosmossdk.io/schema/appdata"

// Attribute is a kv-pair event attribute.
type Attribute = appdata.EventAttribute

func NewAttribute(key, value string) Attribute {
	return Attribute{Key: key, Value: value}
}

// Events represents a list of events.
type Events = appdata.EventData

func NewEvents(events ...Event) Events {
	return Events{Events: events}
}

// Event defines how an event will emitted
type Event = appdata.Event

func NewEvent(ty string, attrs ...Attribute) Event {
	return Event{Type: ty, Attributes: func() ([]Attribute, error) { return attrs, nil }}
}
