package keeper

import (
	"fmt"
	"strconv"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cosmos/gogoproto/proto"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/authz"
)

// TODO: Revisit this once we have propoer gas fee framework.
// Tracking issues https://github.com/cosmos/cosmos-sdk/issues/9054,
// https://github.com/cosmos/cosmos-sdk/discussions/9072
const gasCostPerIteration = uint64(20)

type Keeper struct {
	storeKey   storetypes.StoreKey
	cdc        codec.BinaryCodec
	router     *baseapp.MsgServiceRouter
	authKeeper authz.AccountKeeper
}

// NewKeeper constructs a message authorization Keeper
func NewKeeper(storeKey storetypes.StoreKey, cdc codec.BinaryCodec, router *baseapp.MsgServiceRouter, ak authz.AccountKeeper) Keeper {
	return Keeper{
		storeKey:   storeKey,
		cdc:        cdc,
		router:     router,
		authKeeper: ak,
	}
}

// Logger returns a module-specific logger.
func (k Keeper) Logger(ctx sdk.Context) log.Logger {
	return ctx.Logger().With("module", fmt.Sprintf("x/%s", authz.ModuleName))
}

// getGrant returns grant stored at skey.
func (k Keeper) getGrant(ctx sdk.Context, skey []byte) (grant authz.Grant, found bool) {
	store := ctx.KVStore(k.storeKey)
	bz := store.Get(skey)
	if bz == nil {
		return grant, false
	}
	k.cdc.MustUnmarshal(bz, &grant)
	return grant, true
}

func (k Keeper) update(ctx sdk.Context, grantee sdk.AccAddress, granter sdk.AccAddress, updated authz.Authorization) error {
	skey := grantStoreKey(grantee, granter, updated.MsgTypeURL())
	grant, found := k.getGrant(ctx, skey)
	if !found {
		return authz.ErrNoAuthorizationFound
	}

	msg, ok := updated.(proto.Message)
	if !ok {
		return sdkerrors.ErrPackAny.Wrapf("cannot proto marshal %T", updated)
	}

	any, err := codectypes.NewAnyWithValue(msg)
	if err != nil {
		return err
	}

	grant.Authorization = any
	store := ctx.KVStore(k.storeKey)
	store.Set(skey, k.cdc.MustMarshal(&grant))

	return nil
}

// DispatchActions attempts to execute the provided messages via authorization
// grants from the message signer to the grantee.
func (k Keeper) DispatchActions(ctx sdk.Context, grantee sdk.AccAddress, msgs []sdk.Msg) ([][]byte, error) {
	results := make([][]byte, len(msgs))
	now := ctx.BlockTime()

	for i, msg := range msgs {
		signers := msg.GetSigners()
		if len(signers) != 1 {
			return nil, authz.ErrAuthorizationNumOfSigners
		}

		granter := signers[0]

		// If granter != grantee then check authorization.Accept, otherwise we
		// implicitly accept.
		if !granter.Equals(grantee) {
			skey := grantStoreKey(grantee, granter, sdk.MsgTypeURL(msg))

			grant, found := k.getGrant(ctx, skey)
			if !found {
				return nil, sdkerrors.Wrapf(authz.ErrNoAuthorizationFound, "failed to update grant with key %s", string(skey))
			}

			if grant.Expiration != nil && grant.Expiration.Before(now) {
				return nil, authz.ErrAuthorizationExpired
			}

			authorization, err := grant.GetAuthorization()
			if err != nil {
				return nil, err
			}

			resp, err := authorization.Accept(ctx, msg)
			if err != nil {
				return nil, err
			}

			if resp.Delete {
				err = k.DeleteGrant(ctx, grantee, granter, sdk.MsgTypeURL(msg))
			} else if resp.Updated != nil {
				err = k.update(ctx, grantee, granter, resp.Updated)
			}
			if err != nil {
				return nil, err
			}

			if !resp.Accept {
				return nil, sdkerrors.ErrUnauthorized
			}
		}

		handler := k.router.Handler(msg)
		if handler == nil {
			return nil, sdkerrors.ErrUnknownRequest.Wrapf("unrecognized message route: %s", sdk.MsgTypeURL(msg))
		}

		msgResp, err := handler(ctx, msg)
		if err != nil {
			return nil, sdkerrors.Wrapf(err, "failed to execute message; message %v", msg)
		}

		results[i] = msgResp.Data

		// emit the events from the dispatched actions
		events := msgResp.Events
		sdkEvents := make([]sdk.Event, 0, len(events))
		for _, event := range events {
			e := event
			e.Attributes = append(e.Attributes, abci.EventAttribute{Key: "authz_msg_index", Value: strconv.Itoa(i)})

			sdkEvents = append(sdkEvents, sdk.Event(e))
		}

		ctx.EventManager().EmitEvents(sdkEvents)
	}

	return results, nil
}

