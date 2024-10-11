# ADR 008: SocketPV

Tendermint node's should support only two in-process PrivValidator
implementations:

- FilePV uses an unencrypted private key in a "priv_validator.json" file - no
  configuration required (just `tendermint init validator`).
- TCPVal and IPCVal use TCP and Unix sockets respectively to send signing requests
  to another process - the user is responsible for starting that process themselves.

Both TCPVal and IPCVal addresses can be provided via flags at the command line
or in the configuration file; TCPVal addresses must be of the form
`tcp://<ip_address>:<port>` and IPCVal addresses `unix:///path/to/file.sock` -
doing so will cause Tendermint to ignore any private validator files.

TCPVal will listen on the given address for incoming connections from an external
private validator process. It will halt any operation until at least one external
process successfully connected.

The external priv_validator process will dial the address to connect to
Tendermint, and then Tendermint will send requests on the ensuing connection to
sign votes and proposals. Thus the external process initiates the connection,
but the Tendermint process makes all requests. In a later stage we're going to
support multiple validators for fault tolerance. To prevent double signing they
need to be synced, which is deferred to an external solution (see #1185).

Conversely, IPCVal will make an outbound connection to an existing socket opened
by the external validator process.

In addition, Tendermint will provide implementations that can be run in that
external process. These include:

- FilePV will encrypt the private key, and the user must enter password to
  decrypt key when process is started.
- LedgerPV uses a Ledger Nano S to handle all signing.
