package awslambda

import (
	"context"
	"encoding/json"
	"fmt"
	"testing"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/transport"
	"github.com/go-kit/log"
)

type key int

const (
	KeyBeforeOne key = iota
	KeyBeforeTwo key = iota
	KeyAfterOne  key = iota
	KeyEncMode   key = iota
)

// Created based on github.com/aws/aws-lambda-go@v1.13.3/events.APIGatewayProxyRequest for the purposes of the tests below.
type apiGatewayProxyRequest struct {
	Body string `json:"body"`
}

// Created based on github.com/aws/aws-lambda-go@v1.13.3/events.APIGatewayProxyResponse for the purposes of the tests below.
type apiGatewayProxyResponse struct {
	StatusCode int    `json:"statusCode"`
	Body       string `json:"body"`
}

func TestDefaultErrorEncoder(t *testing.T) {
	ctx := context.Background()
	rootErr := fmt.Errorf("root")
	b, err := DefaultErrorEncoder(ctx, rootErr)
	if b != nil {
		t.Fatalf("DefaultErrorEncoder should return nil as []byte")
	}
	if err != rootErr {
		t.Fatalf("DefaultErrorEncoder expects return back the given error.")
	}
}

func TestInvokeHappyPath(t *testing.T) {
	svc := serviceTest01{}

	helloHandler := NewHandler(
		makeTest01HelloEndpoint(svc),
		decodeHelloRequestWithTwoBefores,
		encodeResponse,
		HandlerErrorHandler(transport.NewLogErrorHandler(log.NewNopLogger())),
		HandlerBefore(func(
			ctx context.Context,
			payload []byte,
		) context.Context {
			ctx = context.WithValue(ctx, KeyBeforeOne, "bef1")
			return ctx
		}),
		HandlerBefore(func(
			ctx context.Context,
			payload []byte,
		) context.Context {
			ctx = context.WithValue(ctx, KeyBeforeTwo, "bef2")
			return ctx
		}),
		HandlerAfter(func(
			ctx context.Context,
			response interface{},
		) context.Context {
			ctx = context.WithValue(ctx, KeyAfterOne, "af1")
			return ctx
		}),
		HandlerAfter(func(
			ctx context.Context,
			response interface{},
		) context.Context {
			if _, ok := ctx.Value(KeyAfterOne).(string); !ok {
				t.Fatalf("Value was not set properly during multi HandlerAfter")
			}
			return ctx
		}),
		HandlerFinalizer(func(
			_ context.Context,
			resp []byte,
			_ error,
		) {
			apigwResp := apiGatewayProxyResponse{}
			err := json.Unmarshal(resp, &apigwResp)
			if err != nil {
				t.Fatalf("Should have no error, but got: %+v", err)
			}

			response := helloResponse{}
			err = json.Unmarshal([]byte(apigwResp.Body), &response)
			if err != nil {
				t.Fatalf("Should have no error, but got: %+v", err)
			}

			expectedGreeting := "hello john doe bef1 bef2"
			if response.Greeting != expectedGreeting {
				t.Fatalf(
					"Expect: %s, Actual: %s", expectedGreeting, response.Greeting)
			}
		}),
	)

	ctx := context.Background()
	req, _ := json.Marshal(apiGatewayProxyRequest{
		Body: `{"name":"john doe"}`,
	})
	resp, err := helloHandler.Invoke(ctx, req)

	if err != nil {
		t.Fatalf("Should have no error, but got: %+v", err)
	}

	apigwResp := apiGatewayProxyResponse{}
	err = json.Unmarshal(resp, &apigwResp)
	if err != nil {
		t.Fatalf("Should have no error, but got: %+v", err)
	}

	response := helloResponse{}
	err = json.Unmarshal([]byte(apigwResp.Body), &response)
	if err != nil {
		t.Fatalf("Should have no error, but got: %+v", err)
	}

	expectedGreeting := "hello john doe bef1 bef2"
	if response.Greeting != expectedGreeting {
		t.Fatalf(
			"Expect: %s, Actual: %s", expectedGreeting, response.Greeting)
	}
}

func TestInvokeFailDecode(t *testing.T) {
	svc := serviceTest01{}

	helloHandler := NewHandler(
		makeTest01HelloEndpoint(svc),
		decodeHelloRequestWithTwoBefores,
		encodeResponse,
		HandlerErrorEncoder(func(
			ctx context.Context,
			err error,
		) ([]byte, error) {
			apigwResp := apiGatewayProxyResponse{}
			apigwResp.Body = `{"error":"yes"}`
			apigwResp.StatusCode = 500
			resp, err := json.Marshal(apigwResp)
			return resp, err
		}),
	)

	ctx := context.Background()
	req, _ := json.Marshal(apiGatewayProxyRequest{
		Body: `{"name":"john doe"}`,
	})
	resp, err := helloHandler.Invoke(ctx, req)

	if err != nil {
		t.Fatalf("Should have no error, but got: %+v", err)
	}

	apigwResp := apiGatewayProxyResponse{}
	json.Unmarshal(resp, &apigwResp)
	if apigwResp.StatusCode != 500 {
		t.Fatalf("Expect status code of 500, instead of %d", apigwResp.StatusCode)
	}
}