// SaveGrant method grants the provided authorization to the grantee on the granter's account
// with the provided expiration time and insert authorization key into the grants queue. If there is an existing authorization grant for the
// same `sdk.Msg` type, this grant overwrites that.
func (k Keeper) SaveGrant(ctx sdk.Context, grantee, granter sdk.AccAddress, authorization authz.Authorization, expiration *time.Time) error {
	store := ctx.KVStore(k.storeKey)
	msgType := authorization.MsgTypeURL()
	skey := grantStoreKey(grantee, granter, msgType)

	grant, err := authz.NewGrant(ctx.BlockTime(), authorization, expiration)
	if err != nil {
		return err
	}

	var oldExp *time.Time
	if oldGrant, found := k.getGrant(ctx, skey); found {
		oldExp = oldGrant.Expiration
	}

	if oldExp != nil && (expiration == nil || !oldExp.Equal(*expiration)) {
		if err = k.removeFromGrantQueue(ctx, skey, granter, grantee, *oldExp); err != nil {
			return err
		}
	}

	// If the expiration didn't change, then we don't remove it and we should not insert again
	if expiration != nil && (oldExp == nil || !oldExp.Equal(*expiration)) {
		if err = k.insertIntoGrantQueue(ctx, granter, grantee, msgType, *expiration); err != nil {
			return err
		}
	}

	bz := k.cdc.MustMarshal(&grant)
	store.Set(skey, bz)

	return ctx.EventManager().EmitTypedEvent(&authz.EventGrant{
		MsgTypeUrl: authorization.MsgTypeURL(),
		Granter:    granter.String(),
		Grantee:    grantee.String(),
	})
}

// DeleteGrant revokes any authorization for the provided message type granted to the grantee
// by the granter.
func (k Keeper) DeleteGrant(ctx sdk.Context, grantee sdk.AccAddress, granter sdk.AccAddress, msgType string) error {
	store := ctx.KVStore(k.storeKey)
	skey := grantStoreKey(grantee, granter, msgType)
	grant, found := k.getGrant(ctx, skey)
	if !found {
		return sdkerrors.Wrapf(authz.ErrNoAuthorizationFound, "failed to delete grant with key %s", string(skey))
	}

	if grant.Expiration != nil {
		err := k.removeFromGrantQueue(ctx, skey, granter, grantee, *grant.Expiration)
		if err != nil {
			return err
		}
	}

	store.Delete(skey)

	return ctx.EventManager().EmitTypedEvent(&authz.EventRevoke{
		MsgTypeUrl: msgType,
		Granter:    granter.String(),
		Grantee:    grantee.String(),
	})
}

// GetAuthorizations Returns list of `Authorizations` granted to the grantee by the granter.
func (k Keeper) GetAuthorizations(ctx sdk.Context, grantee sdk.AccAddress, granter sdk.AccAddress) ([]authz.Authorization, error) {
	store := ctx.KVStore(k.storeKey)
	key := grantStoreKey(grantee, granter, "")
	iter := sdk.KVStorePrefixIterator(store, key)
	defer iter.Close()

	var authorization authz.Grant
	var authorizations []authz.Authorization
	for ; iter.Valid(); iter.Next() {
		if err := k.cdc.Unmarshal(iter.Value(), &authorization); err != nil {
			return nil, err
		}

		a, err := authorization.GetAuthorization()
		if err != nil {
			return nil, err
		}

		authorizations = append(authorizations, a)
	}

	return authorizations, nil
}

// GetAuthorization returns an Authorization and it's expiration time.
// A nil Authorization is returned under the following circumstances:
//   - No grant is found.
//   - A grant is found, but it is expired.
//   - There was an error getting the authorization from the grant.
func (k Keeper) GetAuthorization(ctx sdk.Context, grantee sdk.AccAddress, granter sdk.AccAddress, msgType string) (authz.Authorization, *time.Time) {
	grant, found := k.getGrant(ctx, grantStoreKey(grantee, granter, msgType))
	if !found || (grant.Expiration != nil && grant.Expiration.Before(ctx.BlockHeader().Time)) {
		return nil, nil
	}

	auth, err := grant.GetAuthorization()
	if err != nil {
		return nil, nil
	}

	return auth, grant.Expiration
}

