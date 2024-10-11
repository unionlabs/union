package flags

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestEnableHTTPPrysmAPI(t *testing.T) {
	assert.Equal(t, true, EnableHTTPPrysmAPI("prysm"))
	assert.Equal(t, true, EnableHTTPPrysmAPI("prysm,foo"))
	assert.Equal(t, true, EnableHTTPPrysmAPI("foo,prysm"))
	assert.Equal(t, true, EnableHTTPPrysmAPI("prysm,prysm"))
	assert.Equal(t, true, EnableHTTPPrysmAPI("PrYsM"))
	assert.Equal(t, false, EnableHTTPPrysmAPI("foo"))
	assert.Equal(t, false, EnableHTTPPrysmAPI(""))
}

func TestEnableHTTPEthAPI(t *testing.T) {
	assert.Equal(t, true, EnableHTTPEthAPI("eth"))
	assert.Equal(t, true, EnableHTTPEthAPI("eth,foo"))
	assert.Equal(t, true, EnableHTTPEthAPI("foo,eth"))
	assert.Equal(t, true, EnableHTTPEthAPI("eth,eth"))
	assert.Equal(t, true, EnableHTTPEthAPI("EtH"))
	assert.Equal(t, false, EnableHTTPEthAPI("foo"))
	assert.Equal(t, false, EnableHTTPEthAPI(""))
}

func TestEnableApi(t *testing.T) {
	assert.Equal(t, true, enableAPI("foo", "foo"))
	assert.Equal(t, true, enableAPI("foo,bar", "foo"))
	assert.Equal(t, true, enableAPI("bar,foo", "foo"))
	assert.Equal(t, true, enableAPI("foo,foo", "foo"))
	assert.Equal(t, true, enableAPI("FoO", "foo"))
	assert.Equal(t, false, enableAPI("bar", "foo"))
	assert.Equal(t, false, enableAPI("", "foo"))
}
