// Package any is a helper library to be used as a replacement for some of the
// official "google.golang.org/protobuf/types/known/anypb"'s functions which
// prepend the "type.googleapis.com" prefix in type URLs.
//
// In the Cosmos SDK, an Any's TypeURL starts with a `/` character, e.g.
// "/cosmos.bank.v1beta1.MsgSend". However, if we pack this MsgSend into
// an anypb.Any using the offical anypb package's helper functions, we get a
// TypeURL which is "type.googleapis.com/cosmos.bank.v1beta1.MsgSend".
//
// Therefore, the following 3 functions/methods cannot be used within the
// Cosmos context:
// - anypb.New
// - anypb.MarshalFrom
// - anypb.Any#MarshalFrom
// as all of them append the unwanted prefix.
//
// This package exposes the `New` and `MarshalFrom` helper functions, which do
// not prepend any prefix to type URLs.
package anyutil
