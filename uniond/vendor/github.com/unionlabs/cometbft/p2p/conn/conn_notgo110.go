//go:build !go1.10
// +build !go1.10

package conn

import (
	"net"
	"time"
)

// Only Go1.10 has a proper net.Conn implementation that
// has the SetDeadline method implemented as per
// https://github.com/golang/go/commit/e2dd8ca946be884bb877e074a21727f1a685a706
// lest we run into problems like
// https://github.com/tendermint/tendermint/issues/851
// so for go versions < Go1.10 use our custom net.Conn creator
// that doesn't return an `Unimplemented error` for net.Conn.
// Before https://github.com/cometbft/cometbft/commit/49faa79bdce5663894b3febbf4955fb1d172df04
// we hadn't cared about errors from SetDeadline so swallow them up anyways.
type pipe struct {
	net.Conn
}

func (p *pipe) SetDeadline(t time.Time) error {
	return nil
}

func NetPipe() (net.Conn, net.Conn) {
	p1, p2 := net.Pipe()
	return &pipe{p1}, &pipe{p2}
}

var _ net.Conn = (*pipe)(nil)
