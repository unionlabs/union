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

package errbase_test

import (
	"context"
	"reflect"
	"testing"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/markers"
	"github.com/gogo/protobuf/proto"
)

// Scenario 1: simple migration, forward direction
// - v2 renames foo -> bar
// - v2 and v1 are connected
// - v1 sends an error to v2.
func TestSimpleMigrationForward(t *testing.T) {
	defer errbase.TestingWithEmptyMigrationRegistry()()

	// == Scenario on v1 ==
	origErr := fooErr{}
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(origErr),
		func(_ context.Context, _ string, _ []string, _ proto.Message) error { return fooErr{} })

	// Send the error to v2.
	enc := errbase.EncodeError(context.Background(), origErr)
	// Clean up, so that type foo becomes unknown.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(origErr), nil)

	// == Scenario on v2 ==
	// Register the fact that foo was migrated to bar.
	errbase.RegisterTypeMigration(myPkgPath, "errbase_test.fooErr", barErr{})
	// Register the bar decoder.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(barErr{}),
		func(_ context.Context, _ string, _ []string, _ proto.Message) error { return barErr{} })
	// Receive the error from v1.
	dec := errbase.DecodeError(context.Background(), enc)
	// Clean up, so that type bar becomes unknown for further tests.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(barErr{}), nil)

	// Main test: check that v2 recognized the foo error from
	// v1 and instantiated it as bar.
	if _, ok := dec.(barErr); !ok {
		t.Errorf("migration failed; expected type barErr, got %T", dec)
	}
}

// Scenario 1: simple migration, backward direction
// - v2 renames foo -> bar
// - v2 and v1 are connected
// - v2 sends an error to v1.
func TestSimpleMigrationBackward(t *testing.T) {
	defer errbase.TestingWithEmptyMigrationRegistry()()

	// == Scenario on v2 ==
	origErr := barErr{}
	// Register the fact that foo was migrated to bar.
	errbase.RegisterTypeMigration(myPkgPath, "errbase_test.fooErr", origErr)
	// Send the error to v1.
	enc := errbase.EncodeError(context.Background(), origErr)
	// Clean up the decoder, so that type becomes unknown for further tests.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(origErr), nil)
	// Erase the migration we have set up above, so that the test
	// underneath does not know about it.
	defer errbase.TestingWithEmptyMigrationRegistry()()

	// == Scenario on v1 ==
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(fooErr{}),
		func(_ context.Context, _ string, _ []string, _ proto.Message) error { return fooErr{} })

	// Receive the error from v2.
	dec := errbase.DecodeError(context.Background(), enc)
	// Clean up, so that type foo becomes unknown.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(fooErr{}), nil)

	// Main test: check that v1 recognized the foo error from
	// v2 and instantiated it as foo.
	if _, ok := dec.(fooErr); !ok {
		t.Errorf("migration failed; expected type fooErr, got %T", dec)
	}
}

// This is the same as above, using a pointer receiver for the error type.
func TestSimpleMigrationForwardPtr(t *testing.T) {
	defer errbase.TestingWithEmptyMigrationRegistry()()

	// == Scenario on v1 ==
	origErr := (*fooErrP)(nil)
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(origErr),
		func(_ context.Context, _ string, _ []string, _ proto.Message) error { return (*fooErrP)(nil) })

	// Send the error to v2.
	enc := errbase.EncodeError(context.Background(), origErr)
	// Clean up, so that type foo becomes unknown.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(origErr), nil)

	// == Scenario on v2 ==
	// Register the fact that foo was migrated to bar.
	errbase.RegisterTypeMigration(myPkgPath, "*errbase_test.fooErrP", (*barErrP)(nil))
	// Register the bar decoder.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey((*barErrP)(nil)),
		func(_ context.Context, _ string, _ []string, _ proto.Message) error { return (*barErrP)(nil) })
	// Receive the error from v1.
	dec := errbase.DecodeError(context.Background(), enc)
	// Clean up, so that type bar becomes unknown for further tests.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey((*barErrP)(nil)), nil)

	// Main test: check that v2 recognized the foo error from
	// v1 and instantiated it as bar.
	if _, ok := dec.(*barErrP); !ok {
		t.Errorf("migration failed; expected type *barErrP, got %T", dec)
	}
}

