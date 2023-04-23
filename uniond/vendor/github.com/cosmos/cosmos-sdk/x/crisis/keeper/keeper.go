package keeper

import (
	"fmt"
	"time"

	"github.com/cometbft/cometbft/libs/log"

	"github.com/cosmos/cosmos-sdk/codec"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/crisis/types"
)

// Keeper - crisis keeper
type Keeper struct {
	routes         []types.InvarRoute
	invCheckPeriod uint
	storeKey       storetypes.StoreKey
	cdc            codec.BinaryCodec

	// the address capable of executing a MsgUpdateParams message. Typically, this
	// should be the x/gov module account.
	authority string

	supplyKeeper types.SupplyKeeper

	feeCollectorName string // name of the FeeCollector ModuleAccount
}

// NewKeeper creates a new Keeper object
func NewKeeper(
	cdc codec.BinaryCodec, storeKey storetypes.StoreKey, invCheckPeriod uint,
	supplyKeeper types.SupplyKeeper, feeCollectorName string, authority string,
) *Keeper {
	return &Keeper{
		storeKey:         storeKey,
		cdc:              cdc,
		routes:           make([]types.InvarRoute, 0),
		invCheckPeriod:   invCheckPeriod,
		supplyKeeper:     supplyKeeper,
		feeCollectorName: feeCollectorName,
		authority:        authority,
	}
}

// GetAuthority returns the x/crisis module's authority.
func (k *Keeper) GetAuthority() string {
	return k.authority
}

// Logger returns a module-specific logger.
func (k *Keeper) Logger(ctx sdk.Context) log.Logger {
	return ctx.Logger().With("module", "x/"+types.ModuleName)
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
		if res, stop := ir.Invar(ctx); stop {
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
func (k *Keeper) SendCoinsFromAccountToFeeCollector(ctx sdk.Context, senderAddr sdk.AccAddress, amt sdk.Coins) error {
	return k.supplyKeeper.SendCoinsFromAccountToModule(ctx, senderAddr, k.feeCollectorName, amt)
}
