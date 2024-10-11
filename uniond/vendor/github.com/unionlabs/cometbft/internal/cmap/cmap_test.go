package cmap

import (
	"fmt"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestIterateKeysWithValues(t *testing.T) {
	cmap := NewCMap()

	for i := 1; i <= 10; i++ {
		cmap.Set(fmt.Sprintf("key%d", i), fmt.Sprintf("value%d", i))
	}

	// Testing size
	assert.Equal(t, 10, cmap.Size())
	assert.Len(t, cmap.Keys(), 10)
	assert.Len(t, cmap.Values(), 10)

	// Iterating Keys, checking for matching Value
	for _, key := range cmap.Keys() {
		val := strings.ReplaceAll(key, "key", "value")
		assert.Equal(t, val, cmap.Get(key))
	}

	// Test if all keys are within []Keys()
	keys := cmap.Keys()
	for i := 1; i <= 10; i++ {
		assert.Contains(t, keys, fmt.Sprintf("key%d", i), "cmap.Keys() should contain key")
	}

	// Delete 1 Key
	cmap.Delete("key1")

	assert.NotEqual(
		t,
		len(keys),
		len(cmap.Keys()),
		"[]keys and []Keys() should not be equal, they are copies, one item was removed",
	)
}

func TestContains(t *testing.T) {
	cmap := NewCMap()

	cmap.Set("key1", "value1")

	// Test for known values
	assert.True(t, cmap.Has("key1"))
	assert.Equal(t, "value1", cmap.Get("key1"))

	// Test for unknown values
	assert.False(t, cmap.Has("key2"))
	assert.Nil(t, cmap.Get("key2"))
}

func BenchmarkCMapHas(b *testing.B) {
	m := NewCMap()
	for i := 0; i < 1000; i++ {
		m.Set(string(rune(i)), i)
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		m.Has(string(rune(i)))
	}
}