// Scenario 2: simultaneous migration
// - vA renames foo -> bar
//   vA calls RegisterTypeMigration("foo", (*bar)(nil))
// - vB renames foo -> qux
//   vB calls RegisterTypeMigration("foo", (*qux)(nil))
// - vA and vB are connected
// - vA sends an error to vB:
//   - vA translates the error key upon send from bar to foo's key
//   - vB recognizes that "foo" refers to qux
func TestSimultaneousMigration(t *testing.T) {
	// == Scenario on vA ==
	defer errbase.TestingWithEmptyMigrationRegistry()()
	origErr := barErr{}
	// Register the fact that foo was migrated to bar.
	errbase.RegisterTypeMigration(myPkgPath, "errbase_test.fooErr", origErr)
	// Send the error to vB.
	enc := errbase.EncodeError(context.Background(), origErr)
	// Clean up the decoder, so that type becomes unknown for further tests.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(origErr), nil)
	// Erase the migration we have set up above, so that the test
	// underneath does not know about it.
	defer errbase.TestingWithEmptyMigrationRegistry()()

	// == Scenario on v2 ==
	defer errbase.TestingWithEmptyMigrationRegistry()()
	// Register the fact that foo was migrated to qux.
	errbase.RegisterTypeMigration(myPkgPath, "errbase_test.fooErr", quxErr{})
	// Register the qux decoder.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(quxErr{}),
		func(_ context.Context, _ string, _ []string, _ proto.Message) error { return quxErr{} })
	// Receive the error from vA.
	dec := errbase.DecodeError(context.Background(), enc)
	// Clean up, so that type qux becomes unknown for further tests.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(quxErr{}), nil)

	// Main test: check that vA normalized the bar error to foo,
	// and v2 instantiated it as qux.
	if _, ok := dec.(quxErr); !ok {
		t.Errorf("migration failed; expected type quxErr, got %T", dec)
	}
}

// Scenario 3: migrated error passing through
// - v2 renames foo -> bar
// - v2.a, v2.b and v1 are connected: v2.a -> v1 -> v2.b
// - v2.a sends an error to v2.b via v1
func TestMigratedErrorPassingThrough(t *testing.T) {
	defer errbase.TestingWithEmptyMigrationRegistry()()

	// == Scenario on v2.a ==
	origErr := barErr{}
	// Register the fact that foo was migrated to bar.
	errbase.RegisterTypeMigration(myPkgPath, "errbase_test.fooErr", origErr)
	// Send the error to v1.
	enc := errbase.EncodeError(context.Background(), origErr)
	// Clean up the decoder, so that type becomes unknown for further tests.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(origErr), nil)
	// Erase the migration we have set up above, so that the test
	// underneath does not know about it.
	defer errbase.TestingWithEmptyMigrationRegistry()()

	// == Scenario on v1 ==
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(fooErr{}),
		func(_ context.Context, _ string, _ []string, _ proto.Message) error { return fooErr{} })
	// Receive the error from v2.b.
	dec := errbase.DecodeError(context.Background(), enc)
	// Send the error to v2.b.
	enc2 := errbase.EncodeError(context.Background(), dec)
	// Clean up, so that type foo becomes unknown.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(fooErr{}), nil)

	// == Scenario on v2.b ==
	// Register the fact that foo was migrated to bar.
	errbase.RegisterTypeMigration(myPkgPath, "errbase_test.fooErr", barErr{})
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(barErr{}),
		func(_ context.Context, _ string, _ []string, _ proto.Message) error { return barErr{} })
	// Receive the error from v1.
	dec2 := errbase.DecodeError(context.Background(), enc2)
	// Clean up the decoder, so that type becomes unknown for further tests.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(barErr{}), nil)
	// Erase the migration we have set up above, so that the test
	// underneath does not know about it.
	defer errbase.TestingWithEmptyMigrationRegistry()()

	// Main test: check that v2.b recognized the foo error
	// that was passed from v2.a.
	if _, ok := dec2.(barErr); !ok {
		t.Errorf("migration failed; expected type barErr, got %T", dec2)
	}
}

