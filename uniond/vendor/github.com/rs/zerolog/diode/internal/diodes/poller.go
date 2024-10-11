package diodes

import (
	"context"
	"time"
)

// Diode is any implementation of a diode.
type Diode interface {
	Set(GenericDataType)
	TryNext() (GenericDataType, bool)
}

// Poller will poll a diode until a value is available.
type Poller struct {
	Diode
	interval time.Duration
	ctx      context.Context
}

// PollerConfigOption can be used to setup the poller.
type PollerConfigOption func(*Poller)

// WithPollingInterval sets the interval at which the diode is queried
// for new data. The default is 10ms.
func WithPollingInterval(interval time.Duration) PollerConfigOption {
	return func(c *Poller) {
		c.interval = interval
	}
}

// WithPollingContext sets the context to cancel any retrieval (Next()). It
// will not change any results for adding data (Set()). Default is
// context.Background().
func WithPollingContext(ctx context.Context) PollerConfigOption {
	return func(c *Poller) {
		c.ctx = ctx
	}
}

// NewPoller returns a new Poller that wraps the given diode.
func NewPoller(d Diode, opts ...PollerConfigOption) *Poller {
	p := &Poller{
		Diode:    d,
		interval: 10 * time.Millisecond,
		ctx:      context.Background(),
	}

	for _, o := range opts {
		o(p)
	}

	return p
}

// Next polls the diode until data is available or until the context is done.
// If the context is done, then nil will be returned.
func (p *Poller) Next() GenericDataType {
	for {
		data, ok := p.Diode.TryNext()
		if !ok {
			if p.isDone() {
				return nil
			}

			time.Sleep(p.interval)
			continue
		}
		return data
	}
}

func (p *Poller) isDone() bool {
	select {
	case <-p.ctx.Done():
		return true
	default:
		return false
	}
}
