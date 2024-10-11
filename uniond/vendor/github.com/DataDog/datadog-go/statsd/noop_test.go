package statsd

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestNoOpClient(t *testing.T) {
	a := assert.New(t)
	c := NoOpClient{}
	tags := []string{"a:b"}

	a.Nil(c.Gauge("asd", 123.4, tags, 56.0))
	a.Nil(c.Count("asd", 1234, tags, 56.0))
	a.Nil(c.Histogram("asd", 12.34, tags, 56.0))
	a.Nil(c.Distribution("asd", 1.234, tags, 56.0))
	a.Nil(c.Decr("asd", tags, 56.0))
	a.Nil(c.Incr("asd", tags, 56.0))
	a.Nil(c.Set("asd", "asd", tags, 56.0))
	a.Nil(c.Timing("asd", time.Second, tags, 56.0))
	a.Nil(c.TimeInMilliseconds("asd", 1234.5, tags, 56.0))
	a.Nil(c.Event(nil))
	a.Nil(c.SimpleEvent("asd", "zxc"))
	a.Nil(c.ServiceCheck(nil))
	a.Nil(c.SimpleServiceCheck("asd", Ok))
	a.Nil(c.Close())
	a.Nil(c.Flush())
	a.Nil(c.SetWriteTimeout(time.Second))
}
