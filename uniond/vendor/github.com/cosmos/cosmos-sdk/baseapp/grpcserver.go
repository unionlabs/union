package baseapp

import (
	"context"
	"strconv"

	gogogrpc "github.com/cosmos/gogoproto/grpc"
	grpcmiddleware "github.com/grpc-ecosystem/go-grpc-middleware"
	grpcrecovery "github.com/grpc-ecosystem/go-grpc-middleware/recovery"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	grpctypes "github.com/cosmos/cosmos-sdk/types/grpc"
)

// GRPCQueryRouter returns the GRPCQueryRouter of a BaseApp.
func (app *BaseApp) GRPCQueryRouter() *GRPCQueryRouter { return app.grpcQueryRouter }

// RegisterGRPCServer registers gRPC services directly with the gRPC server.
func (app *BaseApp) RegisterGRPCServer(server gogogrpc.Server) {
	// Define an interceptor for all gRPC queries: this interceptor will create
	// a new sdk.Context, and pass it into the query handler.
	interceptor := func(grpcCtx context.Context, req interface{}, _ *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (resp interface{}, err error) {
		// If there's some metadata in the context, retrieve it.
		md, ok := metadata.FromIncomingContext(grpcCtx)
		if !ok {
			return nil, status.Error(codes.Internal, "unable to retrieve metadata")
		}

		// Get height header from the request context, if present.
		var height int64
		if heightHeaders := md.Get(grpctypes.GRPCBlockHeightHeader); len(heightHeaders) == 1 {
			height, err = strconv.ParseInt(heightHeaders[0], 10, 64)
			if err != nil {
				return nil, errorsmod.Wrapf(
					sdkerrors.ErrInvalidRequest,
					"Baseapp.RegisterGRPCServer: invalid height header %q: %v", grpctypes.GRPCBlockHeightHeader, err)
			}
			if err := checkNegativeHeight(height); err != nil {
				return nil, err
			}
		}

		// Create the sdk.Context. Passing false as 2nd arg, as we can't
		// actually support proofs with gRPC right now.
		sdkCtx, err := app.CreateQueryContext(height, false)
		if err != nil {
			return nil, err
		}

		// Add relevant gRPC headers
		if height == 0 {
			height = sdkCtx.BlockHeight() // If height was not set in the request, set it to the latest
		}

		// Attach the sdk.Context into the gRPC's context.Context.
		grpcCtx = context.WithValue(grpcCtx, sdk.SdkContextKey, sdkCtx)

		md = metadata.Pairs(grpctypes.GRPCBlockHeightHeader, strconv.FormatInt(height, 10))
		if err = grpc.SetHeader(grpcCtx, md); err != nil {
			app.logger.Error("failed to set gRPC header", "err", err)
		}

		return handler(grpcCtx, req)
	}

	// Loop through all services and methods, add the interceptor, and register
	// the service.
	for _, data := range app.GRPCQueryRouter().serviceData {
		desc := data.serviceDesc
		newMethods := make([]grpc.MethodDesc, len(desc.Methods))

		for i, method := range desc.Methods {
			methodHandler := method.Handler
			newMethods[i] = grpc.MethodDesc{
				MethodName: method.MethodName,
				Handler: func(srv interface{}, ctx context.Context, dec func(interface{}) error, _ grpc.UnaryServerInterceptor) (interface{}, error) {
					return methodHandler(srv, ctx, dec, grpcmiddleware.ChainUnaryServer(
						grpcrecovery.UnaryServerInterceptor(),
						interceptor,
					))
				},
			}
		}

		newDesc := &grpc.ServiceDesc{
			ServiceName: desc.ServiceName,
			HandlerType: desc.HandlerType,
			Methods:     newMethods,
			Streams:     desc.Streams,
			Metadata:    desc.Metadata,
		}

		server.RegisterService(newDesc, data.handler)
	}
}
