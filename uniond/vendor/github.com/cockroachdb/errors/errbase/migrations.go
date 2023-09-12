// Copyright 2019 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package errbase

import "fmt"

// This file provides the library with the ability to handle cases
// where an error type migrates, i.e. its package changes path or the
// type name is changed.
//
// There are several scenarios to contend with. Assuming the error
// type is initially called "foo", in version v1 of the code.
//
// Scenario 1: simple migration
// - v2 renames foo -> bar
//   v2 calls: RegisterTypeMigration("foo", &bar{})
// - v2 and v1 are connected
// - v1 sends an error to v2:
//   - v2 has the migration registered, recognizes that "foo"
//     refers to bar
// - v2 sends an error to v1
//   - v2 rewrites the error key upon send to the name known to v1
//
// Scenario 2: simultaneous migration
// - vA renames foo -> bar
//   vA calls RegisterTypeMigration("foo", &bar{})
// - vB renames foo -> qux
//   vB calls RegisterTypeMigration("foo", &qux{})
// - vA and vB are connected
// - vA sends an error to vB:
//   - vA translates the error key upon send from bar to foo's key
//   - vB recognizes that "foo" refers to qux
//
// Scenario 3: migrated error passing through
// - v2 renames foo -> bar
//   v2 calls: RegisterTypeMigration("foo", &bar{})
// - v2.a, v2.b and v1 are connected: v2.a -> v1 -> v2.b
// - v2.a sends an error to v2.b via v1:
//   - v2.a encodes using foo's key, v1 receives as foo
//   - v1 encodes using foo's key
//   - v2.b receive's foo's key, knows about migration, decodes as bar
//
// Scenario 4: migrated error passing through node that does not know
// about it whatsoever (the key is preserved).
// - v2 renames foo -> bar
//   v2 calls: RegisterTypeMigration("foo", &bar{})
// - v2.a, v2.b and v0 are connected: v2.a -> v0 -> v2.b
//   (v0 does not know about error foo at all)
// - v2.a sends an error to v2.b via v0:
//   - v2.a encodes using foo's key, v0 receives as "unknown foo"
//   - v0 passes through unchanged
//   - v2.b receive's foo's key, knows about migration, decodes as bar
//
// Scenario 5: comparison between migrated and non-migrated errors
// on 3rd party node.
// - v2 renames foo -> bar
// - v2 sends error bar to v0
// - v1 sends an equivalent error with type foo to v0
// - v0 (that doesn't know about the type) compares the two errors.
// Here we're expecting v0 to properly ascertain the errors are equivalent.

// RegisterTypeMigration tells the library that the type of the error
// given as 3rd argument was previously known with type
// previousTypeName, located at previousPkgPath.
//
// The value of previousTypeName must be the result of calling
// reflect.TypeOf(err).String() on the original error object.
// This is usually composed as follows:
//     [*]<shortpackage>.<errortype>
//
// For example, Go's standard error type has name "*errors.errorString".
// The asterisk indicates that `errorString` implements the `error`
// interface via pointer receiver.
//
// Meanwhile, the singleton error type context.DeadlineExceeded
// has name "context.deadlineExceededError", without asterisk
// because the type implements `error` by value.
//
// Remember that the short package name inside the error type name and
// the last component of the package path can be different. This is
// why they must be specified separately.
func RegisterTypeMigration(previousPkgPath, previousTypeName string, newType error) {
	prevKey := TypeKey(makeTypeKey(previousPkgPath, previousTypeName))
	newKey := TypeKey(getFullTypeName(newType))

	// Register the backward migration: make the encode function
	// aware of the old name.
	if f, ok := backwardRegistry[newKey]; ok {
		panic(fmt.Errorf("migration to type %q already registered (from %q)", newKey, f))
	}
	backwardRegistry[newKey] = prevKey
	// If any other key was registered as a migration from newKey,
	// we'll forward those as well.
	// This changes X -> newKey to X -> prevKey for every X.
	for new, prev := range backwardRegistry {
		if prev == newKey {
			backwardRegistry[new] = prevKey
		}
	}
}

// registry used when encoding an error, so that the receiver observes
// the original key. This maps new keys to old keys.
var backwardRegistry = map[TypeKey]TypeKey{}

// TestingWithEmptyMigrationRegistry is intended for use by tests.
func TestingWithEmptyMigrationRegistry() (restore func()) {
	save := backwardRegistry
	backwardRegistry = map[TypeKey]TypeKey{}
	return func() { backwardRegistry = save }
}
