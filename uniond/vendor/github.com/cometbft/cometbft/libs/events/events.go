// Package events - Pub-Sub in go with event caching
package events

import (
	"fmt"

	"github.com/cometbft/cometbft/libs/service"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
)

// ErrListenerWasRemoved is returned by AddEvent if the listener was removed.
type ErrListenerWasRemoved struct {
	listenerID string
}

// Error implements the error interface.
func (e ErrListenerWasRemoved) Error() string {
	return fmt.Sprintf("listener #%s was removed", e.listenerID)
}

// EventData is a generic event data can be typed and registered with
// tendermint/go-amino via concrete implementation of this interface.
type EventData interface{}

// Eventable is the interface reactors and other modules must export to become
// eventable.
type Eventable interface {
	SetEventSwitch(evsw EventSwitch)
}

// Fireable is the interface that wraps the FireEvent method.
//
// FireEvent fires an event with the given name and data.
type Fireable interface {
	FireEvent(event string, data EventData)
}

// EventSwitch is the interface for synchronous pubsub, where listeners
// subscribe to certain events and, when an event is fired (see Fireable),
// notified via a callback function.
//
// Listeners are added by calling AddListenerForEvent function.
// They can be removed by calling either RemoveListenerForEvent or
// RemoveListener (for all events).
type EventSwitch interface {
	service.Service
	Fireable

	AddListenerForEvent(listenerID, event string, cb EventCallback) error
	RemoveListenerForEvent(event string, listenerID string)
	RemoveListener(listenerID string)
}

type eventSwitch struct {
	service.BaseService

	mtx        cmtsync.RWMutex
	eventCells map[string]*eventCell
	listeners  map[string]*eventListener
}

func NewEventSwitch() EventSwitch {
	evsw := &eventSwitch{
		eventCells: make(map[string]*eventCell),
		listeners:  make(map[string]*eventListener),
	}
	evsw.BaseService = *service.NewBaseService(nil, "EventSwitch", evsw)
	return evsw
}

func (evsw *eventSwitch) OnStart() error {
	return nil
}

func (evsw *eventSwitch) OnStop() {}

func (evsw *eventSwitch) AddListenerForEvent(listenerID, event string, cb EventCallback) error {
	// Get/Create eventCell and listener.
	evsw.mtx.Lock()
	eventCell := evsw.eventCells[event]
	if eventCell == nil {
		eventCell = newEventCell()
		evsw.eventCells[event] = eventCell
	}
	listener := evsw.listeners[listenerID]
	if listener == nil {
		listener = newEventListener(listenerID)
		evsw.listeners[listenerID] = listener
	}
	evsw.mtx.Unlock()

	// Add event and listener.
	if err := listener.AddEvent(event); err != nil {
		return err
	}
	eventCell.AddListener(listenerID, cb)

	return nil
}

func (evsw *eventSwitch) RemoveListener(listenerID string) {
	// Get and remove listener.
	evsw.mtx.RLock()
	listener := evsw.listeners[listenerID]
	evsw.mtx.RUnlock()
	if listener == nil {
		return
	}

	evsw.mtx.Lock()
	delete(evsw.listeners, listenerID)
	evsw.mtx.Unlock()

	// Remove callback for each event.
	listener.SetRemoved()
	for _, event := range listener.GetEvents() {
		evsw.RemoveListenerForEvent(event, listenerID)
	}
}

func (evsw *eventSwitch) RemoveListenerForEvent(event string, listenerID string) {
	// Get eventCell
	evsw.mtx.Lock()
	eventCell := evsw.eventCells[event]
	evsw.mtx.Unlock()

	if eventCell == nil {
		return
	}

	// Remove listenerID from eventCell
	numListeners := eventCell.RemoveListener(listenerID)

	// Maybe garbage collect eventCell.
	if numListeners == 0 {
		// Lock again and double check.
		evsw.mtx.Lock()      // OUTER LOCK
		eventCell.mtx.Lock() // INNER LOCK
		if len(eventCell.listeners) == 0 {
			delete(evsw.eventCells, event)
		}
		eventCell.mtx.Unlock() // INNER LOCK
		evsw.mtx.Unlock()      // OUTER LOCK
	}
}

func (evsw *eventSwitch) FireEvent(event string, data EventData) {
	// Get the eventCell
	evsw.mtx.RLock()
	eventCell := evsw.eventCells[event]
	evsw.mtx.RUnlock()

	if eventCell == nil {
		return
	}

	// Fire event for all listeners in eventCell
	eventCell.FireEvent(data)
}

//-----------------------------------------------------------------------------

// eventCell handles keeping track of listener callbacks for a given event.
type eventCell struct {
	mtx       cmtsync.RWMutex
	listeners map[string]EventCallback
}

func newEventCell() *eventCell {
	return &eventCell{
		listeners: make(map[string]EventCallback),
	}
}

func (cell *eventCell) AddListener(listenerID string, cb EventCallback) {
	cell.mtx.Lock()
	cell.listeners[listenerID] = cb
	cell.mtx.Unlock()
}

func (cell *eventCell) RemoveListener(listenerID string) int {
	cell.mtx.Lock()
	delete(cell.listeners, listenerID)
	numListeners := len(cell.listeners)
	cell.mtx.Unlock()
	return numListeners
}

func (cell *eventCell) FireEvent(data EventData) {
	cell.mtx.RLock()
	eventCallbacks := make([]EventCallback, 0, len(cell.listeners))
	for _, cb := range cell.listeners {
		eventCallbacks = append(eventCallbacks, cb)
	}
	cell.mtx.RUnlock()

	for _, cb := range eventCallbacks {
		cb(data)
	}
}

//-----------------------------------------------------------------------------

type EventCallback func(data EventData)

type eventListener struct {
	id string

	mtx     cmtsync.RWMutex
	removed bool
	events  []string
}

func newEventListener(id string) *eventListener {
	return &eventListener{
		id:      id,
		removed: false,
		events:  nil,
	}
}

func (evl *eventListener) AddEvent(event string) error {
	evl.mtx.Lock()

	if evl.removed {
		evl.mtx.Unlock()
		return ErrListenerWasRemoved{listenerID: evl.id}
	}

	evl.events = append(evl.events, event)
	evl.mtx.Unlock()
	return nil
}

func (evl *eventListener) GetEvents() []string {
	evl.mtx.RLock()
	events := make([]string, len(evl.events))
	copy(events, evl.events)
	evl.mtx.RUnlock()
	return events
}

func (evl *eventListener) SetRemoved() {
	evl.mtx.Lock()
	evl.removed = true
	evl.mtx.Unlock()
}
