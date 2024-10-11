// This test calls a method that doesn't exist.

--> {"jsonrpc": "2.0", "id": 2, "method": "invalid_method", "params": [2, 3]}
<-- {"jsonrpc":"2.0","id":2,"error":{"code":-32601,"message":"the method invalid_method does not exist/is not available"}}
