package keeper

import (
	"context"

	"github.com/cosmos/cosmos-sdk/store/prefix"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/query"
	"github.com/cosmos/cosmos-sdk/x/nft"
)

var _ nft.QueryServer = Keeper{}

// Balance return the number of NFTs of a given class owned by the owner, same as balanceOf in ERC721
func (k Keeper) Balance(goCtx context.Context, r *nft.QueryBalanceRequest) (*nft.QueryBalanceResponse, error) {
	if r == nil {
		return nil, sdkerrors.ErrInvalidRequest.Wrap("empty request")
	}

	if len(r.ClassId) == 0 {
		return nil, nft.ErrEmptyClassID
	}

	owner, err := sdk.AccAddressFromBech32(r.Owner)
	if err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	balance := k.GetBalance(ctx, r.ClassId, owner)
	return &nft.QueryBalanceResponse{Amount: balance}, nil
}

// Owner return the owner of the NFT based on its class and id, same as ownerOf in ERC721
func (k Keeper) Owner(goCtx context.Context, r *nft.QueryOwnerRequest) (*nft.QueryOwnerResponse, error) {
	if r == nil {
		return nil, sdkerrors.ErrInvalidRequest.Wrap("empty request")
	}

	if len(r.ClassId) == 0 {
		return nil, nft.ErrEmptyClassID
	}

	if len(r.Id) == 0 {
		return nil, nft.ErrEmptyNFTID
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	owner := k.GetOwner(ctx, r.ClassId, r.Id)
	return &nft.QueryOwnerResponse{Owner: owner.String()}, nil
}

// Supply return the number of NFTs from the given class, same as totalSupply of ERC721.
func (k Keeper) Supply(goCtx context.Context, r *nft.QuerySupplyRequest) (*nft.QuerySupplyResponse, error) {
	if r == nil {
		return nil, sdkerrors.ErrInvalidRequest.Wrap("empty request")
	}

	if len(r.ClassId) == 0 {
		return nil, nft.ErrEmptyClassID
	}
	ctx := sdk.UnwrapSDKContext(goCtx)
	supply := k.GetTotalSupply(ctx, r.ClassId)
	return &nft.QuerySupplyResponse{Amount: supply}, nil
}

// NFTs queries all NFTs of a given class or owner (at least one must be provided), similar to tokenByIndex in ERC721Enumerable
func (k Keeper) NFTs(goCtx context.Context, r *nft.QueryNFTsRequest) (*nft.QueryNFTsResponse, error) {
	if r == nil {
		return nil, sdkerrors.ErrInvalidRequest.Wrap("empty request")
	}

	var err error
	var owner sdk.AccAddress

	if len(r.Owner) > 0 {
		owner, err = sdk.AccAddressFromBech32(r.Owner)
		if err != nil {
			return nil, err
		}
	}

	var nfts []*nft.NFT
	var pageRes *query.PageResponse
	ctx := sdk.UnwrapSDKContext(goCtx)

	switch {
	case len(r.ClassId) > 0 && len(r.Owner) > 0:
		if pageRes, err = query.Paginate(k.getClassStoreByOwner(ctx, owner, r.ClassId), r.Pagination, func(key []byte, _ []byte) error {
			nft, has := k.GetNFT(ctx, r.ClassId, string(key))
			if has {
				nfts = append(nfts, &nft)
			}
			return nil
		}); err != nil {
			return nil, err
		}
	case len(r.ClassId) > 0 && len(r.Owner) == 0:
		nftStore := k.getNFTStore(ctx, r.ClassId)
		if pageRes, err = query.Paginate(nftStore, r.Pagination, func(_ []byte, value []byte) error {
			var nft nft.NFT
			if err := k.cdc.Unmarshal(value, &nft); err != nil {
				return err
			}
			nfts = append(nfts, &nft)
			return nil
		}); err != nil {
			return nil, err
		}
	case len(r.ClassId) == 0 && len(r.Owner) > 0:
		if pageRes, err = query.Paginate(k.prefixStoreNftOfClassByOwner(ctx, owner), r.Pagination, func(key []byte, value []byte) error {
			classID, nftID := parseNftOfClassByOwnerStoreKey(key)
			if n, has := k.GetNFT(ctx, classID, nftID); has {
				nfts = append(nfts, &n)
			}
			return nil
		}); err != nil {
			return nil, err
		}
	default:
		return nil, sdkerrors.ErrInvalidRequest.Wrap("must provide at least one of classID or owner")
	}
	return &nft.QueryNFTsResponse{
		Nfts:       nfts,
		Pagination: pageRes,
	}, nil
}

// NFT return an NFT based on its class and id.
func (k Keeper) NFT(goCtx context.Context, r *nft.QueryNFTRequest) (*nft.QueryNFTResponse, error) {
	if r == nil {
		return nil, sdkerrors.ErrInvalidRequest.Wrap("empty request")
	}

	if len(r.ClassId) == 0 {
		return nil, nft.ErrEmptyClassID
	}
	if len(r.Id) == 0 {
		return nil, nft.ErrEmptyNFTID
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	n, has := k.GetNFT(ctx, r.ClassId, r.Id)
	if !has {
		return nil, nft.ErrNFTNotExists.Wrapf("not found nft: class: %s, id: %s", r.ClassId, r.Id)
	}
	return &nft.QueryNFTResponse{Nft: &n}, nil
}

// Class return an NFT class based on its id
func (k Keeper) Class(goCtx context.Context, r *nft.QueryClassRequest) (*nft.QueryClassResponse, error) {
	if r == nil {
		return nil, sdkerrors.ErrInvalidRequest.Wrap("empty request")
	}

	if len(r.ClassId) == 0 {
		return nil, nft.ErrEmptyClassID
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	class, has := k.GetClass(ctx, r.ClassId)
	if !has {
		return nil, nft.ErrClassNotExists.Wrapf("not found class: %s", r.ClassId)
	}
	return &nft.QueryClassResponse{Class: &class}, nil
}

// Classes return all NFT classes
func (k Keeper) Classes(goCtx context.Context, r *nft.QueryClassesRequest) (*nft.QueryClassesResponse, error) {
	if r == nil {
		return nil, sdkerrors.ErrInvalidRequest.Wrap("empty request")
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	store := ctx.KVStore(k.storeKey)
	classStore := prefix.NewStore(store, ClassKey)

	var classes []*nft.Class
	pageRes, err := query.Paginate(classStore, r.Pagination, func(_ []byte, value []byte) error {
		var class nft.Class
		if err := k.cdc.Unmarshal(value, &class); err != nil {
			return err
		}
		classes = append(classes, &class)
		return nil
	})
	if err != nil {
		return nil, err
	}
	return &nft.QueryClassesResponse{
		Classes:    classes,
		Pagination: pageRes,
	}, nil
}
