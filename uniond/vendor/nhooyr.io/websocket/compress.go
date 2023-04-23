package websocket

// CompressionMode represents the modes available to the deflate extension.
// See https://tools.ietf.org/html/rfc7692
//
// A compatibility layer is implemented for the older deflate-frame extension used
// by safari. See https://tools.ietf.org/html/draft-tyoshino-hybi-websocket-perframe-deflate-06
// It will work the same in every way except that we cannot signal to the peer we
// want to use no context takeover on our side, we can only signal that they should.
// It is however currently disabled due to Safari bugs. See https://github.com/nhooyr/websocket/issues/218
type CompressionMode int

const (
	// CompressionNoContextTakeover grabs a new flate.Reader and flate.Writer as needed
	// for every message. This applies to both server and client side.
	//
	// This means less efficient compression as the sliding window from previous messages
	// will not be used but the memory overhead will be lower if the connections
	// are long lived and seldom used.
	//
	// The message will only be compressed if greater than 512 bytes.
	CompressionNoContextTakeover CompressionMode = iota

	// CompressionContextTakeover uses a flate.Reader and flate.Writer per connection.
	// This enables reusing the sliding window from previous messages.
	// As most WebSocket protocols are repetitive, this can be very efficient.
	// It carries an overhead of 8 kB for every connection compared to CompressionNoContextTakeover.
	//
	// If the peer negotiates NoContextTakeover on the client or server side, it will be
	// used instead as this is required by the RFC.
	CompressionContextTakeover

	// CompressionDisabled disables the deflate extension.
	//
	// Use this if you are using a predominantly binary protocol with very
	// little duplication in between messages or CPU and memory are more
	// important than bandwidth.
	CompressionDisabled
)
