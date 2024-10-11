// This test checks processing of messages that contain just the ID and nothing else.

--> {"id":1}
<-- {"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"invalid request"}}

--> {"jsonrpc":"2.0","id":1}
<-- {"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"invalid request"}}
