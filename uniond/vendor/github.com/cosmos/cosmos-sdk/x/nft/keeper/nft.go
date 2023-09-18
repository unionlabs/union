package keeper

import (
	"github.com/cosmos/cosmos-sdk/store/prefix"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/nft"
)

// Mint defines a method for minting a new nft
func (k Keeper) Mint(ctx sdk.Context, token nft.NFT, receiver sdk.AccAddress) error {
	if !k.HasClass(ctx, token.ClassId) {
		return sdkerrors.Wrap(nft.ErrClassNotExists, token.ClassId)
	}

	if k.HasNFT(ctx, token.ClassId, token.Id) {
		return sdkerrors.Wrap(nft.ErrNFTExists, token.Id)
	}

	k.mintWithNoCheck(ctx, token, receiver)
	return nil
}

// mintWithNoCheck defines a method for minting a new nft
// Note: this method does not check whether the class already exists in nft.
// The upper-layer application needs to check it when it needs to use it.
func (k Keeper) mintWithNoCheck(ctx sdk.Context, token nft.NFT, receiver sdk.AccAddress) {
	k.setNFT(ctx, token)
	k.setOwner(ctx, token.ClassId, token.Id, receiver)
	k.incrTotalSupply(ctx, token.ClassId)

	ctx.EventManager().EmitTypedEvent(&nft.EventMint{
		ClassId: token.ClassId,
		Id:      token.Id,
		Owner:   receiver.String(),
	})
}

// Burn defines a method for burning a nft from a specific account.
// Note: When the upper module uses this method, it needs to authenticate nft
func (k Keeper) Burn(ctx sdk.Context, classID string, nftID string) error {
	if !k.HasClass(ctx, classID) {
		return sdkerrors.Wrap(nft.ErrClassNotExists, classID)
	}

	if !k.HasNFT(ctx, classID, nftID) {
		return sdkerrors.Wrap(nft.ErrNFTNotExists, nftID)
	}

	k.burnWithNoCheck(ctx, classID, nftID)
	return nil
}

// burnWithNoCheck defines a method for burning a nft from a specific account.
// Note: this method does not check whether the class already exists in nft.
// The upper-layer application needs to check it when it needs to use it
func (k Keeper) burnWithNoCheck(ctx sdk.Context, classID string, nftID string) error {
	owner := k.GetOwner(ctx, classID, nftID)
	nftStore := k.getNFTStore(ctx, classID)
	nftStore.Delete([]byte(nftID))

	k.deleteOwner(ctx, classID, nftID, owner)
	k.decrTotalSupply(ctx, classID)
	ctx.EventManager().EmitTypedEvent(&nft.EventBurn{
		ClassId: classID,
		Id:      nftID,
		Owner:   owner.String(),
	})
	return nil
}

// Update defines a method for updating an exist nft
// Note: When the upper module uses this method, it needs to authenticate nft
func (k Keeper) Update(ctx sdk.Context, token nft.NFT) error {
	if !k.HasClass(ctx, token.ClassId) {
		return sdkerrors.Wrap(nft.ErrClassNotExists, token.ClassId)
	}

	if !k.HasNFT(ctx, token.ClassId, token.Id) {
		return sdkerrors.Wrap(nft.ErrNFTNotExists, token.Id)
	}
	k.updateWithNoCheck(ctx, token)
	return nil
}

// Update defines a method for updating an exist nft
// Note: this method does not check whether the class already exists in nft.
// The upper-layer application needs to check it when it needs to use it
func (k Keeper) updateWithNoCheck(ctx sdk.Context, token nft.NFT) {
	k.setNFT(ctx, token)
}

// Transfer defines a method for sending a nft from one account to another account.
// Note: When the upper module uses this method, it needs to authenticate nft
func (k Keeper) Transfer(ctx sdk.Context,
	classID string,
	nftID string,
	receiver sdk.AccAddress,
) error {
	if !k.HasClass(ctx, classID) {
		return sdkerrors.Wrap(nft.ErrClassNotExists, classID)
	}

	if !k.HasNFT(ctx, classID, nftID) {
		return sdkerrors.Wrap(nft.ErrNFTNotExists, nftID)
	}

	k.transferWithNoCheck(ctx, classID, nftID, receiver)
	return nil
}

// Transfer defines a method for sending a nft from one account to another account.
// Note: this method does not check whether the class already exists in nft.
// The upper-layer application needs to check it when it needs to use it
func (k Keeper) transferWithNoCheck(ctx sdk.Context,
	classID string,
	nftID string,
	receiver sdk.AccAddress,
) error {
	owner := k.GetOwner(ctx, classID, nftID)
	k.deleteOwner(ctx, classID, nftID, owner)
	k.setOwner(ctx, classID, nftID, receiver)
	return nil
}

// GetNFT returns the nft information of the specified classID and nftID
func (k Keeper) GetNFT(ctx sdk.Context, classID, nftID string) (nft.NFT, bool) {
	store := k.getNFTStore(ctx, classID)
	bz := store.Get([]byte(nftID))
	if len(bz) == 0 {
		return nft.NFT{}, false
	}
	var nft nft.NFT
	k.cdc.MustUnmarshal(bz, &nft)
	return nft, true
}

