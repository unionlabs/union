package client

// ABCIQueryOptions can be used to provide options for ABCIQuery call other
// than the DefaultABCIQueryOptions.
type ABCIQueryOptions struct {
	Height int64
	Prove  bool
}

// DefaultABCIQueryOptions are latest height (0) and prove false.
var DefaultABCIQueryOptions = ABCIQueryOptions{Height: 0, Prove: false}
