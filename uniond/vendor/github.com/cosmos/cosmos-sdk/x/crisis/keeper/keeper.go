package keeper

import (
	"context"
	"fmt"
	"time"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/address"
	storetypes "cosmossdk.io/core/store"
	"cosmossdk.io/log"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/crisis/types"
)

// Keeper - crisis keeper
type Keeper struct {
	routes         []types.InvarRoute
	invCheckPeriod uint
	storeService   storetypes.KVStoreService
	cdc            codec.BinaryCodec

	// the address capable of executing a MsgUpdateParams message. Typically, this
	// should be the x/gov module account.
	authority string

	supplyKeeper types.SupplyKeeper

	feeCollectorName string // name of the FeeCollector ModuleAccount

	addressCodec address.Codec

	Schema      collections.Schema
	ConstantFee collections.Item[sdk.Coin]
}

// NewKeeper creates a new Keeper object
func NewKeeper(
	cdc codec.BinaryCodec, storeService storetypes.KVStoreService, invCheckPeriod uint,
	supplyKeeper types.SupplyKeeper, feeCollectorName, authority string, ac address.Codec,
) *Keeper {
	sb := collections.NewSchemaBuilder(storeService)
	k := &Keeper{
		storeService:     storeService,
		cdc:              cdc,
		routes:           make([]types.InvarRoute, 0),
		invCheckPeriod:   invCheckPeriod,
		supplyKeeper:     supplyKeeper,
		feeCollectorName: feeCollectorName,
		authority:        authority,
		addressCodec:     ac,

		ConstantFee: collections.NewItem(sb, types.ConstantFeeKey, "constant_fee", codec.CollValue[sdk.Coin](cdc)),
	}
	schema, err := sb.Build()
	if err != nil {
		panic(err)
	}
	k.Schema = schema
	return k
}

// GetAuthority returns the x/crisis module's authority.
func (k *Keeper) GetAuthority() string {
	return k.authority
}

// Logger returns a module-specific logger.
func (k *Keeper) Logger(ctx context.Context) log.Logger {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	return sdkCtx.Logger().With("module", "x/"+types.ModuleName)
}

// RegisterRoute register the routes for each of the invariants
func (k *Keeper) RegisterRoute(moduleName, route string, invar sdk.Invariant) {
	invarRoute := types.NewInvarRoute(moduleName, route, invar)
	k.routes = append(k.routes, invarRoute)
}

// Routes - return the keeper's invariant routes
func (k *Keeper) Routes() []types.InvarRoute {
	return k.routes
}

// Invariants returns a copy of all registered Crisis keeper invariants.
func (k *Keeper) Invariants() []sdk.Invariant {
	invars := make([]sdk.Invariant, len(k.routes))
	for i, route := range k.routes {
		invars[i] = route.Invar
	}
	return invars
}

// AssertInvariants asserts all registered invariants. If any invariant fails,
// the method panics.
func (k *Keeper) AssertInvariants(ctx sdk.Context) {
	logger := k.Logger(ctx)

	start := time.Now()
	invarRoutes := k.Routes()
	n := len(invarRoutes)
	for i, ir := range invarRoutes {
		logger.Info("asserting crisis invariants", "inv", fmt.Sprint(i+1, "/", n), "name", ir.FullRoute())

		invCtx, _ := ctx.CacheContext()
		if res, stop := ir.Invar(invCtx); stop {
			// TODO: Include app name as part of context to allow for this to be
			// variable.
			panic(fmt.Errorf("invariant broken: %s\n"+
				"\tCRITICAL please submit the following transaction:\n"+
				"\t\t tx crisis invariant-broken %s %s", res, ir.ModuleName, ir.Route))
		}
	}

	diff := time.Since(start)
	logger.Info("asserted all invariants", "duration", diff, "height", ctx.BlockHeight())
}

// InvCheckPeriod returns the invariant checks period.
func (k *Keeper) InvCheckPeriod() uint { return k.invCheckPeriod }

// SendCoinsFromAccountToFeeCollector transfers amt to the fee collector account.
func (k *Keeper) SendCoinsFromAccountToFeeCollector(ctx context.Context, senderAddr sdk.AccAddress, amt sdk.Coins) error {
	return k.supplyKeeper.SendCoinsFromAccountToModule(ctx, senderAddr, k.feeCollectorName, amt)
}
