// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

// Package disalloweq provides a method for disallowing struct comparisons
// with the `==` operator.
package disalloweq

// DisallowEqual can be used to cause the compiler to reject attempts to
// compare structs with the `==` operator.
//
// The better solution would be for Go to embrace circa 1960s technology
// and support operator overloading a la ALGOL 68.
//
// See: https://twitter.com/bradfitz/status/860145039573385216
type DisallowEqual [0]func()
