// Package testing includes useful mocks for testing initial
// sync status in unit tests.
package testing

// Sync defines a mock for the sync service.
type Sync struct {
	IsSyncing     bool
	IsInitialized bool
	IsSynced      bool
}

// Syncing --
func (s *Sync) Syncing() bool {
	return s.IsSyncing
}

// Initialized --
func (s *Sync) Initialized() bool {
	return s.IsInitialized
}

// Status --
func (_ *Sync) Status() error {
	return nil
}

// Resync --
func (_ *Sync) Resync() error {
	return nil
}

// Synced --
func (s *Sync) Synced() bool {
	return s.IsSynced
}
