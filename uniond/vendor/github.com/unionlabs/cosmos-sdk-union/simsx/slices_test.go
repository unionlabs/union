package simsx

import (
	"strconv"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestCollect(t *testing.T) {
	src := []int{1, 2, 3}
	got := Collect(src, func(a int) int { return a * 2 })
	assert.Equal(t, []int{2, 4, 6}, got)
	gotStrings := Collect(src, strconv.Itoa)
	assert.Equal(t, []string{"1", "2", "3"}, gotStrings)
}

func TestFirst(t *testing.T) {
	src := []string{"a", "b"}
	assert.Equal(t, &src[1], First(src, func(a string) bool { return a == "b" }))
	assert.Nil(t, First(src, func(a string) bool { return false }))
}

func TestOneOf(t *testing.T) {
	src := []string{"a", "b"}
	got := OneOf(randMock{next: 1}, src)
	assert.Equal(t, "b", got)
	// test with other type
	src2 := []int{1, 2, 3}
	got2 := OneOf(randMock{next: 2}, src2)
	assert.Equal(t, 3, got2)
}

type randMock struct {
	next int
}

func (x randMock) Intn(n int) int {
	return x.next
}