// IterateGrants iterates over all authorization grants
// This function should be used with caution because it can involve significant IO operations.
// It should not be used in query or msg services without charging additional gas.
// The iteration stops when the handler function returns true or the iterator exhaust.
func (k Keeper) IterateGrants(ctx sdk.Context,
	handler func(granterAddr sdk.AccAddress, granteeAddr sdk.AccAddress, grant authz.Grant) bool,
) {
	store := ctx.KVStore(k.storeKey)
	iter := sdk.KVStorePrefixIterator(store, GrantKey)
	defer iter.Close()
	for ; iter.Valid(); iter.Next() {
		var grant authz.Grant
		granterAddr, granteeAddr, _ := parseGrantStoreKey(iter.Key())
		k.cdc.MustUnmarshal(iter.Value(), &grant)
		if handler(granterAddr, granteeAddr, grant) {
			break
		}
	}
}

func (k Keeper) getGrantQueueItem(ctx sdk.Context, expiration time.Time, granter, grantee sdk.AccAddress) (*authz.GrantQueueItem, error) {
	store := ctx.KVStore(k.storeKey)
	bz := store.Get(GrantQueueKey(expiration, granter, grantee))
	if bz == nil {
		return &authz.GrantQueueItem{}, nil
	}

	var queueItems authz.GrantQueueItem
	if err := k.cdc.Unmarshal(bz, &queueItems); err != nil {
		return nil, err
	}
	return &queueItems, nil
}

func (k Keeper) setGrantQueueItem(ctx sdk.Context, expiration time.Time,
	granter sdk.AccAddress, grantee sdk.AccAddress, queueItems *authz.GrantQueueItem,
) error {
	store := ctx.KVStore(k.storeKey)
	bz, err := k.cdc.Marshal(queueItems)
	if err != nil {
		return err
	}
	store.Set(GrantQueueKey(expiration, granter, grantee), bz)

	return nil
}

// insertIntoGrantQueue inserts a grant key into the grant queue
func (k Keeper) insertIntoGrantQueue(ctx sdk.Context, granter, grantee sdk.AccAddress, msgType string, expiration time.Time) error {
	queueItems, err := k.getGrantQueueItem(ctx, expiration, granter, grantee)
	if err != nil {
		return err
	}

	if len(queueItems.MsgTypeUrls) == 0 {
		k.setGrantQueueItem(ctx, expiration, granter, grantee, &authz.GrantQueueItem{
			MsgTypeUrls: []string{msgType},
		})
	} else {
		queueItems.MsgTypeUrls = append(queueItems.MsgTypeUrls, msgType)
		k.setGrantQueueItem(ctx, expiration, granter, grantee, queueItems)
	}

	return nil
}

// removeFromGrantQueue removes a grant key from the grant queue
func (k Keeper) removeFromGrantQueue(ctx sdk.Context, grantKey []byte, granter, grantee sdk.AccAddress, expiration time.Time) error {
	store := ctx.KVStore(k.storeKey)
	key := GrantQueueKey(expiration, granter, grantee)
	bz := store.Get(key)
	if bz == nil {
		return sdkerrors.Wrap(authz.ErrNoGrantKeyFound, "can't remove grant from the expire queue, grant key not found")
	}

	var queueItem authz.GrantQueueItem
	if err := k.cdc.Unmarshal(bz, &queueItem); err != nil {
		return err
	}

	_, _, msgType := parseGrantStoreKey(grantKey)
	queueItems := queueItem.MsgTypeUrls

	for index, typeURL := range queueItems {
		ctx.GasMeter().ConsumeGas(gasCostPerIteration, "grant queue")

		if typeURL == msgType {
			end := len(queueItem.MsgTypeUrls) - 1
			queueItems[index] = queueItems[end]
			queueItems = queueItems[:end]

			if err := k.setGrantQueueItem(ctx, expiration, granter, grantee, &authz.GrantQueueItem{
				MsgTypeUrls: queueItems,
			}); err != nil {
				return err
			}
			break
		}
	}

	return nil
}

// DequeueAndDeleteExpiredGrants deletes expired grants from the state and grant queue.
func (k Keeper) DequeueAndDeleteExpiredGrants(ctx sdk.Context) error {
	store := ctx.KVStore(k.storeKey)

	iterator := store.Iterator(GrantQueuePrefix, sdk.InclusiveEndBytes(GrantQueueTimePrefix(ctx.BlockTime())))
	defer iterator.Close()

	for ; iterator.Valid(); iterator.Next() {
		var queueItem authz.GrantQueueItem
		if err := k.cdc.Unmarshal(iterator.Value(), &queueItem); err != nil {
			return err
		}

		_, granter, grantee, err := parseGrantQueueKey(iterator.Key())
		if err != nil {
			return err
		}

		store.Delete(iterator.Key())

		for _, typeURL := range queueItem.MsgTypeUrls {
			store.Delete(grantStoreKey(grantee, granter, typeURL))
		}
	}

	return nil
}
