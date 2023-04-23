package events

// An EventCache buffers events for a Fireable
// All events are cached. Filtering happens on Flush
type EventCache struct {
	evsw   Fireable
	events []eventInfo
}

// Create a new EventCache with an EventSwitch as backend
func NewEventCache(evsw Fireable) *EventCache {
	return &EventCache{
		evsw: evsw,
	}
}

// a cached event
type eventInfo struct {
	event string
	data  EventData
}

// Cache an event to be fired upon finality.
func (evc *EventCache) FireEvent(event string, data EventData) {
	// append to list (go will grow our backing array exponentially)
	evc.events = append(evc.events, eventInfo{event, data})
}

// Fire events by running evsw.FireEvent on all cached events. Blocks.
// Clears cached events
func (evc *EventCache) Flush() {
	for _, ei := range evc.events {
		evc.evsw.FireEvent(ei.event, ei.data)
	}
	// Clear the buffer, since we only add to it with append it's safe to just set it to nil and maybe safe an allocation
	evc.events = nil
}
