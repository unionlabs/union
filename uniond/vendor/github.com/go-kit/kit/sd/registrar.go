package sd

// Registrar registers instance information to a service discovery system when
// an instance becomes alive and healthy, and deregisters that information when
// the service becomes unhealthy or goes away.
//
// Registrar implementations exist for various service discovery systems. Note
// that identifying instance information (e.g. host:port) must be given via the
// concrete constructor; this interface merely signals lifecycle changes.
type Registrar interface {
	Register()
	Deregister()
}
