package test_vector_utils

import (
	fiatshamir "github.com/consensys/gnark-crypto/fiat-shamir"
	"github.com/consensys/gnark-crypto/internal/generator/test_vector_utils/small_rational/test_vector_utils"
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestCounterTranscriptInequality(t *testing.T) {
	const challengeName = "fC.0"
	t1 := fiatshamir.NewTranscript(test_vector_utils.NewMessageCounter(1, 1), challengeName)
	t2 := fiatshamir.NewTranscript(test_vector_utils.NewMessageCounter(0, 1), challengeName)
	var c1, c2 []byte
	var err error
	c1, err = t1.ComputeChallenge(challengeName)
	assert.NoError(t, err)
	c2, err = t2.ComputeChallenge(challengeName)
	assert.NoError(t, err)
	assert.NotEqual(t, c1, c2)
}
