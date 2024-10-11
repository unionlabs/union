package armor

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestArmor(t *testing.T) {
	blockType := "MINT TEST"
	data := []byte("somedata")
	armorStr, err := EncodeArmor(blockType, nil, data)
	require.NoError(t, err, "%+v", err)

	// Decode armorStr and test for equivalence.
	blockType2, _, data2, err := DecodeArmor(armorStr)
	require.NoError(t, err, "%+v", err)
	assert.Equal(t, blockType, blockType2)
	assert.Equal(t, data, data2)
}
