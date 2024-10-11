package test

import "context"

type Service interface {
	Test(ctx context.Context, a string, b int64) (context.Context, string, error)
}

type TestRequest struct {
	A string
	B int64
}

type TestResponse struct {
	Ctx context.Context
	V   string
}
