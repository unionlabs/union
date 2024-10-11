package keeper

import (
	"bytes"
	"context"
	"fmt"
	"time"

	gogoproto "github.com/cosmos/gogoproto/proto"
	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	"cosmossdk.io/core/appmodule"
	corecontext "cosmossdk.io/core/context"
	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/authz"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// TODO: Revisit this once we have proper gas fee framework.
// Tracking issues https://github.com/cosmos/cosmos-sdk/issues/9054,
// https://github.com/cosmos/cosmos-sdk/discussions/9072
const gasCostPerIteration = uint64(20)

type Keeper struct {
	appmodule.Environment

	cdc        codec.Codec
	authKeeper authz.AccountKeeper
}

// NewKeeper constructs a message authorization Keeper
func NewKeeper(env appmodule.Environment, cdc codec.Codec, ak authz.AccountKeeper) Keeper {
	return Keeper{
		Environment: env,
		cdc:         cdc,
		authKeeper:  ak,
	}
}

// getGrant returns grant stored at skey.
func (k Keeper) getGrant(ctx context.Context, skey []byte) (grant authz.Grant, found bool) {
	store := k.KVStoreService.OpenKVStore(ctx)

	bz, err := store.Get(skey)
	if err != nil {
		panic(err)
	}

	if bz == nil {
		return grant, false
	}
	k.cdc.MustUnmarshal(bz, &grant)
	return grant, true
}

func (k Keeper) updateGrant(ctx context.Context, grantee, granter sdk.AccAddress, updated authz.Authorization) error {
	skey := grantStoreKey(grantee, granter, updated.MsgTypeURL())
	grant, found := k.getGrant(ctx, skey)
	if !found {
		return authz.ErrNoAuthorizationFound
	}

	msg, ok := updated.(gogoproto.Message)
	if !ok {
		return sdkerrors.ErrPackAny.Wrapf("cannot proto marshal %T", updated)
	}

	any, err := codectypes.NewAnyWithValue(msg)
	if err != nil {
		return err
	}

	grant.Authorization = any
	store := k.KVStoreService.OpenKVStore(ctx)
	return store.Set(skey, k.cdc.MustMarshal(&grant))
}

// DispatchActions attempts to execute the provided messages via authorization
// grants from the message signer to the grantee.
func (k Keeper) DispatchActions(ctx context.Context, grantee sdk.AccAddress, msgs []sdk.Msg) ([][]byte, error) {
	results := make([][]byte, len(msgs))
	now := k.Environment.HeaderService.HeaderInfo(ctx).Time

	for i, msg := range msgs {
		signers, _, err := k.cdc.GetMsgSigners(msg)
		if err != nil {
			return nil, err
		}

		if len(signers) != 1 {
			return nil, authz.ErrAuthorizationNumOfSigners
		}

		granter := signers[0]

		// If granter != grantee then check authorization.Accept, otherwise we
		// implicitly accept.
		if !bytes.Equal(granter, grantee) {
			skey := grantStoreKey(grantee, granter, sdk.MsgTypeURL(msg))

			grant, found := k.getGrant(ctx, skey)
			if !found {
				return nil, errorsmod.Wrapf(authz.ErrNoAuthorizationFound,
					"failed to get grant with given granter: %s, grantee: %s & msgType: %s ", sdk.AccAddress(granter), grantee, sdk.MsgTypeURL(msg))
			}

			if grant.Expiration != nil && grant.Expiration.Before(now) {
				return nil, authz.ErrAuthorizationExpired
			}

			authorization, err := grant.GetAuthorization()
			if err != nil {
				return nil, err
			}

			// pass the environment in the context
			// users on server/v2 are expected to unwrap the environment from the context
			// users on baseapp can still unwrap the sdk context
			resp, err := authorization.Accept(context.WithValue(ctx, corecontext.EnvironmentContextKey, k.Environment), msg)
			if err != nil {
				return nil, err
			}

			if resp.Delete {
				err = k.DeleteGrant(ctx, grantee, granter, sdk.MsgTypeURL(msg))
			} else if resp.Updated != nil {
				updated, ok := resp.Updated.(authz.Authorization)
				if !ok {
					return nil, fmt.Errorf("expected authz.Authorization but got %T", resp.Updated)
				}
				err = k.updateGrant(ctx, grantee, granter, updated)
			}
			if err != nil {
				return nil, err
			}

			if !resp.Accept {
				return nil, sdkerrors.ErrUnauthorized
			}
		}

		// no need to use the branch service here, as if the transaction fails, the transaction will be reverted
		resp, err := k.MsgRouterService.Invoke(ctx, msg)
		if err != nil {
			return nil, fmt.Errorf("failed to execute message %d; message %v: %w", i, msg, err)
		}

		msgRespAny, err := gogoprotoany.NewAnyWithCacheWithValue(resp)
		if err != nil {
			return nil, fmt.Errorf("failed to create any for response %d; message %s: %w", i, gogoproto.MessageName(msg), err)
		}

		results[i], err = gogoproto.Marshal(msgRespAny)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal response %d; message %s: %w", i, gogoproto.MessageName(msg), err)
		}
	}

	return results, nil
}

