package bindings

import (
	"encoding/json"
	"fmt"

	wasmvmtypes "github.com/CosmWasm/wasmvm/types"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	bindingstypes "union/x/tokenfactory/bindings/types"
)

// CustomQuerier dispatches custom CosmWasm bindings queries.
func CustomQuerier(qp *QueryPlugin) func(ctx sdk.Context, request json.RawMessage) ([]byte, error) {
	return func(ctx sdk.Context, request json.RawMessage) ([]byte, error) {
		var contractQuery bindingstypes.TokenFactoryQuery
		if err := json.Unmarshal(request, &contractQuery); err != nil {
			return nil, errorsmod.Wrap(err, "tokenfactory query")
		}

		switch {
		case contractQuery.FullDenom != nil:
			creator := contractQuery.FullDenom.CreatorAddr
			subdenom := contractQuery.FullDenom.Subdenom

			fullDenom, err := GetFullDenom(creator, subdenom)
			if err != nil {
				return nil, errorsmod.Wrap(err, "osmo full denom query")
			}

			res := bindingstypes.FullDenomResponse{
				Denom: fullDenom,
			}

			bz, err := json.Marshal(res)
			if err != nil {
				return nil, errorsmod.Wrap(err, "failed to marshal FullDenomResponse")
			}

			return bz, nil

		case contractQuery.Admin != nil:
			res, err := qp.GetDenomAdmin(ctx, contractQuery.Admin.Denom)
			if err != nil {
				return nil, err
			}

			bz, err := json.Marshal(res)
			if err != nil {
				return nil, fmt.Errorf("failed to JSON marshal AdminResponse: %w", err)
			}

			return bz, nil

		case contractQuery.Metadata != nil:
			res, err := qp.GetMetadata(ctx, contractQuery.Metadata.Denom)
			if err != nil {
				return nil, err
			}

			bz, err := json.Marshal(res)
			if err != nil {
				return nil, fmt.Errorf("failed to JSON marshal MetadataResponse: %w", err)
			}

			return bz, nil

		case contractQuery.DenomsByCreator != nil:
			res, err := qp.GetDenomsByCreator(ctx, contractQuery.DenomsByCreator.Creator)
			if err != nil {
				return nil, err
			}

			bz, err := json.Marshal(res)
			if err != nil {
				return nil, fmt.Errorf("failed to JSON marshal DenomsByCreatorResponse: %w", err)
			}

			return bz, nil

		case contractQuery.Params != nil:
			res, err := qp.GetParams(ctx)
			if err != nil {
				return nil, err
			}

			bz, err := json.Marshal(res)
			if err != nil {
				return nil, fmt.Errorf("failed to JSON marshal ParamsResponse: %w", err)
			}

			return bz, nil

		default:
			return nil, wasmvmtypes.UnsupportedRequest{Kind: "unknown token query variant"}
		}
	}
}

// ConvertSdkCoinsToWasmCoins converts sdk type coins to wasm vm type coins
func ConvertSdkCoinsToWasmCoins(coins []sdk.Coin) wasmvmtypes.Coins {
	var toSend wasmvmtypes.Coins
	for _, coin := range coins {
		c := ConvertSdkCoinToWasmCoin(coin)
		toSend = append(toSend, c)
	}
	return toSend
}

// ConvertSdkCoinToWasmCoin converts a sdk type coin to a wasm vm type coin
func ConvertSdkCoinToWasmCoin(coin sdk.Coin) wasmvmtypes.Coin {
	return wasmvmtypes.Coin{
		Denom: coin.Denom,
		// Note: tokenfactory tokens have 18 decimal places, so 10^22 is common, no longer in u64 range
		Amount: coin.Amount.String(),
	}
}