// GetNFTsOfClassByOwner returns all nft information of the specified classID under the specified owner
func (k Keeper) GetNFTsOfClassByOwner(ctx sdk.Context, classID string, owner sdk.AccAddress) (nfts []nft.NFT) {
	ownerStore := k.getClassStoreByOwner(ctx, owner, classID)
	iterator := ownerStore.Iterator(nil, nil)
	defer iterator.Close()
	for ; iterator.Valid(); iterator.Next() {
		nft, has := k.GetNFT(ctx, classID, string(iterator.Key()))
		if has {
			nfts = append(nfts, nft)
		}
	}
	return nfts
}

// GetNFTsOfClass returns all nft information under the specified classID
func (k Keeper) GetNFTsOfClass(ctx sdk.Context, classID string) (nfts []nft.NFT) {
	nftStore := k.getNFTStore(ctx, classID)
	iterator := nftStore.Iterator(nil, nil)
	defer iterator.Close()
	for ; iterator.Valid(); iterator.Next() {
		var nft nft.NFT
		k.cdc.MustUnmarshal(iterator.Value(), &nft)
		nfts = append(nfts, nft)
	}
	return nfts
}

// GetOwner returns the owner information of the specified nft
func (k Keeper) GetOwner(ctx sdk.Context, classID string, nftID string) sdk.AccAddress {
	store := ctx.KVStore(k.storeKey)
	bz := store.Get(ownerStoreKey(classID, nftID))
	return sdk.AccAddress(bz)
}

// GetBalance returns the specified account, the number of all nfts under the specified classID
func (k Keeper) GetBalance(ctx sdk.Context, classID string, owner sdk.AccAddress) uint64 {
	nfts := k.GetNFTsOfClassByOwner(ctx, classID, owner)
	return uint64(len(nfts))
}

// GetTotalSupply returns the number of all nfts under the specified classID
func (k Keeper) GetTotalSupply(ctx sdk.Context, classID string) uint64 {
	store := ctx.KVStore(k.storeKey)
	bz := store.Get(classTotalSupply(classID))
	return sdk.BigEndianToUint64(bz)
}

// HasNFT determines whether the specified classID and nftID exist
func (k Keeper) HasNFT(ctx sdk.Context, classID, id string) bool {
	store := k.getNFTStore(ctx, classID)
	return store.Has([]byte(id))
}

func (k Keeper) setNFT(ctx sdk.Context, token nft.NFT) {
	nftStore := k.getNFTStore(ctx, token.ClassId)
	bz := k.cdc.MustMarshal(&token)
	nftStore.Set([]byte(token.Id), bz)
}

func (k Keeper) setOwner(ctx sdk.Context, classID, nftID string, owner sdk.AccAddress) {
	store := ctx.KVStore(k.storeKey)
	store.Set(ownerStoreKey(classID, nftID), owner.Bytes())

	ownerStore := k.getClassStoreByOwner(ctx, owner, classID)
	ownerStore.Set([]byte(nftID), Placeholder)
}

func (k Keeper) deleteOwner(ctx sdk.Context, classID, nftID string, owner sdk.AccAddress) {
	store := ctx.KVStore(k.storeKey)
	store.Delete(ownerStoreKey(classID, nftID))

	ownerStore := k.getClassStoreByOwner(ctx, owner, classID)
	ownerStore.Delete([]byte(nftID))
}

func (k Keeper) getNFTStore(ctx sdk.Context, classID string) prefix.Store {
	store := ctx.KVStore(k.storeKey)
	return prefix.NewStore(store, nftStoreKey(classID))
}

func (k Keeper) getClassStoreByOwner(ctx sdk.Context, owner sdk.AccAddress, classID string) prefix.Store {
	store := ctx.KVStore(k.storeKey)
	key := nftOfClassByOwnerStoreKey(owner, classID)
	return prefix.NewStore(store, key)
}

func (k Keeper) prefixStoreNftOfClassByOwner(ctx sdk.Context, owner sdk.AccAddress) prefix.Store {
	store := ctx.KVStore(k.storeKey)
	key := prefixNftOfClassByOwnerStoreKey(owner)
	return prefix.NewStore(store, key)
}

func (k Keeper) incrTotalSupply(ctx sdk.Context, classID string) {
	supply := k.GetTotalSupply(ctx, classID) + 1
	k.updateTotalSupply(ctx, classID, supply)
}

func (k Keeper) decrTotalSupply(ctx sdk.Context, classID string) {
	supply := k.GetTotalSupply(ctx, classID) - 1
	k.updateTotalSupply(ctx, classID, supply)
}

func (k Keeper) updateTotalSupply(ctx sdk.Context, classID string, supply uint64) {
	store := ctx.KVStore(k.storeKey)
	supplyKey := classTotalSupply(classID)
	store.Set(supplyKey, sdk.Uint64ToBigEndian(supply))
}
