package zk

import (
	"fmt"

	"github.com/go-zookeeper/zk"

	"github.com/go-kit/log"
)

// wrapLogger wraps a Go kit logger so we can use it as the logging service for
// the ZooKeeper library, which expects a Printf method to be available.
type wrapLogger struct {
	log.Logger
}

func (logger wrapLogger) Printf(format string, args ...interface{}) {
	logger.Log("msg", fmt.Sprintf(format, args...))
}

// withLogger replaces the ZooKeeper library's default logging service with our
// own Go kit logger.
func withLogger(logger log.Logger) func(c *zk.Conn) {
	return func(c *zk.Conn) {
		c.SetLogger(wrapLogger{logger})
	}
}
