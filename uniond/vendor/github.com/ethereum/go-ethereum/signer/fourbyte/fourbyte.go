// Copyright 2019 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

// Package fourbyte contains the 4byte database.
package fourbyte

import (
	_ "embed"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
)

//go:embed 4byte.json
var embeddedJSON []byte

// Database is a 4byte database with the possibility of maintaining an immutable
// set (embedded) into the process and a mutable set (loaded and written to file).
type Database struct {
	embedded   map[string]string
	custom     map[string]string
	customPath string
}

// newEmpty exists for testing purposes.
func newEmpty() *Database {
	return &Database{
		embedded: make(map[string]string),
		custom:   make(map[string]string),
	}
}

// New loads the standard signature database embedded in the package.
func New() (*Database, error) {
	return NewWithFile("")
}

// NewFromFile loads signature database from file, and errors if the file is not
// valid JSON. The constructor does no other validation of contents. This method
// does not load the embedded 4byte database.
//
// The provided path will be used to write new values into if they are submitted
// via the API.
func NewFromFile(path string) (*Database, error) {
	raw, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer raw.Close()

	db := newEmpty()
	if err := json.NewDecoder(raw).Decode(&db.embedded); err != nil {
		return nil, err
	}
	return db, nil
}

// NewWithFile loads both the standard signature database (embedded resource
// file) as well as a custom database. The latter will be used to write new
// values into if they are submitted via the API.
func NewWithFile(path string) (*Database, error) {
	db := &Database{make(map[string]string), make(map[string]string), path}
	db.customPath = path

	if err := json.Unmarshal(embeddedJSON, &db.embedded); err != nil {
		return nil, err
	}
	// Custom file may not exist. Will be created during save, if needed.
	if _, err := os.Stat(path); err == nil {
		var blob []byte
		if blob, err = os.ReadFile(path); err != nil {
			return nil, err
		}
		if err := json.Unmarshal(blob, &db.custom); err != nil {
			return nil, err
		}
	}
	return db, nil
}

// Size returns the number of 4byte entries in the embedded and custom datasets.
func (db *Database) Size() (int, int) {
	return len(db.embedded), len(db.custom)
}

// Selector checks the given 4byte ID against the known ABI methods.
//
// This method does not validate the match, it's assumed the caller will do.
func (db *Database) Selector(id []byte) (string, error) {
	if len(id) < 4 {
		return "", fmt.Errorf("expected 4-byte id, got %d", len(id))
	}
	sig := hex.EncodeToString(id[:4])
	if selector, exists := db.embedded[sig]; exists {
		return selector, nil
	}
	if selector, exists := db.custom[sig]; exists {
		return selector, nil
	}
	return "", fmt.Errorf("signature %v not found", sig)
}

// AddSelector inserts a new 4byte entry into the database. If custom database
// saving is enabled, the new dataset is also persisted to disk.
//
// Node, this method does _not_ validate the correctness of the data. It assumes
// the caller has already done so.
func (db *Database) AddSelector(selector string, data []byte) error {
	// If the selector is already known, skip duplicating it
	if len(data) < 4 {
		return nil
	}
	if _, err := db.Selector(data[:4]); err == nil {
		return nil
	}
	// Inject the custom selector into the database and persist if needed
	db.custom[hex.EncodeToString(data[:4])] = selector
	if db.customPath == "" {
		return nil
	}
	blob, err := json.Marshal(db.custom)
	if err != nil {
		return err
	}
	return os.WriteFile(db.customPath, blob, 0600)
}
