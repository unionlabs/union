package filters

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestQueryFilter_ChainsCorrectly(t *testing.T) {
	f := NewFilter().
		SetStartSlot(2).
		SetEndSlot(4).
		SetParentRoot([]byte{3, 4, 5})

	filterSet := f.Filters()
	assert.Equal(t, 3, len(filterSet), "Unexpected number of filters")
	for k, v := range filterSet {
		switch k {
		case StartSlot:
			t.Log(v.(primitives.Slot))
		case EndSlot:
			t.Log(v.(primitives.Slot))
		case ParentRoot:
			t.Log(v.([]byte))
		default:
			t.Log("Unknown filter type")
		}
	}
}
