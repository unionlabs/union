// Package keyspan provides facilities for sorting, fragmenting and
// iterating over spans of user keys.
//
// A Span represents a range of user key space with an inclusive start
// key and exclusive end key. A span may hold any number of Keys which are
// applied over the entirety of the span's keyspace.
//
// Spans are used within Pebble as an in-memory representation of range
// deletion tombstones, and range key sets, unsets and deletes. Spans
// are fragmented at overlapping key boundaries by the Fragmenter type.
// This package's various iteration facilities require these
// non-overlapping fragmented spans.
package keyspan
