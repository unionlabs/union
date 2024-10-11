/*
Package encoder allows for registering custom data encoders for information
sent as raw bytes over the wire via p2p to other nodes. Examples of encoders
are SSZ (SimpleSerialize), SSZ with Snappy compression, among others. Providing
an abstract interface for these encoders allows for future flexibility of
Ethereum beacon node p2p.
*/
package encoder
