package v1

import (
	"slices"
	"strings"

	"google.golang.org/grpc"

	"cosmossdk.io/x/accounts/internal/implementation"
)

func MakeAccountsSchemas(impls map[string]implementation.Implementation) map[string]*SchemaResponse {
	m := make(map[string]*SchemaResponse, len(impls))
	for name, impl := range impls {
		m[name] = makeAccountSchema(impl)
	}
	return m
}

func makeAccountSchema(impl implementation.Implementation) *SchemaResponse {
	return &SchemaResponse{
		InitSchema: &SchemaResponse_Handler{
			Request:  impl.InitHandlerSchema.RequestSchema.Name,
			Response: impl.InitHandlerSchema.ResponseSchema.Name,
		},
		ExecuteHandlers: makeHandlersSchema(impl.ExecuteHandlersSchema),
		QueryHandlers:   makeHandlersSchema(impl.QueryHandlersSchema),
	}
}

func makeHandlersSchema(handlers map[string]implementation.HandlerSchema) []*SchemaResponse_Handler {
	schemas := make([]*SchemaResponse_Handler, 0, len(handlers))
	for name, handler := range handlers {
		schemas = append(schemas, &SchemaResponse_Handler{
			Request:  name,
			Response: handler.ResponseSchema.Name,
		})
	}
	slices.SortFunc(schemas, func(a, b *SchemaResponse_Handler) int {
		return strings.Compare(a.Request, b.Request)
	})
	return schemas
}

func MsgServiceDesc() *grpc.ServiceDesc {
	return &_Msg_serviceDesc
}
