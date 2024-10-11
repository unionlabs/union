// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package tool

import (
	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/bloom"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage/remote"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/spf13/cobra"
)

// Comparer exports the base.Comparer type.
type Comparer = base.Comparer

// FilterPolicy exports the base.FilterPolicy type.
type FilterPolicy = base.FilterPolicy

// Merger exports the base.Merger type.
type Merger = base.Merger

// T is the container for all of the introspection tools.
type T struct {
	Commands        []*cobra.Command
	db              *dbT
	find            *findT
	lsm             *lsmT
	manifest        *manifestT
	remotecat       *remoteCatalogT
	sstable         *sstableT
	wal             *walT
	opts            pebble.Options
	comparers       sstable.Comparers
	mergers         sstable.Mergers
	defaultComparer string
	openErrEnhancer func(error) error
}

// A Option configures the Pebble introspection tool.
type Option func(*T)

// Comparers may be passed to New to register comparers for use by
// the introspesction tools.
func Comparers(cmps ...*Comparer) Option {
	return func(t *T) {
		for _, c := range cmps {
			t.comparers[c.Name] = c
		}
	}
}

// DefaultComparer registers a comparer for use by the introspection tools and
// sets it as the default.
func DefaultComparer(c *Comparer) Option {
	return func(t *T) {
		t.comparers[c.Name] = c
		t.defaultComparer = c.Name
	}
}

// Mergers may be passed to New to register mergers for use by the
// introspection tools.
func Mergers(mergers ...*Merger) Option {
	return func(t *T) {
		for _, m := range mergers {
			t.mergers[m.Name] = m
		}
	}
}

// Filters may be passed to New to register filter policies for use by the
// introspection tools.
func Filters(filters ...FilterPolicy) Option {
	return func(t *T) {
		for _, f := range filters {
			t.opts.Filters[f.Name()] = f
		}
	}
}

// FS sets the filesystem implementation to use by the introspection tools.
func FS(fs vfs.FS) Option {
	return func(t *T) {
		t.opts.FS = fs
	}
}

// OpenErrEnhancer sets a function that enhances an error encountered when the
// tool opens a database; used to provide the user additional context, for
// example that a corruption error might be caused by encryption at rest not
// being configured properly.
func OpenErrEnhancer(fn func(error) error) Option {
	return func(t *T) {
		t.openErrEnhancer = fn
	}
}

// New creates a new introspection tool.
func New(opts ...Option) *T {
	t := &T{
		opts: pebble.Options{
			Filters:  make(map[string]FilterPolicy),
			FS:       vfs.Default,
			ReadOnly: true,
		},
		comparers:       make(sstable.Comparers),
		mergers:         make(sstable.Mergers),
		defaultComparer: base.DefaultComparer.Name,
	}

	opts = append(opts,
		Comparers(base.DefaultComparer),
		Filters(bloom.FilterPolicy(10)),
		Mergers(base.DefaultMerger))

	for _, opt := range opts {
		opt(t)
	}

	t.db = newDB(&t.opts, t.comparers, t.mergers, t.openErrEnhancer)
	t.find = newFind(&t.opts, t.comparers, t.defaultComparer, t.mergers)
	t.lsm = newLSM(&t.opts, t.comparers)
	t.manifest = newManifest(&t.opts, t.comparers)
	t.remotecat = newRemoteCatalog(&t.opts)
	t.sstable = newSSTable(&t.opts, t.comparers, t.mergers)
	t.wal = newWAL(&t.opts, t.comparers, t.defaultComparer)
	t.Commands = []*cobra.Command{
		t.db.Root,
		t.find.Root,
		t.lsm.Root,
		t.manifest.Root,
		t.remotecat.Root,
		t.sstable.Root,
		t.wal.Root,
	}
	return t
}

// ConfigureSharedStorage updates the shared storage options.
func (t *T) ConfigureSharedStorage(
	s remote.StorageFactory,
	createOnShared remote.CreateOnSharedStrategy,
	createOnSharedLocator remote.Locator,
) {
	t.opts.Experimental.RemoteStorage = s
	t.opts.Experimental.CreateOnShared = createOnShared
	t.opts.Experimental.CreateOnSharedLocator = createOnSharedLocator
}
