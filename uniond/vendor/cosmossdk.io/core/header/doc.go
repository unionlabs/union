/*
Package header defines a generalized Header type that all consensus & networking layers must provide.

If modules need access to the current block header information, like height, hash, time, or chain ID
they should use the Header Service interface.
*/
package header
