// Package gogoreflection implements gRPC reflection for gogoproto consumers
// the normal reflection library does not work as it points to a different
// singleton registry. The API and codebase is taken from the official gRPC
// reflection repository.
package gogoreflection
