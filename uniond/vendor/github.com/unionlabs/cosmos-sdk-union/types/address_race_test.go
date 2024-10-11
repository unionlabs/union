package types_test

import (
	"encoding/binary"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/types"
)

// generates AccAddress with `prefix` and calls String method
func addressStringCaller(require *require.Assertions, prefix byte, max uint32, cancel chan bool, done chan<- bool) {
	bz := make([]byte, 5) // prefix + 4 bytes for uint
	bz[0] = prefix
	for i := uint32(0); ; i++ {
		if i >= max {
			i = 0
		}
		select {
		case <-cancel:
			done <- true
			return
		default:
			binary.BigEndian.PutUint32(bz[1:], i)
			str := types.AccAddress(bz).String()
			require.True(str != "")
		}

	}
}

func (s *addressTestSuite) TestAddressRace() {
	if testing.Short() {
		s.T().Skip("AddressRace test is not short")
	}

	workers := 4
	done := make(chan bool, workers)
	cancel := make(chan bool)

	for i := byte(1); i <= 2; i++ { // works which will loop in first 100 addresses
		go addressStringCaller(s.Require(), i, 100, cancel, done)
	}

	for i := byte(1); i <= 2; i++ { // works which will generate 1e6 new addresses
		go addressStringCaller(s.Require(), i, 1000000, cancel, done)
	}

	<-time.After(time.Millisecond * 30)
	close(cancel)

	// cleanup
	for i := 0; i < 4; i++ {
		<-done
	}
}
