/*
Copyright 2019 The logr Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

// Package testing provides support for using logr in tests.
// Deprecated.  See github.com/go-logr/logr/testr instead.
package testing

import "github.com/go-logr/logr/testr"

// NewTestLogger returns a logr.Logger that prints through a testing.T object.
// Deprecated.  See github.com/go-logr/logr/testr.New instead.
var NewTestLogger = testr.New

// Options carries parameters which influence the way logs are generated.
// Deprecated.  See github.com/go-logr/logr/testr.Options instead.
type Options = testr.Options

// NewTestLoggerWithOptions returns a logr.Logger that prints through a testing.T object.
// Deprecated.  See github.com/go-logr/logr/testr.NewWithOptions instead.
var NewTestLoggerWithOptions = testr.NewWithOptions

// Underlier exposes access to the underlying testing.T instance.
// Deprecated.  See github.com/go-logr/logr/testr.Underlier instead.
type Underlier = testr.Underlier