func TestInvokeFailEndpoint(t *testing.T) {
	svc := serviceTest01{}

	helloHandler := NewHandler(
		makeTest01FailEndpoint(svc),
		decodeHelloRequestWithTwoBefores,
		encodeResponse,
		HandlerBefore(func(
			ctx context.Context,
			payload []byte,
		) context.Context {
			ctx = context.WithValue(ctx, KeyBeforeOne, "bef1")
			return ctx
		}),
		HandlerBefore(func(
			ctx context.Context,
			payload []byte,
		) context.Context {
			ctx = context.WithValue(ctx, KeyBeforeTwo, "bef2")
			return ctx
		}),
		HandlerErrorEncoder(func(
			ctx context.Context,
			err error,
		) ([]byte, error) {
			apigwResp := apiGatewayProxyResponse{}
			apigwResp.Body = `{"error":"yes"}`
			apigwResp.StatusCode = 500
			resp, err := json.Marshal(apigwResp)
			return resp, err
		}),
	)

	ctx := context.Background()
	req, _ := json.Marshal(apiGatewayProxyRequest{
		Body: `{"name":"john doe"}`,
	})
	resp, err := helloHandler.Invoke(ctx, req)

	if err != nil {
		t.Fatalf("Should have no error, but got: %+v", err)
	}

	apigwResp := apiGatewayProxyResponse{}
	json.Unmarshal(resp, &apigwResp)
	if apigwResp.StatusCode != 500 {
		t.Fatalf("Expect status code of 500, instead of %d", apigwResp.StatusCode)
	}
}

func TestInvokeFailEncode(t *testing.T) {
	svc := serviceTest01{}

	helloHandler := NewHandler(
		makeTest01HelloEndpoint(svc),
		decodeHelloRequestWithTwoBefores,
		encodeResponse,
		HandlerBefore(func(
			ctx context.Context,
			payload []byte,
		) context.Context {
			ctx = context.WithValue(ctx, KeyBeforeOne, "bef1")
			return ctx
		}),
		HandlerBefore(func(
			ctx context.Context,
			payload []byte,
		) context.Context {
			ctx = context.WithValue(ctx, KeyBeforeTwo, "bef2")
			return ctx
		}),
		HandlerAfter(func(
			ctx context.Context,
			response interface{},
		) context.Context {
			ctx = context.WithValue(ctx, KeyEncMode, "fail_encode")
			return ctx
		}),
		HandlerErrorEncoder(func(
			ctx context.Context,
			err error,
		) ([]byte, error) {
			// convert error into proper APIGateway response.
			apigwResp := apiGatewayProxyResponse{}
			apigwResp.Body = `{"error":"yes"}`
			apigwResp.StatusCode = 500
			resp, err := json.Marshal(apigwResp)
			return resp, err
		}),
	)

	ctx := context.Background()
	req, _ := json.Marshal(apiGatewayProxyRequest{
		Body: `{"name":"john doe"}`,
	})
	resp, err := helloHandler.Invoke(ctx, req)

	if err != nil {
		t.Fatalf("Should have no error, but got: %+v", err)
	}

	apigwResp := apiGatewayProxyResponse{}
	json.Unmarshal(resp, &apigwResp)
	if apigwResp.StatusCode != 500 {
		t.Fatalf("Expect status code of 500, instead of %d", apigwResp.StatusCode)
	}
}

func decodeHelloRequestWithTwoBefores(
	ctx context.Context, req []byte,
) (interface{}, error) {
	apigwReq := apiGatewayProxyRequest{}
	err := json.Unmarshal([]byte(req), &apigwReq)
	if err != nil {
		return apigwReq, err
	}

	request := helloRequest{}
	err = json.Unmarshal([]byte(apigwReq.Body), &request)
	if err != nil {
		return request, err
	}

	valOne, ok := ctx.Value(KeyBeforeOne).(string)
	if !ok {
		return request, fmt.Errorf(
			"Value was not set properly when multiple HandlerBefores are used")
	}

	valTwo, ok := ctx.Value(KeyBeforeTwo).(string)
	if !ok {
		return request, fmt.Errorf(
			"Value was not set properly when multiple HandlerBefores are used")
	}

	request.Name += " " + valOne + " " + valTwo
	return request, err
}

func encodeResponse(
	ctx context.Context, response interface{},
) ([]byte, error) {
	apigwResp := apiGatewayProxyResponse{}

	mode, ok := ctx.Value(KeyEncMode).(string)
	if ok && mode == "fail_encode" {
		return nil, fmt.Errorf("fail encoding")
	}

	respByte, err := json.Marshal(response)
	if err != nil {
		return nil, err
	}

	apigwResp.Body = string(respByte)
	apigwResp.StatusCode = 200

	resp, err := json.Marshal(apigwResp)
	return resp, err
}

type helloRequest struct {
	Name string `json:"name"`
}

type helloResponse struct {
	Greeting string `json:"greeting"`
}

func makeTest01HelloEndpoint(svc serviceTest01) endpoint.Endpoint {
	return func(_ context.Context, request interface{}) (interface{}, error) {
		req := request.(helloRequest)
		greeting := svc.hello(req.Name)
		return helloResponse{greeting}, nil
	}
}

func makeTest01FailEndpoint(_ serviceTest01) endpoint.Endpoint {
	return func(_ context.Context, request interface{}) (interface{}, error) {
		return nil, fmt.Errorf("test error endpoint")
	}
}

type serviceTest01 struct{}

func (ts *serviceTest01) hello(name string) string {
	return fmt.Sprintf("hello %s", name)
}
