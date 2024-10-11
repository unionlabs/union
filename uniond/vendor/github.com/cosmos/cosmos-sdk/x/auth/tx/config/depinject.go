package tx

import (
	"context"

	gogoproto "github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	grpcstatus "google.golang.org/grpc/status"
	"google.golang.org/protobuf/reflect/protoreflect"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	txconfigv1 "cosmossdk.io/api/cosmos/tx/config/v1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"
	txsigning "cosmossdk.io/x/tx/signing"
	"cosmossdk.io/x/tx/signing/textual"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/x/auth/tx"
)

// DepinjectModuleName is the module name used for depinject.
const DepinjectModuleName = "tx"

func init() {
	appconfig.RegisterModule(&txconfigv1.Config{},
		appconfig.Provide(ProvideModule),
		appconfig.Provide(ProvideProtoRegistry),
	)
}

type ModuleInputs struct {
	depinject.In

	Config                 *txconfigv1.Config
	AddressCodec           address.Codec
	ValidatorAddressCodec  address.ValidatorAddressCodec
	Codec                  codec.Codec
	ProtoFileResolver      txsigning.ProtoFileResolver
	CustomSignModeHandlers func() []txsigning.SignModeHandler `optional:"true"`
	CustomGetSigners       []txsigning.CustomGetSigner        `optional:"true"`
}

type ModuleOutputs struct {
	depinject.Out

	BaseAppOption       runtime.BaseAppOption // This is only useful for chains using baseapp.
	TxConfig            client.TxConfig
	TxConfigOptions     tx.ConfigOptions
	TxSigningHandlerMap *txsigning.HandlerMap
}

func ProvideProtoRegistry() txsigning.ProtoFileResolver {
	return gogoproto.HybridResolver
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
	var customSignModeHandlers []txsigning.SignModeHandler
	if in.CustomSignModeHandlers != nil {
		customSignModeHandlers = in.CustomSignModeHandlers()
	}

	txConfigOptions := tx.ConfigOptions{
		EnabledSignModes: tx.DefaultSignModes,
		SigningOptions: &txsigning.Options{
			FileResolver:          in.ProtoFileResolver,
			AddressCodec:          in.AddressCodec,
			ValidatorAddressCodec: in.ValidatorAddressCodec,
			CustomGetSigners:      make(map[protoreflect.FullName]txsigning.GetSignersFunc),
		},
		CustomSignModes: customSignModeHandlers,
	}

	for _, mode := range in.CustomGetSigners {
		txConfigOptions.SigningOptions.CustomGetSigners[mode.MsgType] = mode.Fn
	}

	txConfig, err := tx.NewTxConfigWithOptions(in.Codec, txConfigOptions)
	if err != nil {
		panic(err)
	}

	return ModuleOutputs{
		BaseAppOption: func(app *baseapp.BaseApp) {
			app.SetTxDecoder(txConfig.TxDecoder())
			app.SetTxEncoder(txConfig.TxEncoder())
		},
		TxConfig:            txConfig,
		TxConfigOptions:     txConfigOptions,
		TxSigningHandlerMap: txConfig.SignModeHandler(),
	}
}

// NewBankKeeperCoinMetadataQueryFn creates a new Textual struct using the given BankKeeper to retrieve coin metadata.
func NewBankKeeperCoinMetadataQueryFn(bk BankKeeper) textual.CoinMetadataQueryFn {
	return func(ctx context.Context, denom string) (*bankv1beta1.Metadata, error) {
		res, err := bk.DenomMetadataV2(ctx, &bankv1beta1.QueryDenomMetadataRequest{Denom: denom})
		if err != nil {
			return nil, metadataExists(err)
		}

		return res.Metadata, nil
	}
}

// NewGRPCCoinMetadataQueryFn returns a new Textual instance where the metadata
// queries are done via gRPC using the provided GRPC client connection. In the
// SDK, you can pass a client.Context as the GRPC connection.
//
// Example:
//
//	clientCtx := client.GetClientContextFromCmd(cmd)
//	txt := tx.NewTextualWithGRPCConn(clientCtx)
//
// This should be used in the client (root.go) of an application.
func NewGRPCCoinMetadataQueryFn(grpcConn grpc.ClientConnInterface) textual.CoinMetadataQueryFn {
	return func(ctx context.Context, denom string) (*bankv1beta1.Metadata, error) {
		bankQueryClient := bankv1beta1.NewQueryClient(grpcConn)
		res, err := bankQueryClient.DenomMetadata(ctx, &bankv1beta1.QueryDenomMetadataRequest{
			Denom: denom,
		})
		if err != nil {
			return nil, metadataExists(err)
		}

		return res.Metadata, nil
	}
}

// metadataExists parses the error, and only propagates the error if it's
// different than a "not found" error.
func metadataExists(err error) error {
	status, ok := grpcstatus.FromError(err)
	if !ok {
		return err
	}

	// This means we didn't find any metadata for this denom. Returning
	// empty metadata.
	if status.Code() == codes.NotFound {
		return nil
	}

	return err
}
