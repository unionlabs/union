package kv

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestIntInSlice(t *testing.T) {
	assert.True(t, intInSlice(1, []int{1, 2, 3}))
	assert.False(t, intInSlice(4, []int{1, 2, 3}))
	assert.True(t, intInSlice(0, []int{0}))
	assert.False(t, intInSlice(0, []int{}))
}
