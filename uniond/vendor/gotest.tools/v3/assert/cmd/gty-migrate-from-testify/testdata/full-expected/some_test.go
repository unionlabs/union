package foo

import (
	"fmt"
	"testing"

	"github.com/go-check/check"
	"gotest.tools/v3/assert"
	"gotest.tools/v3/assert/cmp"
)

type mystruct struct {
	a        int
	expected int
}

func TestFirstThing(t *testing.T) {
	rt := assert.TestingT(t)
	assert.Check(t, cmp.Equal("foo", "bar"))
	assert.Check(t, cmp.Equal(1, 2))
	assert.Check(t, false)
	assert.Check(t, !true)
	assert.NilError(rt, nil)

	assert.Check(t, cmp.DeepEqual(map[string]bool{"a": true}, nil))
	assert.Check(t, cmp.DeepEqual([]int{1}, nil))
	assert.Equal(rt, "a", "B")
}

func TestSecondThing(t *testing.T) {
	var foo mystruct
	assert.DeepEqual(t, foo, mystruct{})

	assert.DeepEqual(t, mystruct{}, mystruct{})

	assert.Check(t, nil, "foo %d", 3)
	assert.NilError(t, nil, "foo %d", 3)

	assert.Check(t, cmp.ErrorContains(fmt.Errorf("foo"), ""))

	assert.Assert(t, 77 != 0)
}

func TestOthers(t *testing.T) {
	assert.Check(t, cmp.Contains([]string{}, "foo"))
	assert.Assert(t, cmp.Len([]int{}, 3))
	assert.Check(t, cmp.Panics(func() { panic("foo") }))
	assert.Error(t, fmt.Errorf("bad days"), "good days")
	assert.Check(t, nil != nil)

	t.Error("why")
	t.Fatal("why not")
	assert.Assert(t, len([]bool{}) != 0)

	// Unsupported asseert
	assert.NotContains(t, []bool{}, true)
}

func TestAssertNew(t *testing.T) {

	assert.Check(t, cmp.Equal("a", "b"))
}

type unit struct {
	c *testing.T
}

func thing(t *testing.T) unit {
	return unit{c: t}
}

func TestStoredTestingT(t *testing.T) {
	u := thing(t)
	assert.Check(u.c, cmp.Equal("A", "b"))

	u = unit{c: t}
	assert.Check(u.c, cmp.Equal("A", "b"))
}

func TestNotNamedT(c *testing.T) {
	assert.Check(c, cmp.Equal("A", "b"))
}

func TestEqualsWithComplexTypes(t *testing.T) {
	expected := []int{1, 2, 3}
	assert.Check(t, cmp.DeepEqual(expected, nil))

	expectedM := map[int]bool{}
	assert.Check(t, cmp.DeepEqual(expectedM, nil))

	expectedI := 123
	assert.Check(t, cmp.Equal(expectedI, 0))

	assert.Check(t, cmp.Equal(doInt(), 3))
	// TODO: struct field
}

func doInt() int {
	return 1
}

func TestEqualWithPrimitiveTypes(t *testing.T) {
	s := "foo"
	ptrString := &s
	assert.Check(t, cmp.Equal(*ptrString, "foo"))

	assert.Check(t, cmp.Equal(doInt(), doInt()))

	x := doInt()
	y := doInt()
	assert.Check(t, cmp.Equal(x, y))

	tc := mystruct{a: 3, expected: 5}
	assert.Check(t, cmp.Equal(tc.a, tc.expected))
}

func TestTableTest(t *testing.T) {
	var testcases = []struct {
		opts         []string
		actual       string
		expected     string
		expectedOpts []string
	}{
		{
			opts:     []string{"a", "b"},
			actual:   "foo",
			expected: "else",
		},
	}

	for _, testcase := range testcases {
		assert.Check(t, cmp.Equal(testcase.actual, testcase.expected))
		assert.Check(t, cmp.DeepEqual(testcase.opts, testcase.expectedOpts))
	}
}

func TestWithChecker(c *check.C) {
	var err error
	assert.Check(c, err)
}

func HelperWithAssertTestingT(t assert.TestingT) {
	var err error
	assert.Check(t, err, "with assert.TestingT")
}

func BenchmarkSomething(b *testing.B) {
	var err error
	assert.Check(b, err)
}
