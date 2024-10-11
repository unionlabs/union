#!/usr/bin/env bash

mockgen_cmd="mockgen"
$mockgen_cmd -source=baseapp/abci_utils.go -package mock -destination baseapp/testutil/mock/mocks.go
$mockgen_cmd -source=client/account_retriever.go -package mock -destination testutil/mock/account_retriever.go
$mockgen_cmd -source=types/module/module.go -package mock -destination testutil/mock/types_module_module.go
$mockgen_cmd -source=types/module/mock_appmodule_test.go -package mock -destination testutil/mock/types_mock_appmodule.go
$mockgen_cmd -source=types/invariant.go -package mock -destination testutil/mock/types_invariant.go
$mockgen_cmd -package mock -destination testutil/mock/grpc_server.go github.com/cosmos/gogoproto/grpc Server
$mockgen_cmd -package mock -destination testutil/mock/logger.go cosmossdk.io/log Logger
#$mockgen_cmd -source=orm/model/ormtable/hooks.go -package ormmocks -destination orm/testing/ormmocks/hooks.go
$mockgen_cmd -source=x/nft/expected_keepers.go -package testutil -destination x/nft/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/feegrant/expected_keepers.go -package testutil -destination x/feegrant/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/mint/types/expected_keepers.go -package testutil -destination x/mint/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/auth/tx/config/expected_keepers.go -package testutil -destination x/auth/tx/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/auth/types/expected_keepers.go -package testutil -destination x/auth/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/auth/ante/expected_keepers.go -package testutil -destination x/auth/ante/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/authz/expected_keepers.go -package testutil -destination x/authz/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/bank/types/expected_keepers.go -package testutil -destination x/bank/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/group/testutil/expected_keepers.go -package testutil -destination x/group/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/evidence/types/expected_keepers.go -package testutil -destination x/evidence/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/distribution/types/expected_keepers.go -package testutil -destination x/distribution/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/slashing/types/expected_keepers.go -package testutil -destination x/slashing/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/genutil/types/expected_keepers.go -package testutil -destination x/genutil/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/gov/testutil/expected_keepers.go -package testutil -destination x/gov/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/staking/types/expected_keepers.go -package testutil -destination x/staking/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/auth/vesting/types/expected_keepers.go -package testutil -destination x/auth/vesting/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/protocolpool/types/expected_keepers.go -package testutil -destination x/protocolpool/testutil/expected_keepers_mocks.go
$mockgen_cmd -source=x/upgrade/types/expected_keepers.go -package testutil -destination x/upgrade/testutil/expected_keepers_mocks.go
