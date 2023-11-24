// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package remote

import "github.com/pkg/errors"

// MakeSimpleFactory returns a StorageFactory implementation that produces the given
// Storage objects.
func MakeSimpleFactory(m map[Locator]Storage) StorageFactory {
	return simpleFactory(m)
}

type simpleFactory map[Locator]Storage

var _ StorageFactory = simpleFactory{}

// CreateStorage is part of the StorageFactory interface.
func (sf simpleFactory) CreateStorage(locator Locator) (Storage, error) {
	if s, ok := sf[locator]; ok {
		return s, nil
	}
	return nil, errors.Errorf("unknown locator '%s'", locator)
}
