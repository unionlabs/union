package keeper

import (
	"context"
	"encoding/hex"
	"fmt"
	"strings"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/event"
	"cosmossdk.io/errors"
	"cosmossdk.io/x/evidence/exported"
	"cosmossdk.io/x/evidence/types"

	"github.com/cosmos/cosmos-sdk/codec"
)

// Keeper defines the evidence module's keeper. The keeper is responsible for
// managing persistence, state transitions and query handling for the evidence
// module.
type Keeper struct {
	appmodule.Environment

	cdc             codec.BinaryCodec
	router          types.Router
	stakingKeeper   types.StakingKeeper
	slashingKeeper  types.SlashingKeeper
	consensusKeeper types.ConsensusKeeper
	addressCodec    address.Codec

	Schema collections.Schema
	// Evidences key: evidence hash bytes | value: Evidence
	Evidences collections.Map[[]byte, exported.Evidence]
}

// NewKeeper creates a new Keeper object.
func NewKeeper(
	cdc codec.BinaryCodec, env appmodule.Environment, stakingKeeper types.StakingKeeper,
	slashingKeeper types.SlashingKeeper, ck types.ConsensusKeeper, ac address.Codec,
) *Keeper {
	sb := collections.NewSchemaBuilder(env.KVStoreService)
	k := &Keeper{
		Environment:     env,
		cdc:             cdc,
		stakingKeeper:   stakingKeeper,
		slashingKeeper:  slashingKeeper,
		consensusKeeper: ck,
		addressCodec:    ac,
		Evidences:       collections.NewMap(sb, types.KeyPrefixEvidence, "evidences", collections.BytesKey, codec.CollInterfaceValue[exported.Evidence](cdc)),
	}
	schema, err := sb.Build()
	if err != nil {
		panic(err)
	}
	k.Schema = schema
	return k
}

// SetRouter sets the Evidence Handler router for the x/evidence module. Note,
// we allow the ability to set the router after the Keeper is constructed as a
// given Handler may need access the Keeper before being constructed. The router
// may only be set once and will be sealed if it's not already sealed.
func (k *Keeper) SetRouter(rtr types.Router) {
	// It is vital to seal the Evidence Handler router as to not allow further
	// handlers to be registered after the keeper is created since this
	// could create invalid or non-deterministic behavior.
	if !rtr.Sealed() {
		rtr.Seal()
	}
	if k.router != nil {
		panic(fmt.Sprintf("attempting to reset router on x/%s", types.ModuleName))
	}

	k.router = rtr
}

// GetEvidenceHandler returns a registered Handler for a given Evidence type. If
// no handler exists, an error is returned.
func (k Keeper) GetEvidenceHandler(evidenceRoute string) (types.Handler, error) {
	if !k.router.HasRoute(evidenceRoute) {
		return nil, errors.Wrap(types.ErrNoEvidenceHandlerExists, evidenceRoute)
	}

	return k.router.GetRoute(evidenceRoute), nil
}

// SubmitEvidence attempts to match evidence against the keepers router and execute
// the corresponding registered Evidence Handler. An error is returned if no
// registered Handler exists or if the Handler fails. Otherwise, the evidence is
// persisted.
func (k Keeper) SubmitEvidence(ctx context.Context, evidence exported.Evidence) error {
	if _, err := k.Evidences.Get(ctx, evidence.Hash()); err == nil {
		return errors.Wrap(types.ErrEvidenceExists, strings.ToUpper(hex.EncodeToString(evidence.Hash())))
	}
	if !k.router.HasRoute(evidence.Route()) {
		return errors.Wrap(types.ErrNoEvidenceHandlerExists, evidence.Route())
	}

	handler := k.router.GetRoute(evidence.Route())
	if err := handler(ctx, evidence); err != nil {
		return errors.Wrap(types.ErrInvalidEvidence, err.Error())
	}

	if err := k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeSubmitEvidence,
		event.NewAttribute(types.AttributeKeyEvidenceHash, strings.ToUpper(hex.EncodeToString(evidence.Hash()))),
	); err != nil {
		return err
	}

	return k.Evidences.Set(ctx, evidence.Hash(), evidence)
}
