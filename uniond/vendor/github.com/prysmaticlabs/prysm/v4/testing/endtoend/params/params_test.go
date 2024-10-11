package params

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func Test_port(t *testing.T) {
	var existingRegistrations []int

	p, err := port(2000, 3, 0, &existingRegistrations)
	require.NoError(t, err)
	assert.Equal(t, 2000, p)
	p, err = port(2000, 3, 1, &existingRegistrations)
	require.NoError(t, err)
	assert.Equal(t, 2016, p)
	p, err = port(2000, 3, 2, &existingRegistrations)
	require.NoError(t, err)
	assert.Equal(t, 2032, p)
	_, err = port(2000, 3, 2, &existingRegistrations)
	assert.NotNil(t, err)
	// We pass the last unavailable port
	_, err = port(2047, 3, 0, &existingRegistrations)
	assert.NotNil(t, err)
}

func TestStandardPorts(t *testing.T) {
	var existingRegistrations []int
	testPorts := &ports{}
	assert.NoError(t, initializeStandardPorts(2, 0, testPorts, &existingRegistrations))
	assert.Equal(t, 16, len(existingRegistrations))
	assert.NotEqual(t, 0, testPorts.PrysmBeaconNodeGatewayPort)
	assert.NotEqual(t, 0, testPorts.PrysmBeaconNodeTCPPort)
	assert.NotEqual(t, 0, testPorts.JaegerTracingPort)
}

func TestMulticlientPorts(t *testing.T) {
	var existingRegistrations []int
	testPorts := &ports{}
	assert.NoError(t, initializeMulticlientPorts(2, 0, testPorts, &existingRegistrations))
	assert.Equal(t, 3, len(existingRegistrations))
	assert.NotEqual(t, 0, testPorts.LighthouseBeaconNodeHTTPPort)
	assert.NotEqual(t, 0, testPorts.LighthouseBeaconNodeMetricsPort)
	assert.NotEqual(t, 0, testPorts.LighthouseBeaconNodeP2PPort)
}
