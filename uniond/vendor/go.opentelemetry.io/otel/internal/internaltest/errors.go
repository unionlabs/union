// Code created by gotmpl. DO NOT MODIFY.
// source: internal/shared/internaltest/errors.go.tmpl

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package internaltest // import "go.opentelemetry.io/otel/internal/internaltest"

type TestError string

var _ error = TestError("")

func NewTestError(s string) error {
	return TestError(s)
}

func (e TestError) Error() string {
	return string(e)
}
