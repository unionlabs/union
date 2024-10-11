// This test checks the behavior of batches with invalid elements.
// Empty batches are not allowed. Batches may contain junk.

--> []
<-- {"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"empty batch"}}

--> [1]
<-- [{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"invalid request"}}]

--> [1,2,3]
<-- [{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"invalid request"}},{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"invalid request"}},{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"invalid request"}}]

--> [null]
<-- [{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"invalid request"}}]

--> [{"jsonrpc":"2.0","id":1,"method":"test_echo","params":["foo",1]},55,{"jsonrpc":"2.0","id":2,"method":"unknown_method"},{"foo":"bar"}]
<-- [{"jsonrpc":"2.0","id":1,"result":{"String":"foo","Int":1,"Args":null}},{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"invalid request"}},{"jsonrpc":"2.0","id":2,"error":{"code":-32601,"message":"the method unknown_method does not exist/is not available"}},{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"invalid request"}}]