// SaveGrant method grants the provided authorization to the grantee on the granter's account
// with the provided expiration time and insert authorization key into the grants queue. If there is an existing authorization grant for the
// same `sdk.Msg` type, this grant overwrites that.
func (k Keeper) SaveGrant(ctx context.Context, grantee, granter sdk.AccAddress, authorization authz.Authorization, expiration *time.Time) error {
	msgType := authorization.MsgTypeURL()
	store := k.KVStoreService.OpenKVStore(ctx)
	skey := grantStoreKey(grantee, granter, msgType)

	grant, err := authz.NewGrant(k.HeaderService.HeaderInfo(ctx).Time, authorization, expiration)
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

	bz, err := k.cdc.Marshal(&grant)
	if err != nil {
		return err
	}

	err = store.Set(skey, bz)
	if err != nil {
		return err
	}

	granterAddr, err := k.authKeeper.AddressCodec().BytesToString(granter)
	if err != nil {
		return err
	}
	granteeAddr, err := k.authKeeper.AddressCodec().BytesToString(grantee)
	if err != nil {
		return err
	}

	return k.EventService.EventManager(ctx).Emit(&authz.EventGrant{
		MsgTypeUrl: authorization.MsgTypeURL(),
		Granter:    granterAddr,
		Grantee:    granteeAddr,
	})
}

// DeleteGrant revokes any authorization for the provided message type granted to the grantee
// by the granter.
func (k Keeper) DeleteGrant(ctx context.Context, grantee, granter sdk.AccAddress, msgType string) error {
	store := k.KVStoreService.OpenKVStore(ctx)
	skey := grantStoreKey(grantee, granter, msgType)
	grant, found := k.getGrant(ctx, skey)
	if !found {
		granterAddr, err := k.authKeeper.AddressCodec().BytesToString(granter)
		if err != nil {
			return errorsmod.Wrapf(authz.ErrNoAuthorizationFound,
				"could not convert granter address to string")
		}
		granteeAddr, err := k.authKeeper.AddressCodec().BytesToString(grantee)
		if err != nil {
			return errorsmod.Wrapf(authz.ErrNoAuthorizationFound,
				"could not convert grantee address to string")
		}
		return errorsmod.Wrapf(authz.ErrNoAuthorizationFound,
			"failed to delete grant with given granter: %s, grantee: %s & msgType: %s ", granterAddr, granteeAddr, msgType)
	}

	if grant.Expiration != nil {
		err := k.removeFromGrantQueue(ctx, skey, granter, grantee, *grant.Expiration)
		if err != nil {
			return err
		}
	}

	err := store.Delete(skey)
	if err != nil {
		return err
	}

	granterAddr, err := k.authKeeper.AddressCodec().BytesToString(granter)
	if err != nil {
		return err
	}
	granteeAddr, err := k.authKeeper.AddressCodec().BytesToString(grantee)
	if err != nil {
		return err
	}
	return k.EventService.EventManager(ctx).Emit(&authz.EventRevoke{
		MsgTypeUrl: msgType,
		Granter:    granterAddr,
		Grantee:    granteeAddr,
	})
}

