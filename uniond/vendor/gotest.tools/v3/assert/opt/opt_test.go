package opt

import (
	"sort"
	"testing"
	"time"

	gocmp "github.com/google/go-cmp/cmp"
	"gotest.tools/v3/assert"
)

func TestDurationWithThreshold(t *testing.T) {
	var testcases = []struct {
		name            string
		x, y, threshold time.Duration
		expected        bool
	}{
		{
			name:      "delta is threshold",
			threshold: time.Second,
			x:         3 * time.Second,
			y:         2 * time.Second,
			expected:  true,
		},
		{
			name:      "delta is negative threshold",
			threshold: time.Second,
			x:         2 * time.Second,
			y:         3 * time.Second,
			expected:  true,
		},
		{
			name:      "delta within threshold",
			threshold: time.Second,
			x:         300 * time.Millisecond,
			y:         100 * time.Millisecond,
			expected:  true,
		},
		{
			name:      "delta within negative threshold",
			threshold: time.Second,
			x:         100 * time.Millisecond,
			y:         300 * time.Millisecond,
			expected:  true,
		},
		{
			name:      "delta outside threshold",
			threshold: time.Second,
			x:         5 * time.Second,
			y:         300 * time.Millisecond,
		},
		{
			name:      "delta outside negative threshold",
			threshold: time.Second,
			x:         300 * time.Millisecond,
			y:         5 * time.Second,
		},
		{
			name:      "x is 0",
			threshold: time.Second,
			y:         5 * time.Millisecond,
		},
		{
			name:      "y is 0",
			threshold: time.Second,
			x:         5 * time.Millisecond,
		},
	}
	for _, testcase := range testcases {
		t.Run(testcase.name, func(t *testing.T) {
			actual := cmpDuration(testcase.threshold)(testcase.x, testcase.y)
			assert.Equal(t, actual, testcase.expected)
		})
	}
}

func TestTimeWithThreshold(t *testing.T) {
	var now = time.Now()

	var testcases = []struct {
		name      string
		x, y      time.Time
		threshold time.Duration
		expected  bool
	}{
		{
			name:      "delta is threshold",
			threshold: time.Minute,
			x:         now,
			y:         now.Add(time.Minute),
			expected:  true,
		},
		{
			name:      "delta is negative threshold",
			threshold: time.Minute,
			x:         now,
			y:         now.Add(-time.Minute),
			expected:  true,
		},
		{
			name:      "delta within threshold",
			threshold: time.Hour,
			x:         now,
			y:         now.Add(time.Minute),
			expected:  true,
		},
		{
			name:      "delta within negative threshold",
			threshold: time.Hour,
			x:         now,
			y:         now.Add(-time.Minute),
			expected:  true,
		},
		{
			name:      "delta outside threshold",
			threshold: time.Second,
			x:         now,
			y:         now.Add(time.Minute),
		},
		{
			name:      "delta outside negative threshold",
			threshold: time.Second,
			x:         now,
			y:         now.Add(-time.Minute),
		},
		{
			name:      "x is 0",
			threshold: time.Second,
			y:         now,
		},
		{
			name:      "y is 0",
			threshold: time.Second,
			x:         now,
		},
	}
	for _, testcase := range testcases {
		t.Run(testcase.name, func(t *testing.T) {
			actual := cmpTime(testcase.threshold)(testcase.x, testcase.y)
			assert.Equal(t, actual, testcase.expected)
		})
	}
}

type node struct {
	Value    nodeValue
	Labels   map[string]node
	Children []node
	Ref      *node
}

type nodeValue struct {
	Value int
}

type pathRecorder struct {
	filter  func(p gocmp.Path) bool
	matches []string
}

func (p *pathRecorder) record(path gocmp.Path) bool {
	if p.filter(path) {
		p.matches = append(p.matches, path.GoString())
	}
	return false
}

func matchPaths(fixture interface{}, filter func(gocmp.Path) bool) []string {
	rec := &pathRecorder{filter: filter}
	gocmp.Equal(fixture, fixture, gocmp.FilterPath(rec.record, gocmp.Ignore()))
	sort.Strings(rec.matches)
	return rec.matches
}

func TestPathStringFromStruct(t *testing.T) {
	fixture := node{
		Ref: &node{
			Children: []node{
				{},
				{
					Labels: map[string]node{
						"first": {Value: nodeValue{Value: 3}},
					},
				},
			},
		},
	}

	spec := "Ref.Children.Labels.Value"
	matches := matchPaths(fixture, PathString(spec))
	expected := []string{
		`{opt.node}.Ref.Children[1].Labels["first"].Value`,
		`{opt.node}.Ref.Children[1].Labels["first"].Value`,
	}
	assert.DeepEqual(t, matches, expected)
}

func TestPathStringFromSlice(t *testing.T) {
	fixture := []node{
		{
			Ref: &node{
				Children: []node{
					{},
					{
						Labels: map[string]node{
							"first": {},
							"second": {
								Ref: &node{Value: nodeValue{Value: 3}},
							},
						},
					},
				},
			},
		},
	}

	spec := "Ref.Children.Labels.Ref.Value"
	matches := matchPaths(fixture, PathString(spec))
	expected := []string{
		`{[]opt.node}[0].Ref.Children[1].Labels["second"].Ref.Value`,
		`{[]opt.node}[0].Ref.Children[1].Labels["second"].Ref.Value`,
		`{[]opt.node}[0].Ref.Children[1].Labels["second"].Ref.Value`,
		`{[]opt.node}[0].Ref.Children[1].Labels["second"].Ref.Value`,
	}
	assert.DeepEqual(t, matches, expected)
}

func TestPathField(t *testing.T) {
	fixture := node{
		Value: nodeValue{Value: 3},
		Children: []node{
			{},
			{Value: nodeValue{Value: 2}},
			{Ref: &node{Value: nodeValue{Value: 9}}},
		},
	}

	filter := PathField(nodeValue{}, "Value")
	matches := matchPaths(fixture, filter)
	expected := []string{
		"{opt.node}.Children[0].Value.Value",
		"{opt.node}.Children[0].Value.Value",
		"{opt.node}.Children[1].Value.Value",
		"{opt.node}.Children[1].Value.Value",
		"{opt.node}.Children[2].Ref.Value.Value",
		"{opt.node}.Children[2].Ref.Value.Value",
		"{opt.node}.Children[2].Value.Value",
		"{opt.node}.Children[2].Value.Value",
		"{opt.node}.Value.Value",
	}
	assert.DeepEqual(t, matches, expected)
}

func TestPathDebug(t *testing.T) {
	fixture := node{
		Value: nodeValue{Value: 3},
		Children: []node{
			{Ref: &node{Value: nodeValue{Value: 9}}},
		},
		Labels: map[string]node{
			"label1": {},
		},
	}
	gocmp.Equal(fixture, fixture, gocmp.FilterPath(PathDebug, gocmp.Ignore()))
}
