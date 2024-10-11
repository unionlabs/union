package kv_test

import (
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/cosmos/cosmos-sdk/types/kv"
)

func TestAssertKeyAtLeastLength(t *testing.T) {
	cases := []struct {
		name        string
		key         []byte
		length      int
		expectPanic bool
	}{
		{
			name:        "Store key length is less than the given length",
			key:         []byte("hello"),
			length:      10,
			expectPanic: true,
		},
		{
			name:        "Store key length is equal to the given length",
			key:         []byte("store-key"),
			length:      9,
			expectPanic: false,
		},
		{
			name:        "Store key length is greater than the given length",
			key:         []byte("unique"),
			length:      3,
			expectPanic: false,
		},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			if tc.expectPanic {
				assert.Panics(t, func() {
					kv.AssertKeyAtLeastLength(tc.key, tc.length)
				})
				return
			}
			kv.AssertKeyAtLeastLength(tc.key, tc.length)
		})
	}
}

func TestAssertKeyLength(t *testing.T) {
	cases := []struct {
		name        string
		key         []byte
		length      int
		expectPanic bool
	}{
		{
			name:        "Store key length is less than the given length",
			key:         []byte("hello"),
			length:      10,
			expectPanic: true,
		},
		{
			name:        "Store key length is equal to the given length",
			key:         []byte("store-key"),
			length:      9,
			expectPanic: false,
		},
		{
			name:        "Store key length is greater than the given length",
			key:         []byte("unique"),
			length:      3,
			expectPanic: true,
		},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			if tc.expectPanic {
				assert.Panics(t, func() {
					kv.AssertKeyLength(tc.key, tc.length)
				})
				return
			}
			kv.AssertKeyLength(tc.key, tc.length)
		})
	}
}
