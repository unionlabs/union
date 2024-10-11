// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build cgo
// +build cgo

package sstable

// useStandardZstdLib indicates whether the zstd implementation is a port of the
// official one in the facebook/zstd repository.
//
// This constant is only used in tests. Some tests rely on reproducibility of
// SST files, but a custom implementation of zstd will produce different
// compression result. So those tests have to be disabled in such cases.
//
// We cannot always use the official facebook/zstd implementation since it
// relies on CGo.
const useStandardZstdLib = true
