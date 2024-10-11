package sd

// Event represents a push notification generated from the underlying service discovery
// implementation. It contains either a full set of available resource instances, or
// an error indicating some issue with obtaining information from discovery backend.
// Examples of errors may include loosing connection to the discovery backend, or
// trying to look up resource instances using an incorrectly formatted key.
// After receiving an Event with an error the listenter should treat previously discovered
// resource instances as stale (although it may choose to continue using them).
// If the Instancer is able to restore connection to the discovery backend it must push
// another Event with the current set of resource instances.
type Event struct {
	Instances []string
	Err       error
}

// Instancer listens to a service discovery system and notifies registered
// observers of changes in the resource instances. Every event sent to the channels
// contains a complete set of instances known to the Instancer. That complete set is
// sent immediately upon registering the channel, and on any future updates from
// discovery system.
type Instancer interface {
	Register(chan<- Event)
	Deregister(chan<- Event)
	Stop()
}

// FixedInstancer yields a fixed set of instances.
type FixedInstancer []string

// Register implements Instancer.
func (d FixedInstancer) Register(ch chan<- Event) { ch <- Event{Instances: d} }

// Deregister implements Instancer.
func (d FixedInstancer) Deregister(ch chan<- Event) {}

// Stop implements Instancer.
func (d FixedInstancer) Stop() {}
