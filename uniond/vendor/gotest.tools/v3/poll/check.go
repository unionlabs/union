package poll

import (
	"net"
	"os"
)

// Check is a function which will be used as check for the [WaitOn] method.
type Check func(t LogT) Result

// FileExists looks on filesystem and check that path exists.
func FileExists(path string) Check {
	return func(t LogT) Result {
		if h, ok := t.(helperT); ok {
			h.Helper()
		}

		_, err := os.Stat(path)
		switch {
		case os.IsNotExist(err):
			t.Logf("waiting on file %s to exist", path)
			return Continue("file %s does not exist", path)
		case err != nil:
			return Error(err)
		default:
			return Success()
		}
	}
}

// Connection try to open a connection to the address on the
// named network. See [net.Dial] for a description of the network and
// address parameters.
func Connection(network, address string) Check {
	return func(t LogT) Result {
		if h, ok := t.(helperT); ok {
			h.Helper()
		}

		_, err := net.Dial(network, address)
		if err != nil {
			t.Logf("waiting on socket %s://%s to be available...", network, address)
			return Continue("socket %s://%s not available", network, address)
		}
		return Success()
	}
}
