package service

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"strconv"
	"strings"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/coinbase/rosetta-sdk-go/types"

	"cosmossdk.io/tools/rosetta/lib/errors"
)

// ConstructionCombine Combine creates a network-specific transaction from an unsigned transaction
// and an array of provided signatures. The signed transaction returned from this method will be
// sent to the /construction/submit endpoint by the caller.
func (on OnlineNetwork) ConstructionCombine(ctx context.Context, request *types.ConstructionCombineRequest) (*types.ConstructionCombineResponse, *types.Error) {
	txBytes, err := hex.DecodeString(request.UnsignedTransaction)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}

	signedTx, err := on.client.SignedTx(ctx, txBytes, request.Signatures)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}

	return &types.ConstructionCombineResponse{
		SignedTransaction: hex.EncodeToString(signedTx),
	}, nil
}

// ConstructionDerive Derive returns the AccountIdentifier associated with a public key.
func (on OnlineNetwork) ConstructionDerive(_ context.Context, request *types.ConstructionDeriveRequest) (*types.ConstructionDeriveResponse, *types.Error) {
	account, err := on.client.AccountIdentifierFromPublicKey(request.PublicKey)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}
	return &types.ConstructionDeriveResponse{
		AccountIdentifier: account,
		Metadata:          nil,
	}, nil
}

// ConstructionHash TransactionHash returns the network-specific transaction hash for a signed
// transaction.
func (on OnlineNetwork) ConstructionHash(ctx context.Context, request *types.ConstructionHashRequest) (*types.TransactionIdentifierResponse, *types.Error) {
	bz, err := hex.DecodeString(request.SignedTransaction)
	if err != nil {
		return nil, errors.ToRosetta(errors.WrapError(errors.ErrInvalidTransaction, "error decoding tx"))
	}

	hash := sha256.Sum256(bz)
	bzHash := hash[:]
	hashString := hex.EncodeToString(bzHash)

	return &types.TransactionIdentifierResponse{
		TransactionIdentifier: &types.TransactionIdentifier{
			Hash: strings.ToUpper(hashString),
		},
	}, nil
}

// ConstructionMetadata Get any information required to construct a transaction for a specific
// network (i.e. ChainID, Gas, Memo, ...).
func (on OnlineNetwork) ConstructionMetadata(ctx context.Context, request *types.ConstructionMetadataRequest) (*types.ConstructionMetadataResponse, *types.Error) {
	metadata, err := on.client.ConstructionMetadataFromOptions(ctx, request.Options)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}

	response := &types.ConstructionMetadataResponse{
		Metadata: metadata,
	}

	if metadata["gas_price"] != nil && metadata["gas_limit"] != nil {
		gasPrice, ok := metadata["gas_price"].(string)
		if !ok {
			return nil, errors.ToRosetta(errors.WrapError(errors.ErrBadArgument, "invalid gas_price"))
		}
		if gasPrice == "" { // gas_price is unset. skip fee suggestion
			return response, nil
		}
		price, err := sdk.ParseDecCoin(gasPrice)
		if err != nil {
			return nil, errors.ToRosetta(err)
		}

		gasLimit, ok := metadata["gas_limit"].(float64)
		if !ok {
			return nil, errors.ToRosetta(errors.WrapError(errors.ErrBadArgument, "invalid gas_limit"))
		}
		if gasLimit == 0 { // gas_limit is unset. skip fee suggestion
			return response, nil
		}
		gas := sdk.NewIntFromUint64(uint64(gasLimit))

		suggestedFee := types.Amount{
			Value: strconv.FormatInt(price.Amount.MulInt64(gas.Int64()).Ceil().TruncateInt64(), 10),
			Currency: &(types.Currency{
				Symbol:   price.Denom,
				Decimals: 0,
			}),
		}
		response.SuggestedFee = []*types.Amount{&suggestedFee}
	}

	return response, nil
}

// ConstructionParse Parse is called on both unsigned and signed transactions to understand the
// intent of the formulated transaction. This is run as a sanity check before signing (after
// /construction/payloads) and before broadcast (after /construction/combine).
func (on OnlineNetwork) ConstructionParse(ctx context.Context, request *types.ConstructionParseRequest) (*types.ConstructionParseResponse, *types.Error) {
	txBytes, err := hex.DecodeString(request.Transaction)
	if err != nil {
		err := errors.WrapError(errors.ErrInvalidTransaction, err.Error())
		return nil, errors.ToRosetta(err)
	}
	ops, signers, err := on.client.TxOperationsAndSignersAccountIdentifiers(request.Signed, txBytes)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}
	return &types.ConstructionParseResponse{
		Operations:               ops,
		AccountIdentifierSigners: signers,
		Metadata:                 nil,
	}, nil
}

// ConstructionPayloads Payloads is called with an array of operations and the response from
// /construction/metadata. It returns an unsigned transaction blob and a collection of payloads that
// must be signed by particular AccountIdentifiers using a certain SignatureType.
func (on OnlineNetwork) ConstructionPayloads(ctx context.Context, request *types.ConstructionPayloadsRequest) (*types.ConstructionPayloadsResponse, *types.Error) {
	payload, err := on.client.ConstructionPayload(ctx, request)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}
	return payload, nil
}

// ConstructionPreprocess Preprocess is called prior to /construction/payloads to construct a
// request for any metadata that is needed for transaction construction given (i.e. account nonce).
func (on OnlineNetwork) ConstructionPreprocess(ctx context.Context, request *types.ConstructionPreprocessRequest) (*types.ConstructionPreprocessResponse, *types.Error) {
	options, err := on.client.PreprocessOperationsToOptions(ctx, request)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}

	return options, nil
}

// ConstructionSubmit Submit a pre-signed transaction to the node. This call does not block on the
// transaction being included in a block. Rather, it returns immediately with an indication of
// whether or not the transaction was included in the mempool.
func (on OnlineNetwork) ConstructionSubmit(ctx context.Context, request *types.ConstructionSubmitRequest) (*types.TransactionIdentifierResponse, *types.Error) {
	txBytes, err := hex.DecodeString(request.SignedTransaction)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}

	res, meta, err := on.client.PostTx(txBytes)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}

	return &types.TransactionIdentifierResponse{
		TransactionIdentifier: res,
		Metadata:              meta,
	}, nil
}
