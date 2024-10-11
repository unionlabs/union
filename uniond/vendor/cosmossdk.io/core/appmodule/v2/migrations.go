package appmodulev2

import "context"

// HasConsensusVersion is the interface for declaring a module consensus version.
type HasConsensusVersion interface {
	// ConsensusVersion is a sequence number for state-breaking change of the
	// module. It should be incremented on each consensus-breaking change
	// introduced by the module. To avoid wrong/empty versions, the initial version
	// should be set to 1.
	ConsensusVersion() uint64
}

// HasMigrations is implemented by a module which upgrades or has upgraded to a new consensus version.
type HasMigrations interface {
	AppModule
	HasConsensusVersion

	// RegisterMigrations registers the module's migrations with the app's migrator.
	RegisterMigrations(MigrationRegistrar) error
}

// MigrationRegistrar is the interface for registering in-place store migrations.
type MigrationRegistrar interface {
	// Register registers an in-place store migration for a module. The
	// handler is a migration script to perform in-place migrations from version
	// `fromVersion` to version `fromVersion+1`.
	//
	// EACH TIME a module's ConsensusVersion increments, a new migration MUST
	// be registered using this function. If a migration handler is missing for
	// a particular function, the upgrade logic (see RunMigrations function)
	// will panic. If the ConsensusVersion bump does not introduce any store
	// changes, then a no-op function must be registered here.
	Register(moduleName string, fromVersion uint64, handler MigrationHandler) error
}

// MigrationHandler is the migration function that each module registers.
type MigrationHandler func(context.Context) error

// VersionMap is a map of moduleName -> version
type VersionMap map[string]uint64
