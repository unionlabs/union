package ed25519

/*
This package contains a wrapper around crypto/ed22519 to make it comply with the crypto interfaces.

This package employs zip215 rules. We use https://github.com/hdevalence/ed25519consensus verification function. This is done in order to keep compatibility with Tendermints ed25519 implementation.
	- https://github.com/cometbft/cometbft/blob/master/crypto/ed25519/ed25519.go#L155

This package works with correctly generated signatures. To read more about what this means see https://hdevalence.ca/blog/2020-10-04-its-25519am

*/
