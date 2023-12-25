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

package domains

import (
	"errors"
	"fmt"
	"path/filepath"
	"runtime"

	"github.com/cockroachdb/errors/barriers"
	"github.com/cockroachdb/errors/errbase"
)

// Domain is the type of a domain annotation.
type Domain string

// NoDomain is the domain of errors that don't originate
// from a barrier.
const NoDomain Domain = "error domain: <none>"

// GetDomain extracts the domain of the given error, or NoDomain if
// the error's cause does not have a domain annotation.
func GetDomain(err error) Domain {
	for {
		if b, ok := err.(*withDomain); ok {
			return b.domain
		}
		// Recurse to the cause.
		if c := errbase.UnwrapOnce(err); c != nil {
			err = c
			continue
		}
		break
	}
	return NoDomain
}

// WithDomain wraps an error so that it appears to come from the given domain.
//
// Domain is shown:
// - via `errors.GetSafeDetails()`.
// - when formatting with `%+v`.
// - in Sentry reports.
func WithDomain(err error, domain Domain) error {
	if err == nil {
		return nil
	}
	return &withDomain{cause: err, domain: domain}
}

// New creates an error in the implicit domain (see PackageDomain() below)
// of its caller.
//
// Domain is shown:
// - via `errors.GetSafeDetails()`.
// - when formatting with `%+v`.
// - in Sentry reports.
func New(msg string) error {
	return WithDomain(errors.New(msg), PackageDomainAtDepth(1))
}

// Newf/Errorf with format and args can be implemented similarly.

// HandledInDomain creates an error in the given domain and retains
// the details of the given original error as context for
// debugging. The original error is hidden and does not become a
// "cause" for the new error. The original's error _message_
// is preserved.
//
// See the documentation of `WithDomain()` and `errors.Handled()` for details.
func HandledInDomain(err error, domain Domain) error {
	return WithDomain(barriers.Handled(err), domain)
}

// HandledInDomainWithMessage is like HandledWithMessage but with a domain.
func HandledInDomainWithMessage(err error, domain Domain, msg string) error {
	return WithDomain(barriers.HandledWithMessage(err, msg), domain)
}

// Handled creates a handled error in the implicit domain (see
// PackageDomain() below) of its caller.
//
// See the documentation of `barriers.Handled()` for details.
func Handled(err error) error {
	return HandledInDomain(err, PackageDomainAtDepth(1))
}

// Handledf with format and args can be implemented similarly.

// NotInDomain returns true if and only if the error's
// domain is not one of the specified domains.
func NotInDomain(err error, domains ...Domain) bool {
	return notInDomainInternal(GetDomain(err), domains...)
}

func notInDomainInternal(d Domain, domains ...Domain) bool {
	for _, given := range domains {
		if d == given {
			return false
		}
	}
	return true
}

// EnsureNotInDomain checks whether the error is in the given domain(s).
// If it is, the given constructor if provided is called to construct
// an alternate error. If no error constructor is provided,
// a new barrier is constructed automatically using the first
// provided domain as new domain. The original error message
// is preserved.
func EnsureNotInDomain(
	err error, constructor func(originalDomain Domain, err error) error, forbiddenDomains ...Domain,
) error {
	if err == nil {
		return nil
	}

	// Is the error already in the wanted domains?
	errDomain := GetDomain(err)
	if notInDomainInternal(errDomain, forbiddenDomains...) {
		// No: no-op.
		return err
	}
	return constructor(errDomain, err)
}

// PackageDomain returns an error domain that represents the
// package of its caller.
func PackageDomain() Domain {
	return PackageDomainAtDepth(1)
}

// PackageDomainAtDepth returns an error domain that describes the
// package at the given call depth.
func PackageDomainAtDepth(depth int) Domain {
	_, f, _, _ := runtime.Caller(1 + depth)
	return Domain("error domain: pkg " + filepath.Dir(f))
}

// NamedDomain returns an error domain identified by the given string.
func NamedDomain(domainName string) Domain {
	return Domain(fmt.Sprintf("error domain: %q", domainName))
}
