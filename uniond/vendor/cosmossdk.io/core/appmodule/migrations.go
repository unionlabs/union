package appmodule

import (
	appmodulev2 "cosmossdk.io/core/appmodule/v2"
)

// HasConsensusVersion is the interface for declaring a module consensus version.
type HasConsensusVersion = appmodulev2.HasConsensusVersion

// HasMigrations is implemented by a module which upgrades or has upgraded to a new consensus version.
type HasMigrations = appmodulev2.HasMigrations

// MigrationRegistrar is the interface for registering in-place store migrations.
type MigrationRegistrar = appmodulev2.MigrationRegistrar

// MigrationHandler is the migration function that each module registers.
type MigrationHandler = appmodulev2.MigrationHandler

// VersionMap is a map of moduleName -> version
type VersionMap = appmodulev2.VersionMap
