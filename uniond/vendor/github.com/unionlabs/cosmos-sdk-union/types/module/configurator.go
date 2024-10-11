package module

import (
	"fmt"

	"github.com/cosmos/gogoproto/grpc"
	googlegrpc "google.golang.org/grpc"
	protobuf "google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"

	cosmosmsg "cosmossdk.io/api/cosmos/msg/v1"
	"cosmossdk.io/core/appmodule"
	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// Deprecated: The Configurator is deprecated.
// Preferably use core services for registering msg/query server and migrations.
type Configurator interface {
	grpc.Server

	// Error returns the last error encountered during RegisterService.
	Error() error

	// MsgServer returns a grpc.Server instance which allows registering services
	// that will handle TxBody.messages in transactions. These Msg's WILL NOT
	// be exposed as gRPC services.
	MsgServer() grpc.Server

	// QueryServer returns a grpc.Server instance which allows registering services
	// that will be exposed as gRPC services as well as ABCI query handlers.
	QueryServer() grpc.Server

	// RegisterMigration registers an in-place store migration for a module. The
	// handler is a migration script to perform in-place migrations from version
	// `fromVersion` to version `fromVersion+1`.
	//
	// EACH TIME a module's ConsensusVersion increments, a new migration MUST
	// be registered using this function. If a migration handler is missing for
	// a particular function, the upgrade logic (see RunMigrations function)
	// will panic. If the ConsensusVersion bump does not introduce any store
	// changes, then a no-op function must be registered here.
	RegisterMigration(moduleName string, fromVersion uint64, handler MigrationHandler) error

	// Register registers an in-place store migration for a module.
	// It permits to register modules migrations that have migrated to serverv2 but still be compatible with baseapp.
	Register(moduleName string, fromVersion uint64, handler appmodule.MigrationHandler) error
}

type configurator struct {
	cdc         codec.Codec
	msgServer   grpc.Server
	queryServer grpc.Server

	// migrations is a map of moduleName -> fromVersion -> migration script handler
	migrations map[string]map[uint64]MigrationHandler

	err error
}

// RegisterService implements the grpc.Server interface.
func (c *configurator) RegisterService(sd *googlegrpc.ServiceDesc, ss interface{}) {
	desc, err := c.cdc.InterfaceRegistry().FindDescriptorByName(protoreflect.FullName(sd.ServiceName))
	if err != nil {
		c.err = err
		return
	}

	if protobuf.HasExtension(desc.Options(), cosmosmsg.E_Service) {
		c.msgServer.RegisterService(sd, ss)
	} else {
		c.queryServer.RegisterService(sd, ss)
	}
}

// Error returns the last error encountered during RegisterService.
func (c *configurator) Error() error {
	return c.err
}

// NewConfigurator returns a new Configurator instance
func NewConfigurator(cdc codec.Codec, msgServer, queryServer grpc.Server) Configurator {
	return &configurator{
		cdc:         cdc,
		msgServer:   msgServer,
		queryServer: queryServer,
		migrations:  map[string]map[uint64]MigrationHandler{},
	}
}

var _ Configurator = &configurator{}

// MsgServer implements the Configurator.MsgServer method
func (c *configurator) MsgServer() grpc.Server {
	return c.msgServer
}

// QueryServer implements the Configurator.QueryServer method
func (c *configurator) QueryServer() grpc.Server {
	return c.queryServer
}

// RegisterMigration implements the Configurator.RegisterMigration method
func (c *configurator) RegisterMigration(moduleName string, fromVersion uint64, handler MigrationHandler) error {
	if fromVersion == 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidVersion, "module migration versions should start at 1")
	}

	if c.migrations[moduleName] == nil {
		c.migrations[moduleName] = map[uint64]MigrationHandler{}
	}

	if c.migrations[moduleName][fromVersion] != nil {
		return errorsmod.Wrapf(sdkerrors.ErrLogic, "another migration for module %s and version %d already exists", moduleName, fromVersion)
	}

	c.migrations[moduleName][fromVersion] = handler

	return nil
}

// Register implements the Configurator.Register method
// It allows to register modules migrations that have migrated to server/v2 but still be compatible with baseapp.
func (c *configurator) Register(moduleName string, fromVersion uint64, handler appmodule.MigrationHandler) error {
	return c.RegisterMigration(moduleName, fromVersion, func(sdkCtx sdk.Context) error {
		return handler(sdkCtx)
	})
}

// runModuleMigrations runs all in-place store migrations for one given module from a
// version to another version.
func (c *configurator) runModuleMigrations(ctx sdk.Context, moduleName string, fromVersion, toVersion uint64) error {
	// No-op if toVersion is the initial version or if the version is unchanged.
	if toVersion <= 1 || fromVersion == toVersion {
		return nil
	}

	moduleMigrationsMap, found := c.migrations[moduleName]
	if !found {
		return errorsmod.Wrapf(sdkerrors.ErrNotFound, "no migrations found for module %s", moduleName)
	}

	// Run in-place migrations for the module sequentially until toVersion.
	for i := fromVersion; i < toVersion; i++ {
		migrateFn, found := moduleMigrationsMap[i]
		if !found {
			return errorsmod.Wrapf(sdkerrors.ErrNotFound, "no migration found for module %s from version %d to version %d", moduleName, i, i+1)
		}
		ctx.Logger().Info(fmt.Sprintf("migrating module %s from version %d to version %d", moduleName, i, i+1))

		err := migrateFn(ctx)
		if err != nil {
			return err
		}
	}

	return nil
}
