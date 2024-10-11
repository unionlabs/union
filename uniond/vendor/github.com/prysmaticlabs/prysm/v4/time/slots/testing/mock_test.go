package testing

import (
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

var _ slots.Ticker = (*MockTicker)(nil)
