// Copyright 2015 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

#ifndef __GO_REF_HDR__
#define __GO_REF_HDR__

#include <Foundation/Foundation.h>

// GoSeqRef is an object tagged with an integer for passing back and
// forth across the language boundary. A GoSeqRef may represent either
// an instance of a Go object, or an Objective-C object passed to Go.
// The explicit allocation of a GoSeqRef is used to pin a Go object
// when it is passed to Objective-C. The Go seq package maintains a
// reference to the Go object in a map keyed by the refnum along with
// a reference count. When the reference count reaches zero, the Go
// seq package will clear the corresponding entry in the map.
@interface GoSeqRef : NSObject {
}
@property(readonly) int32_t refnum;
@property(strong) id obj; // NULL when representing a Go object.

// new GoSeqRef object to proxy a Go object. The refnum must be
// provided from Go side.
- (instancetype)initWithRefnum:(int32_t)refnum obj:(id)obj;

- (int32_t)incNum;

@end

@protocol goSeqRefInterface
-(GoSeqRef*) _ref;
@end

#endif
