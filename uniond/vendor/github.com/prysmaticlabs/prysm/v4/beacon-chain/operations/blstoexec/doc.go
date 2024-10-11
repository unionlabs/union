// Package blstoexecchanges defines an in-memory pool of received BLS-to-ETH1 change objects.
// Internally it uses a combination of doubly-linked list and map to handle pool objects.
// The linked list approach has an advantage over a slice in terms of memory usage.
// For our scenario, we will mostly append objects to the end and remove objects from arbitrary positions,
// and during block proposal we will remove objects from the front.
// The map is used for looking up the list node that needs to be removed when we mark an object as included in a block.
package blstoexec
