package types

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	// sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/stretchr/testify/require"
)

func TestIsJSONObjectWithTopLevelKey(t *testing.T) {
	specs := map[string]struct {
		src         []byte
		allowedKeys []string
		expResult   bool
		expErr      error
	}{
		"happy": {
			src:         []byte(`{"msg": {"foo":"bar"}}`),
			allowedKeys: []string{"msg"},
			expResult:   true,
		},
		"happy with many allowed keys 1": {
			src:         []byte(`{"claim": {"foo":"bar"}}`),
			allowedKeys: []string{"claim", "swap", "burn", "mint"},
			expResult:   true,
		},
		"happy with many allowed keys 2": {
			src:         []byte(`{"burn": {"foo":"bar"}}`),
			allowedKeys: []string{"claim", "swap", "burn", "mint"},
			expResult:   true,
		},
		"happy with many allowed keys 3": {
			src:         []byte(`{"mint": {"foo":"bar"}}`),
			allowedKeys: []string{"claim", "swap", "burn", "mint"},
			expResult:   true,
		},
		"happy with number": {
			src:         []byte(`{"msg": 123}`),
			allowedKeys: []string{"msg"},
			expResult:   true,
		},
		"happy with array": {
			src:         []byte(`{"msg": [1, 2, 3, 4]}`),
			allowedKeys: []string{"msg"},
			expResult:   true,
		},
		"happy with null": {
			src:         []byte(`{"msg": null}`),
			allowedKeys: []string{"msg"},
			expResult:   true,
		},
		"happy with whitespace": {
			src: []byte(`{
				"msg":	null    }`),
			allowedKeys: []string{"msg"},
			expResult:   true,
		},
		"happy with escaped key": {
			src:         []byte(`{"event\u2468thing": {"foo":"bar"}}`),
			allowedKeys: []string{"event⑨thing"},
			expResult:   true,
		},

		// Invalid JSON object
		"errors for bytes that are no JSON": {
			src:         []byte(`nope`),
			allowedKeys: []string{"claim"},
			expErr:      ErrInvalid,
		},
		"false for valid JSON (string)": {
			src:         []byte(`"nope"`),
			allowedKeys: []string{"claim"},
			expResult:   false,
		},
		"false for valid JSON (array)": {
			src:         []byte(`[1, 2, 3]`),
			allowedKeys: []string{"claim"},
			expResult:   false,
		},
		// not supported: https://github.com/golang/go/issues/24415
		// "errors for duplicate key": {
		//	src:         []byte(`{"claim": "foo", "claim":"bar"}`),
		//	allowedKeys: []string{"claim"},
		//	expErr:      ErrNotAJSONObject,
		// },

		// Not one top-level key
		"false for no top-level key": {
			src:         []byte(`{}`),
			allowedKeys: []string{"claim"},
			expResult:   false,
		},
		"false for multiple top-level keys": {
			src:         []byte(`{"claim": {}, "and_swap": {}}`),
			allowedKeys: []string{"claim"},
			expResult:   false,
		},

		// Wrong top-level key
		"wrong top-level key 1": {
			src:         []byte(`{"claim": {}}`),
			allowedKeys: []string{""},
			expResult:   false,
		},
		"wrong top-level key 2": {
			src:         []byte(`{"claim": {}}`),
			allowedKeys: []string{"swap", "burn", "mint"},
			expResult:   false,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			exists, gotErr := isJSONObjectWithTopLevelKey(spec.src, spec.allowedKeys)
			if spec.expErr != nil {
				assert.ErrorIs(t, gotErr, spec.expErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, spec.expResult, exists)
		})
	}
}

func TestDuplicateKeyGivesSameResult(t *testing.T) {
	jsonBytes := []byte(`{"event⑨thing": "foo", "event⑨thing":"bar"}`)
	for i := 0; i < 10000; i++ {
		document := map[string]interface{}{}
		require.NoError(t, json.Unmarshal(jsonBytes, &document))
		assert.Equal(t, "bar", document["event⑨thing"])
	}
}
