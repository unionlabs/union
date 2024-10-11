// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package otelgrpc

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"google.golang.org/grpc/metadata"
)

func TestMetadataSupplier(t *testing.T) {
	md := metadata.New(map[string]string{
		"k1": "v1",
	})
	ms := &metadataSupplier{&md}

	v1 := ms.Get("k1")
	assert.Equal(t, v1, "v1")

	ms.Set("k2", "v2")

	v1 = ms.Get("k1")
	v2 := ms.Get("k2")
	assert.Equal(t, v1, "v1")
	assert.Equal(t, v2, "v2")
}
