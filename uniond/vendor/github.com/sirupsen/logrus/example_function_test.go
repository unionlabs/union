package logrus_test

import (
	"testing"

	log "github.com/sirupsen/logrus"
	"github.com/stretchr/testify/assert"
)

func TestLogger_LogFn(t *testing.T) {
	log.SetFormatter(&log.JSONFormatter{})
	log.SetLevel(log.WarnLevel)

	notCalled := 0
	log.InfoFn(func() []interface{} {
		notCalled++
		return []interface{}{
			"Hello",
		}
	})
	assert.Equal(t, 0, notCalled)

	called := 0
	log.ErrorFn(func() []interface{} {
		called++
		return []interface{}{
			"Oopsi",
		}
	})
	assert.Equal(t, 1, called)
}
