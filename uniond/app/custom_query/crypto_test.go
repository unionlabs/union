package custom_query_test

import (
	"encoding/json"
	"fmt"
	"os"
	"testing"

	"union/app/custom_query"

	"github.com/stretchr/testify/assert"
)

const CORRECT_DATA_DIR = "./testdata/correct_data"
const CORRECT_DATA_DIR_FORMAT = "./testdata/correct_data/%s"

func TestVerifyWorks(t *testing.T) {
	entries, err := os.ReadDir(CORRECT_DATA_DIR)
	assert.NoError(t, err)
	for _, entry := range entries {
		content, err := os.ReadFile(fmt.Sprintf(CORRECT_DATA_DIR_FORMAT, entry.Name()))
		assert.NoError(t, err)

		var verify custom_query.QueryAggregateVerify
		err = json.Unmarshal(content, &verify)
		assert.NoError(t, err)

		msg := [32]byte{}
		copy(msg[:], verify.Message[:])

		result, err := custom_query.VerifySignature(verify.Signature, msg, verify.PublicKeys)
		assert.NoError(t, err)

		assert.Equal(t, true, result)
	}
}

func TestVerifyFailsToVerifyWhenIncorrectPublicKey(t *testing.T) {
	entries, err := os.ReadDir(CORRECT_DATA_DIR)
	assert.NoError(t, err)
	for _, entry := range entries {
		content, err := os.ReadFile(fmt.Sprintf(CORRECT_DATA_DIR_FORMAT, entry.Name()))
		assert.NoError(t, err)

		var verify custom_query.QueryAggregateVerify
		err = json.Unmarshal(content, &verify)
		assert.NoError(t, err)

		msg := [32]byte{}
		copy(msg[:], verify.Message[:])

		// We copy a valid public key so that parsing would still work
		verify.PublicKeys[0] = verify.PublicKeys[1]

		result, err := custom_query.VerifySignature(verify.Signature, msg, verify.PublicKeys)
		assert.NoError(t, err)

		assert.Equal(t, false, result)
	}
}

func TestVerifyFailsToVerifyWhenMissingOnePublicKey(t *testing.T) {
	entries, err := os.ReadDir(CORRECT_DATA_DIR)
	assert.NoError(t, err)
	for _, entry := range entries {
		content, err := os.ReadFile(fmt.Sprintf(CORRECT_DATA_DIR_FORMAT, entry.Name()))
		assert.NoError(t, err)

		var verify custom_query.QueryAggregateVerify
		err = json.Unmarshal(content, &verify)
		assert.NoError(t, err)

		msg := [32]byte{}
		copy(msg[:], verify.Message[:])

		verify.PublicKeys = verify.PublicKeys[:(len(verify.PublicKeys) - 1)]

		result, err := custom_query.VerifySignature(verify.Signature, msg, verify.PublicKeys)
		assert.NoError(t, err)

		assert.Equal(t, false, result)
	}
}

func TestVerifyFailsToVerifyWhenEmptyPublicKeys(t *testing.T) {
	entries, err := os.ReadDir(CORRECT_DATA_DIR)
	assert.NoError(t, err)
	for _, entry := range entries {
		content, err := os.ReadFile(fmt.Sprintf(CORRECT_DATA_DIR_FORMAT, entry.Name()))
		assert.NoError(t, err)

		var verify custom_query.QueryAggregateVerify
		err = json.Unmarshal(content, &verify)
		assert.NoError(t, err)

		msg := [32]byte{}
		copy(msg[:], verify.Message[:])

		_, err = custom_query.VerifySignature(verify.Signature, msg, [][]byte{})
		assert.Error(t, err)
	}
}

func TestVerifyFailsToVerifyWhenIncorrectMessage(t *testing.T) {
	entries, err := os.ReadDir(CORRECT_DATA_DIR)
	assert.NoError(t, err)
	for _, entry := range entries {
		content, err := os.ReadFile(fmt.Sprintf(CORRECT_DATA_DIR_FORMAT, entry.Name()))
		assert.NoError(t, err)

		var verify custom_query.QueryAggregateVerify
		err = json.Unmarshal(content, &verify)
		assert.NoError(t, err)

		msg := [32]byte{}
		copy(msg[:], verify.Message[:])

		msg[0] = ^uint8(0) - msg[0]

		result, err := custom_query.VerifySignature(verify.Signature, msg, verify.PublicKeys)
		assert.NoError(t, err)

		assert.Equal(t, false, result)
	}
}

func TestVerifyFailsToVerifyWhenIncorrectSignature(t *testing.T) {
	content, err := os.ReadFile("./testdata/incorrect_signature.json")
	assert.NoError(t, err)

	var verify custom_query.QueryAggregateVerify
	err = json.Unmarshal(content, &verify)
	assert.NoError(t, err)

	msg := [32]byte{}
	copy(msg[:], verify.Message[:])

	result, err := custom_query.VerifySignature(verify.Signature, msg, verify.PublicKeys)
	assert.NoError(t, err)

	assert.Equal(t, false, result)
}
