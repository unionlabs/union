// Package legacy contains a global amino Cdc which is deprecated but
// still used in several places within the SDK. This package is intended
// to be removed at some point in the future when the global Cdc is removed.
// It also contains a util function RegisterAminoMsg that checks a msg name length
// before registering the concrete msg type with amino.
package legacy
