/*
Package privval provides different implementations of the types.PrivValidator.

# FilePV

FilePV is the simplest implementation and developer default.
It uses one file for the private key and another to store state.

# SignerListenerEndpoint

SignerListenerEndpoint establishes a connection to an external process,
like a Key Management Server (KMS), using a socket.
SignerListenerEndpoint listens for the external KMS process to dial in.
SignerListenerEndpoint takes a listener, which determines the type of connection
(ie. encrypted over tcp, or unencrypted over unix).

# SignerDialerEndpoint

SignerDialerEndpoint is a simple wrapper around a net.Conn. It's used by both IPCVal and TCPVal.

# SignerClient

SignerClient handles remote validator connections that provide signing services.
In production, it's recommended to wrap it with RetrySignerClient to avoid
termination in case of temporary errors.
*/
package privval
