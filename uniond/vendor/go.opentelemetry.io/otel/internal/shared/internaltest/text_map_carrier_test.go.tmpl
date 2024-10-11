// Code created by gotmpl. DO NOT MODIFY.
// source: internal/shared/internaltest/text_map_carrier_test.go.tmpl

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package internaltest

import (
	"reflect"
	"testing"
)

var key, value = "test", "true"

func TestTextMapCarrierKeys(t *testing.T) {
	tmc := NewTextMapCarrier(map[string]string{key: value})
	expected, actual := []string{key}, tmc.Keys()
	if !reflect.DeepEqual(actual, expected) {
		t.Errorf("expected tmc.Keys() to be %v but it was %v", expected, actual)
	}
}

func TestTextMapCarrierGet(t *testing.T) {
	tmc := NewTextMapCarrier(map[string]string{key: value})
	tmc.GotN(t, 0)
	if got := tmc.Get("empty"); got != "" {
		t.Errorf("TextMapCarrier.Get returned %q for an empty key", got)
	}
	tmc.GotKey(t, "empty")
	tmc.GotN(t, 1)
	if got := tmc.Get(key); got != value {
		t.Errorf("TextMapCarrier.Get(%q) returned %q, want %q", key, got, value)
	}
	tmc.GotKey(t, key)
	tmc.GotN(t, 2)
}

func TestTextMapCarrierSet(t *testing.T) {
	tmc := NewTextMapCarrier(nil)
	tmc.SetN(t, 0)
	tmc.Set(key, value)
	if got, ok := tmc.data[key]; !ok {
		t.Errorf("TextMapCarrier.Set(%q,%q) failed to store pair", key, value)
	} else if got != value {
		t.Errorf("TextMapCarrier.Set(%q,%q) stored (%q,%q), not (%q,%q)", key, value, key, got, key, value)
	}
	tmc.SetKeyValue(t, key, value)
	tmc.SetN(t, 1)
}

func TestTextMapCarrierReset(t *testing.T) {
	tmc := NewTextMapCarrier(map[string]string{key: value})
	tmc.GotN(t, 0)
	tmc.SetN(t, 0)
	tmc.Reset(nil)
	tmc.GotN(t, 0)
	tmc.SetN(t, 0)
	if got := tmc.Get(key); got != "" {
		t.Error("TextMapCarrier.Reset() failed to clear initial data")
	}
	tmc.GotN(t, 1)
	tmc.GotKey(t, key)
	tmc.Set(key, value)
	tmc.SetKeyValue(t, key, value)
	tmc.SetN(t, 1)
	tmc.Reset(nil)
	tmc.GotN(t, 0)
	tmc.SetN(t, 0)
	if got := tmc.Get(key); got != "" {
		t.Error("TextMapCarrier.Reset() failed to clear data")
	}
}
