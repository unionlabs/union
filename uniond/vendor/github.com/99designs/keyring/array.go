package keyring

// ArrayKeyring is a mock/non-secure backend that meets the Keyring interface.
// It is intended to be used to aid unit testing of code that relies on the package.
// NOTE: Do not use in production code.
type ArrayKeyring struct {
	items map[string]Item
}

// NewArrayKeyring returns an ArrayKeyring, optionally constructed with an initial slice
// of items.
func NewArrayKeyring(initial []Item) *ArrayKeyring {
	kr := &ArrayKeyring{}
	for _, i := range initial {
		_ = kr.Set(i)
	}
	return kr
}

// Get returns an Item matching Key.
func (k *ArrayKeyring) Get(key string) (Item, error) {
	if i, ok := k.items[key]; ok {
		return i, nil
	}
	return Item{}, ErrKeyNotFound
}

// Set will store an item on the mock Keyring.
func (k *ArrayKeyring) Set(i Item) error {
	if k.items == nil {
		k.items = map[string]Item{}
	}
	k.items[i.Key] = i
	return nil
}

// Remove will delete an Item from the Keyring.
func (k *ArrayKeyring) Remove(key string) error {
	delete(k.items, key)
	return nil
}

// Keys provides a slice of all Item keys on the Keyring.
func (k *ArrayKeyring) Keys() ([]string, error) {
	var keys = []string{}
	for key := range k.items {
		keys = append(keys, key)
	}
	return keys, nil
}

func (k *ArrayKeyring) GetMetadata(_ string) (Metadata, error) {
	return Metadata{}, ErrMetadataNeedsCredentials
}