// DeleteAllGrants revokes all authorizations granted to the grantee by the granter.
func (k Keeper) DeleteAllGrants(ctx context.Context, granter sdk.AccAddress) error {
	var keysToDelete [][]byte

	err := k.IterateGranterGrants(ctx, granter, func(grantee sdk.AccAddress, msgType string) (stop bool, err error) {
		keysToDelete = append(keysToDelete, grantStoreKey(grantee, granter, msgType))
		return false, nil
	})
	if err != nil {
		return err
	}
	if len(keysToDelete) == 0 {
		return errorsmod.Wrapf(authz.ErrNoAuthorizationFound, "no grants found for granter %s", granter)
	}
	for _, key := range keysToDelete {
		_, granteeAddr, msgType := parseGrantStoreKey(key)
		if err := k.DeleteGrant(ctx, granteeAddr, granter, msgType); err != nil {
			return err
		}
	}

	grantAddr, err := k.authKeeper.AddressCodec().BytesToString(granter)
	if err != nil {
		return err
	}

	return k.EventService.EventManager(ctx).Emit(&authz.EventRevokeAll{
		Granter: grantAddr,
	})
}

// GetAuthorizations Returns list of `Authorizations` granted to the grantee by the granter.
func (k Keeper) GetAuthorizations(ctx context.Context, grantee, granter sdk.AccAddress) ([]authz.Authorization, error) {
	store := runtime.KVStoreAdapter(k.KVStoreService.OpenKVStore(ctx))
	key := grantStoreKey(grantee, granter, "")
	iter := storetypes.KVStorePrefixIterator(store, key)
	defer iter.Close()

	var authorizations []authz.Authorization
	for ; iter.Valid(); iter.Next() {
		var authorization authz.Grant
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
func (k Keeper) GetAuthorization(ctx context.Context, grantee, granter sdk.AccAddress, msgType string) (authz.Authorization, *time.Time) {
	grant, found := k.getGrant(ctx, grantStoreKey(grantee, granter, msgType))
	if !found || (grant.Expiration != nil && grant.Expiration.Before(k.HeaderService.HeaderInfo(ctx).Time)) {
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
func (k Keeper) IterateGrants(ctx context.Context,
	handler func(granterAddr, granteeAddr sdk.AccAddress, grant authz.Grant) (bool, error),
) error {
	store := runtime.KVStoreAdapter(k.KVStoreService.OpenKVStore(ctx))
	iter := storetypes.KVStorePrefixIterator(store, GrantKey)
	defer iter.Close()
	for ; iter.Valid(); iter.Next() {
		var grant authz.Grant
		granterAddr, granteeAddr, _ := parseGrantStoreKey(iter.Key())
		k.cdc.MustUnmarshal(iter.Value(), &grant)
		ok, err := handler(granterAddr, granteeAddr, grant)
		if err != nil {
			return err
		}
		if ok {
			break
		}
	}
	return nil
}

func (k Keeper) IterateGranterGrants(ctx context.Context, granter sdk.AccAddress,
	handler func(granteeAddr sdk.AccAddress, msgType string) (stop bool, err error),
) error {
	// no-op if handler is nil
	if handler == nil {
		return nil
	}
	store := runtime.KVStoreAdapter(k.KVStoreService.OpenKVStore(ctx))
	iter := storetypes.KVStorePrefixIterator(store, granterStoreKey(granter))
	defer iter.Close()
	for ; iter.Valid(); iter.Next() {
		_, granteeAddr, msgType := parseGrantStoreKey(iter.Key())
		ok, err := handler(granteeAddr, msgType)
		if err != nil {
			return err
		}
		if ok {
			break
		}
	}
	return nil
}

func (k Keeper) getGrantQueueItem(ctx context.Context, expiration time.Time, granter, grantee sdk.AccAddress) (*authz.GrantQueueItem, error) {
	store := k.KVStoreService.OpenKVStore(ctx)
	bz, err := store.Get(GrantQueueKey(expiration, granter, grantee))
	if err != nil {
		return nil, err
	}

	if bz == nil {
		return &authz.GrantQueueItem{}, nil
	}

	var queueItems authz.GrantQueueItem
	if err := k.cdc.Unmarshal(bz, &queueItems); err != nil {
		return nil, err
	}
	return &queueItems, nil
}

func (k Keeper) setGrantQueueItem(ctx context.Context, expiration time.Time,
	granter, grantee sdk.AccAddress, queueItems *authz.GrantQueueItem,
) error {
	store := k.KVStoreService.OpenKVStore(ctx)
	bz, err := k.cdc.Marshal(queueItems)
	if err != nil {
		return err
	}
	return store.Set(GrantQueueKey(expiration, granter, grantee), bz)
}

// insertIntoGrantQueue inserts a grant key into the grant queue
func (k Keeper) insertIntoGrantQueue(ctx context.Context, granter, grantee sdk.AccAddress, msgType string, expiration time.Time) error {
	queueItems, err := k.getGrantQueueItem(ctx, expiration, granter, grantee)
	if err != nil {
		return err
	}

	queueItems.MsgTypeUrls = append(queueItems.MsgTypeUrls, msgType)
	return k.setGrantQueueItem(ctx, expiration, granter, grantee, queueItems)
}

// removeFromGrantQueue removes a grant key from the grant queue
func (k Keeper) removeFromGrantQueue(ctx context.Context, grantKey []byte, granter, grantee sdk.AccAddress, expiration time.Time) error {
	store := k.KVStoreService.OpenKVStore(ctx)
	key := GrantQueueKey(expiration, granter, grantee)
	bz, err := store.Get(key)
	if err != nil {
		return err
	}

	if bz == nil {
		return errorsmod.Wrap(authz.ErrNoGrantKeyFound, "can't remove grant from the expire queue, grant key not found")
	}

	var queueItem authz.GrantQueueItem
	if err := k.cdc.Unmarshal(bz, &queueItem); err != nil {
		return err
	}

	_, _, msgType := parseGrantStoreKey(grantKey)
	queueItems := queueItem.MsgTypeUrls

	for index, typeURL := range queueItems {
		if err := k.GasService.GasMeter(ctx).Consume(gasCostPerIteration, "grant queue"); err != nil {
			return err
		}

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
func (k Keeper) DequeueAndDeleteExpiredGrants(ctx context.Context, limit int) error {
	store := k.KVStoreService.OpenKVStore(ctx)

	iterator, err := store.Iterator(GrantQueuePrefix, storetypes.InclusiveEndBytes(GrantQueueTimePrefix(k.HeaderService.HeaderInfo(ctx).Time)))
	if err != nil {
		return err
	}
	defer iterator.Close()

	count := 0
	for ; iterator.Valid(); iterator.Next() {
		var queueItem authz.GrantQueueItem
		if err := k.cdc.Unmarshal(iterator.Value(), &queueItem); err != nil {
			return err
		}

		_, granter, grantee, err := parseGrantQueueKey(iterator.Key())
		if err != nil {
			return err
		}

		if err = store.Delete(iterator.Key()); err != nil {
			return err
		}

		for _, typeURL := range queueItem.MsgTypeUrls {
			err = store.Delete(grantStoreKey(grantee, granter, typeURL))
			if err != nil {
				return err
			}
		}

		// limit the amount of iterations to avoid taking too much time
		count++
		if count == limit {
			return nil
		}
	}

	return nil
}
