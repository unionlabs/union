/*
This package is only intended to be used for testing core IBC. In order to maintain secure
testing, we need to do message passing and execution which requires connecting an IBC application
module that fulfills all the callbacks. We cannot connect to ibc-transfer which does not support
all channel types so instead we create a mock application module which does nothing. It simply
return nil in all cases so no error ever occurs. It is intended to be as minimal and lightweight
as possible and should never import simapp.
*/
package mock
