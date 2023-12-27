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

package errors

import "github.com/cockroachdb/errors/domains"

// Domain is the type of a domain annotation.
type Domain = domains.Domain

// NoDomain is the domain of errors that don't originate
// from a barrier.
const NoDomain Domain = domains.NoDomain

// NamedDomain returns an error domain identified by the given string.
func NamedDomain(domainName string) Domain { return domains.NamedDomain(domainName) }

// PackageDomain returns an error domain that represents the
// package of its caller.
func PackageDomain() Domain { return domains.PackageDomainAtDepth(1) }

// PackageDomainAtDepth returns an error domain that describes the
// package at the given call depth.
func PackageDomainAtDepth(depth int) Domain { return domains.PackageDomainAtDepth(depth) }

// WithDomain wraps an error so that it appears to come from the given domain.
//
// Domain is shown:
// - via `errors.GetSafeDetails()`.
// - when formatting with `%+v`.
// - in Sentry reports.
func WithDomain(err error, domain Domain) error { return domains.WithDomain(err, domain) }

// NotInDomain returns true if and only if the error's
// domain is not one of the specified domains.
func NotInDomain(err error, doms ...Domain) bool { return domains.NotInDomain(err, doms...) }

// EnsureNotInDomain checks whether the error is in the given domain(s).
// If it is, the given constructor if provided is called to construct
// an alternate error. If no error constructor is provided,
// a new barrier is constructed automatically using the first
// provided domain as new domain. The original error message
// is preserved.
func EnsureNotInDomain(err error, constructor DomainOverrideFn, forbiddenDomains ...Domain) error {
	return domains.EnsureNotInDomain(err, constructor, forbiddenDomains...)
}

// DomainOverrideFn is the type of the callback function passed to EnsureNotInDomain().
type DomainOverrideFn = func(originalDomain Domain, err error) error

// HandledInDomain creates an error in the given domain and retains
// the details of the given original error as context for
// debugging. The original error is hidden and does not become a
// "cause" for the new error. The original's error _message_
// is preserved.
//
// See the documentation of `WithDomain()` and `errors.Handled()` for details.
func HandledInDomain(err error, domain Domain) error { return domains.HandledInDomain(err, domain) }

// HandledInDomainWithMessage is like HandledWithMessage but with a domain.
func HandledInDomainWithMessage(err error, domain Domain, msg string) error {
	return domains.HandledInDomainWithMessage(err, domain, msg)
}

// GetDomain extracts the domain of the given error, or NoDomain if
// the error's cause does not have a domain annotation.
func GetDomain(err error) Domain { return domains.GetDomain(err) }
