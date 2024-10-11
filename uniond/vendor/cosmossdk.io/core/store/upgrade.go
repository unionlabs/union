package store

// StoreUpgrades defines a series of transformations to apply the multistore db upon load
type StoreUpgrades struct {
	Added   []string `json:"added"`
	Deleted []string `json:"deleted"`
}

// IsAdded returns true if the given key should be added
func (s *StoreUpgrades) IsAdded(key string) bool {
	if s == nil {
		return false
	}
	for _, added := range s.Added {
		if key == added {
			return true
		}
	}
	return false
}

// IsDeleted returns true if the given key should be deleted
func (s *StoreUpgrades) IsDeleted(key string) bool {
	if s == nil {
		return false
	}
	for _, d := range s.Deleted {
		if d == key {
			return true
		}
	}
	return false
}
