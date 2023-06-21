package custom_query_test

import (
	"encoding/json"
	"fmt"
	"os"
	"testing"

	"union/app/custom_query"

	"github.com/stretchr/testify/assert"
)

func TestAggregate(t *testing.T) {
	entries, err := os.ReadDir("./testdata")
	assert.NoError(t, err)
	for _, entry := range entries {
		content, err := os.ReadFile(fmt.Sprintf("./testdata/%s", entry.Name()))
		assert.NoError(t, err)

		var verify custom_query.QueryAggregateVerify
		err = json.Unmarshal(content, &verify)
		assert.NoError(t, err)

		msg := [32]byte{}
		for i := 0; i < 32; i++ {
			msg[i] = verify.Message[i]
		}

		result, err := custom_query.VerifySignature(verify.Signature, msg, verify.PublicKeys)
		assert.NoError(t, err)

		assert.Equal(t, true, result)
	}
}
