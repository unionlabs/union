// This test calls the test_echo method.

--> {"jsonrpc": "2.0", "id": 2, "method": "test_echo", "params": []}
<-- {"jsonrpc":"2.0","id":2,"error":{"code":-32602,"message":"missing value for required argument 0"}}

--> {"jsonrpc": "2.0", "id": 2, "method": "test_echo", "params": ["x"]}
<-- {"jsonrpc":"2.0","id":2,"error":{"code":-32602,"message":"missing value for required argument 1"}}

--> {"jsonrpc": "2.0", "id": 2, "method": "test_echo", "params": ["x", 3]}
<-- {"jsonrpc":"2.0","id":2,"result":{"String":"x","Int":3,"Args":null}}

--> {"jsonrpc": "2.0", "id": 2, "method": "test_echo", "params": ["x", 3, {"S": "foo"}]}
<-- {"jsonrpc":"2.0","id":2,"result":{"String":"x","Int":3,"Args":{"S":"foo"}}}

--> {"jsonrpc": "2.0", "id": 2, "method": "test_echoWithCtx", "params": ["x", 3, {"S": "foo"}]}
<-- {"jsonrpc":"2.0","id":2,"result":{"String":"x","Int":3,"Args":{"S":"foo"}}}
