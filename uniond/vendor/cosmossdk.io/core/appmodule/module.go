package appmodule

import (
	"context"

	appmodulev2 "cosmossdk.io/core/appmodule/v2"
	"cosmossdk.io/core/registry"
)

// AppModule is a tag interface for app module implementations to use as a basis
// for extension interfaces. It provides no functionality itself, but is the
// type that all valid app modules should provide so that they can be identified
// by other modules (usually via depinject) as app modules.
type AppModule = appmodulev2.AppModule

// HasPreBlocker is the extension interface that modules should implement to run
// custom logic before BeginBlock.
type HasPreBlocker = appmodulev2.HasPreBlocker

// HasBeginBlocker is the extension interface that modules should implement to run
// custom logic before transaction processing in a block.
type HasBeginBlocker = appmodulev2.HasBeginBlocker

// HasEndBlocker is the extension interface that modules should implement to run
// custom logic after transaction processing in a block.
type HasEndBlocker = appmodulev2.HasEndBlocker

// HasRegisterInterfaces is the interface for modules to register their msg types.
type HasRegisterInterfaces = appmodulev2.HasRegisterInterfaces

// ValidatorUpdate defines a validator update.
type ValidatorUpdate = appmodulev2.ValidatorUpdate

// HasServices is the extension interface that modules should implement to register
// implementations of services defined in .proto files.
// This API is supported by the Cosmos SDK module managers but is excluded from core to limit dependencies.
// type HasServices interface {
// 	AppModule

// 	// RegisterServices registers the module's services with the app's service
// 	// registrar.
// 	//
// 	// Two types of services are currently supported:
// 	// - read-only gRPC query services, which are the default.
// 	// - transaction message services, which must have the protobuf service
// 	//   option "cosmos.msg.v1.service" (defined in "cosmos/msg/v1/service.proto")
// 	//   set to true.
// 	//
// 	// The service registrar will figure out which type of service you are
// 	// implementing based on the presence (or absence) of protobuf options. You
// 	// do not need to specify this in golang code.
// 	RegisterServices(grpc.ServiceRegistrar) error
// }

// HasPrepareCheckState is an extension interface that contains information about the AppModule
// and PrepareCheckState.
type HasPrepareCheckState interface {
	appmodulev2.AppModule
	PrepareCheckState(context.Context) error
}

// HasPrecommit is an extension interface that contains information about the appmodule.AppModule and Precommit.
type HasPrecommit interface {
	appmodulev2.AppModule
	Precommit(context.Context) error
}

// HasAminoCodec is an extension interface that module must implement to support JSON encoding and decoding of its types
// through amino.  This is used in genesis & the CLI client.
type HasAminoCodec interface {
	RegisterLegacyAminoCodec(registry.AminoRegistrar)
}
