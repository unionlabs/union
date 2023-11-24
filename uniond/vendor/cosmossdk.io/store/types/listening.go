package types

// MemoryListener listens to the state writes and accumulate the records in memory.
type MemoryListener struct {
	stateCache []*StoreKVPair
}

// NewMemoryListener creates a listener that accumulate the state writes in memory.
func NewMemoryListener() *MemoryListener {
	return &MemoryListener{}
}

// OnWrite implements MemoryListener interface
func (fl *MemoryListener) OnWrite(storeKey StoreKey, key, value []byte, delete bool) {
	fl.stateCache = append(fl.stateCache, &StoreKVPair{
		StoreKey: storeKey.Name(),
		Delete:   delete,
		Key:      key,
		Value:    value,
	})
}

// PopStateCache returns the current state caches and set to nil
func (fl *MemoryListener) PopStateCache() []*StoreKVPair {
	res := fl.stateCache
	fl.stateCache = nil
	return res
}
