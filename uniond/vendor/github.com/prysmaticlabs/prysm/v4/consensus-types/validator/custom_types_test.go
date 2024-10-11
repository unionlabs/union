package validator

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"k8s.io/apimachinery/pkg/util/yaml"
)

func Test_customUint_UnmarshalJSON(t *testing.T) {
	type Custom struct {
		Test Uint64 `json:"test"`
	}
	tests := []struct {
		name             string
		jsonString       string
		number           uint64
		wantUnmarshalErr string
	}{
		{
			name:       "Happy Path string",
			jsonString: `{"test": "123441"}`,
			number:     123441,
		},
		{
			name:       "Happy Path number",
			jsonString: `{"test": 123441}`,
			number:     123441,
		},
		{
			name:             "empty",
			jsonString:       `{"test":""}`,
			wantUnmarshalErr: "error unmarshaling JSON",
		},
		{
			name:             "digits more than uint64",
			jsonString:       `{"test":"8888888888888888888888888888888888888888888888888888888888888"}`,
			wantUnmarshalErr: "error unmarshaling JSON",
		},
		{
			name:             "not a uint64",
			jsonString:       `{"test":"one hundred"}`,
			wantUnmarshalErr: "error unmarshaling JSON",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var to Custom
			err := yaml.Unmarshal([]byte(tt.jsonString), &to)
			if tt.wantUnmarshalErr != "" {
				require.ErrorContains(t, tt.wantUnmarshalErr, err)
			} else {
				require.NoError(t, err)
			}
			require.Equal(t, tt.number, uint64(to.Test))
		})
	}
}
