package jwt

import (
	"context"
	"fmt"
	"net/http"
	"testing"

	"google.golang.org/grpc/metadata"
)

func TestHTTPToContext(t *testing.T) {
	reqFunc := HTTPToContext()

	// When the header doesn't exist
	ctx := reqFunc(context.Background(), &http.Request{})

	if ctx.Value(JWTContextKey) != nil {
		t.Error("Context shouldn't contain the encoded JWT")
	}

	// Authorization header value has invalid format
	header := http.Header{}
	header.Set("Authorization", "no expected auth header format value")
	ctx = reqFunc(context.Background(), &http.Request{Header: header})

	if ctx.Value(JWTContextKey) != nil {
		t.Error("Context shouldn't contain the encoded JWT")
	}

	// Authorization header is correct
	header.Set("Authorization", generateAuthHeaderFromToken(signedKey))
	ctx = reqFunc(context.Background(), &http.Request{Header: header})

	token := ctx.Value(JWTContextKey).(string)
	if token != signedKey {
		t.Errorf("Context doesn't contain the expected encoded token value; expected: %s, got: %s", signedKey, token)
	}
}

func TestContextToHTTP(t *testing.T) {
	reqFunc := ContextToHTTP()

	// No JWT is passed in the context
	ctx := context.Background()
	r := http.Request{}
	reqFunc(ctx, &r)

	token := r.Header.Get("Authorization")
	if token != "" {
		t.Error("authorization key should not exist in metadata")
	}

	// Correct JWT is passed in the context
	ctx = context.WithValue(context.Background(), JWTContextKey, signedKey)
	r = http.Request{Header: http.Header{}}
	reqFunc(ctx, &r)

	token = r.Header.Get("Authorization")
	expected := generateAuthHeaderFromToken(signedKey)

	if token != expected {
		t.Errorf("Authorization header does not contain the expected JWT; expected %s, got %s", expected, token)
	}
}

func TestGRPCToContext(t *testing.T) {
	md := metadata.MD{}
	reqFunc := GRPCToContext()

	// No Authorization header is passed
	ctx := reqFunc(context.Background(), md)
	token := ctx.Value(JWTContextKey)
	if token != nil {
		t.Error("Context should not contain a JWT")
	}

	// Invalid Authorization header is passed
	md["authorization"] = []string{signedKey}
	ctx = reqFunc(context.Background(), md)
	token = ctx.Value(JWTContextKey)
	if token != nil {
		t.Error("Context should not contain a JWT")
	}

	// Authorization header is correct
	md["authorization"] = []string{fmt.Sprintf("Bearer %s", signedKey)}
	ctx = reqFunc(context.Background(), md)
	token, ok := ctx.Value(JWTContextKey).(string)
	if !ok {
		t.Fatal("JWT not passed to context correctly")
	}

	if token != signedKey {
		t.Errorf("JWTs did not match: expecting %s got %s", signedKey, token)
	}
}

func TestContextToGRPC(t *testing.T) {
	reqFunc := ContextToGRPC()

	// No JWT is passed in the context
	ctx := context.Background()
	md := metadata.MD{}
	reqFunc(ctx, &md)

	_, ok := md["authorization"]
	if ok {
		t.Error("authorization key should not exist in metadata")
	}

	// Correct JWT is passed in the context
	ctx = context.WithValue(context.Background(), JWTContextKey, signedKey)
	md = metadata.MD{}
	reqFunc(ctx, &md)

	token, ok := md["authorization"]
	if !ok {
		t.Fatal("JWT not passed to metadata correctly")
	}

	if token[0] != generateAuthHeaderFromToken(signedKey) {
		t.Errorf("JWTs did not match: expecting %s got %s", signedKey, token[0])
	}
}
