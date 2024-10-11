package keeper_test

import (
	"cosmossdk.io/math"
	"cosmossdk.io/x/mint/keeper"
	"cosmossdk.io/x/mint/types"
)

func (s *KeeperTestSuite) TestMigrator_Migrate2to3() {
	migrator := keeper.NewMigrator(s.mintKeeper)

	params, err := s.mintKeeper.Params.Get(s.ctx)
	s.NoError(err)

	err = migrator.Migrate2to3(s.ctx)
	s.NoError(err)

	migratedParams, err := s.mintKeeper.Params.Get(s.ctx)
	s.NoError(err)
	s.Equal(params, migratedParams)

	params.MaxSupply = math.NewInt(1000000)
	err = s.mintKeeper.Params.Set(s.ctx, params)
	s.NoError(err)

	err = migrator.Migrate2to3(s.ctx)
	s.NoError(err)

	migratedParams, err = s.mintKeeper.Params.Get(s.ctx)
	s.NoError(err)
	s.NotEqual(params, migratedParams)
	s.Equal(migratedParams, types.DefaultParams())
}