// Scenario 4: migrated error passing through node that
// does not know about the error type whatsoever.
// - v2 renames foo -> bar
// - v2.a, v2.b and v0 are connected: v2.a -> v0 -> v2.b
//   (v0 does not know about error foo at all)
// - v2.a sends an error to v2.b via v0:
func TestMigratedErrorPassingThroughAsUnknown(t *testing.T) {
	defer errbase.TestingWithEmptyMigrationRegistry()()

	// == Scenario on v2.a ==
	origErr := barErr{}
	// Register the fact that foo was migrated to bar.
	errbase.RegisterTypeMigration(myPkgPath, "errbase_test.fooErr", origErr)
	// Send the error to v1.
	enc := errbase.EncodeError(context.Background(), origErr)
	// Clean up the decoder, so that type becomes unknown for further tests.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(origErr), nil)
	// Erase the migration we have set up above, so that the test
	// underneath does not know about it.
	defer errbase.TestingWithEmptyMigrationRegistry()()

	// == Scenario on v1 ==
	// Receive the error from v2.b. Will decode as opaqueLeaf{}.
	dec := errbase.DecodeError(context.Background(), enc)
	// Send the error to v2.b.
	enc2 := errbase.EncodeError(context.Background(), dec)

	// == Scenario on v2.b ==
	// Register the fact that foo was migrated to bar.
	errbase.RegisterTypeMigration(myPkgPath, "errbase_test.fooErr", barErr{})
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(barErr{}),
		func(_ context.Context, _ string, _ []string, _ proto.Message) error { return barErr{} })
	// Receive the error from v1.
	dec2 := errbase.DecodeError(context.Background(), enc2)
	// Clean up the decoder, so that type becomes unknown for further tests.
	errbase.RegisterLeafDecoder(errbase.GetTypeKey(barErr{}), nil)
	// Erase the migration we have set up above, so that the test
	// underneath does not know about it.
	defer errbase.TestingWithEmptyMigrationRegistry()()

	// Main test: check that v2.b recognized the foo error
	// that was passed from v2.a.
	if _, ok := dec2.(barErr); !ok {
		t.Errorf("migration failed; expected type barErr, got %T", dec2)
	}
}

// Scenario 5: comparison between migrated and non-migrated errors
// on 3rd party node that doesn't know about the type.
// - v2 renames foo -> bar
// - v2 sends error bar to v0
// - v1 sends error foo to v0
// - v0 (that doesn't know about the type) compares the two errors.
func TestUnknownErrorComparisonAfterHeterogeneousMigration(t *testing.T) {
	defer errbase.TestingWithEmptyMigrationRegistry()()

	// == Scenario on v1 ==
	// Send the error to v0.
	enc1 := errbase.EncodeError(context.Background(), fooErr{})

	// == Scenario on v2 ==
	// Register the fact that foo was migrated to bar.
	errbase.RegisterTypeMigration(myPkgPath, "errbase_test.fooErr", barErr{})
	// Send the error to v0.
	enc2 := errbase.EncodeError(context.Background(), barErr{})
	// Clear the migration so that the type is invisible to v0 below.
	defer errbase.TestingWithEmptyMigrationRegistry()()

	// == Scenario on v0 ==
	// Receive the two errors.
	dec1 := errbase.DecodeError(context.Background(), enc1)
	dec2 := errbase.DecodeError(context.Background(), enc2)

	// Main test: check that v0 recognizes the two errors as equivalent.
	if !markers.Is(dec1, dec2) {
		t.Error("equivalence after migration failed")
	}
	if !markers.Is(dec2, dec1) {
		t.Error("equivalence after migration failed")
	}
}

type fooErr struct{}

func (fooErr) Error() string { return "" }

type barErr struct{}

func (barErr) Error() string { return "" }

type quxErr struct{}

func (quxErr) Error() string { return "" }

type fooErrP struct{}

func (*fooErrP) Error() string { return "" }

type barErrP struct{}

func (*barErrP) Error() string { return "" }

var myPkgPath = func() string {
	t := fooErr{}
	return reflect.TypeOf(t).PkgPath()
}()
