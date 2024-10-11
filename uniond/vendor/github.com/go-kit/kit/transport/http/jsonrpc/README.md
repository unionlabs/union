# JSON RPC

[JSON RPC](http://www.jsonrpc.org) is "A light weight remote procedure call protocol". It allows for the creation of simple RPC-style APIs with human-readable messages that are front-end friendly.

## Using JSON RPC with Go-Kit
Using JSON RPC and go-kit together is quite simple.

A JSON RPC _server_ acts as an [HTTP Handler](https://godoc.org/net/http#Handler), receiving all requests to the JSON RPC's URL. The server looks at the `method` property of the [Request Object](http://www.jsonrpc.org/specification#request_object), and routes it to the corresponding code.

Each JSON RPC _method_ is implemented as an `EndpointCodec`, a go-kit [Endpoint](https://godoc.org/github.com/go-kit/kit/endpoint#Endpoint), sandwiched between a decoder and encoder. The decoder picks apart the JSON RPC request params, which can be passed to your endpoint. The encoder receives the output from the endpoint and encodes a JSON-RPC result.

## Example — Add Service
Let's say we want a service that adds two ints together. We'll serve this at `http://localhost/rpc`. So a request to our `sum` method will be a POST to `http://localhost/rpc` with a request body of:

	{
	    "id": 123,
	    "jsonrpc": "2.0",
	    "method": "sum",
	    "params": {
	    	"A": 2,
	    	"B": 2
	    }
	}

### `EndpointCodecMap`
The routing table for incoming JSON RPC requests is the `EndpointCodecMap`. The key of the map is the JSON RPC method name. Here, we're routing the `sum` method to an `EndpointCodec` wrapped around `sumEndpoint`.

	jsonrpc.EndpointCodecMap{
		"sum": jsonrpc.EndpointCodec{
			Endpoint: sumEndpoint,
			Decode:   decodeSumRequest,
			Encode:   encodeSumResponse,
		},
	}

### Decoder
	type DecodeRequestFunc func(context.Context, json.RawMessage) (request interface{}, err error)

A `DecodeRequestFunc` is given the raw JSON from the `params` property of the Request object, _not_ the whole request object. It returns an object that will be the input to the Endpoint. For our purposes, the output should be a SumRequest, like this:

	type SumRequest struct {
		A, B int
	}

So here's our decoder:

	func decodeSumRequest(ctx context.Context, msg json.RawMessage) (interface{}, error) {
		var req SumRequest
		err := json.Unmarshal(msg, &req)
		if err != nil {
			return nil, err
		}
		return req, nil
	}

So our `SumRequest` will now be passed to the endpoint. Once the endpoint has done its work, we hand over to the…

### Encoder
The encoder takes the output of the endpoint, and builds the raw JSON message that will form the `result` field of a [Response Object](http://www.jsonrpc.org/specification#response_object). Our result is going to be a plain int. Here's our encoder:

	func encodeSumResponse(ctx context.Context, result interface{}) (json.RawMessage, error) {
		sum, ok := result.(int)
		if !ok {
			return nil, errors.New("result is not an int")
		}
		b, err := json.Marshal(sum)
		if err != nil {
			return nil, err
		}
		return b, nil
	}

### Server
Now that we have an EndpointCodec with decoder, endpoint, and encoder, we can wire up the server:

	handler := jsonrpc.NewServer(jsonrpc.EndpointCodecMap{
		"sum": jsonrpc.EndpointCodec{
			Endpoint: sumEndpoint,
			Decode:   decodeSumRequest,
			Encode:   encodeSumResponse,
		},
	})
	http.Handle("/rpc", handler)
	http.ListenAndServe(":80", nil)

With all of this done, our example request above should result in a response like this:

	{
	    "jsonrpc": "2.0",
	    "result": 4
	}
