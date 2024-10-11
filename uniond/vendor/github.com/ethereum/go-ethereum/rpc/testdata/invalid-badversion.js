// This test checks processing of messages with invalid Version.

--> {"jsonrpc":"2.0","id":1,"method":"test_echo","params":["x", 3]}
<-- {"jsonrpc":"2.0","id":1,"result":{"String":"x","Int":3,"Args":null}}

--> {"jsonrpc":"2.1","id":1,"method":"test_echo","params":["x", 3]}
<-- {"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"invalid request"}}

--> {"jsonrpc":"go-ethereum","id":1,"method":"test_echo","params":["x", 3]}
<-- {"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"invalid request"}}

--> {"jsonrpc":1,"id":1,"method":"test_echo","params":["x", 3]}
<-- {"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"invalid request"}}

--> {"jsonrpc":2.0,"id":1,"method":"test_echo","params":["x", 3]}
<-- {"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"invalid request"}}

--> {"id":1,"method":"test_echo","params":["x", 3]}
<-- {"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"invalid request"}}
