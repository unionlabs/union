// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package otel_test

import (
	"log"
	"os"

	"github.com/go-logr/stdr"

	"go.opentelemetry.io/otel"
)

func ExampleSetLogger() {
	logger := stdr.New(log.New(os.Stdout, "", log.LstdFlags|log.Lshortfile))
	otel.SetLogger(logger)
}
