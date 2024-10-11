package grpc_logrus

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestDurationToTimeMillisField(t *testing.T) {
	_, val := DurationToTimeMillisField(time.Microsecond * 100)
	assert.Equal(t, val.(float32), float32(0.1), "sub millisecond values should be correct")
}
