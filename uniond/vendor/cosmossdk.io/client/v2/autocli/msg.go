package autocli

import (
	"context"
	"fmt"

	"github.com/cockroachdb/errors"
	"github.com/spf13/cobra"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/dynamicpb"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	"cosmossdk.io/client/v2/autocli/flag"
	"cosmossdk.io/client/v2/internal/util"

	"github.com/cosmos/cosmos-sdk/client"
	clienttx "github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
	authtxconfig "github.com/cosmos/cosmos-sdk/x/auth/tx/config"
)

// BuildMsgCommand builds the msg commands for all the provided modules. If a custom command is provided for a
// module, this is used instead of any automatically generated CLI commands. This allows apps to a fully dynamic client
// with a more customized experience if a binary with custom commands is downloaded.
func (b *Builder) BuildMsgCommand(appOptions AppOptions, customCmds map[string]*cobra.Command) (*cobra.Command, error) {
	msgCmd := topLevelCmd("tx", "Transaction subcommands")
	if err := b.enhanceCommandCommon(msgCmd, msgCmdType, appOptions, customCmds); err != nil {
		return nil, err
	}

	return msgCmd, nil
}

// AddMsgServiceCommands adds a sub-command to the provided command for each
// method in the specified service and returns the command. This can be used in
// order to add auto-generated commands to an existing command.
func (b *Builder) AddMsgServiceCommands(cmd *cobra.Command, cmdDescriptor *autocliv1.ServiceCommandDescriptor) error {
	for cmdName, subCmdDescriptor := range cmdDescriptor.SubCommands {
		subCmd := findSubCommand(cmd, cmdName)
		if subCmd == nil {
			subCmd = topLevelCmd(cmdName, fmt.Sprintf("Tx commands for the %s service", subCmdDescriptor.Service))
		}

		// Add recursive sub-commands if there are any. This is used for nested services.
		if err := b.AddMsgServiceCommands(subCmd, subCmdDescriptor); err != nil {
			return err
		}

		cmd.AddCommand(subCmd)
	}

	if cmdDescriptor.Service == "" {
		// skip empty command descriptor
		return nil
	}

	descriptor, err := b.FileResolver.FindDescriptorByName(protoreflect.FullName(cmdDescriptor.Service))
	if err != nil {
		return errors.Errorf("can't find service %s: %v", cmdDescriptor.Service, err)
	}
	service := descriptor.(protoreflect.ServiceDescriptor)
	methods := service.Methods()

	rpcOptMap := map[protoreflect.Name]*autocliv1.RpcCommandOptions{}
	for _, option := range cmdDescriptor.RpcCommandOptions {
		methodName := protoreflect.Name(option.RpcMethod)
		// validate that methods exist
		if m := methods.ByName(methodName); m == nil {
			return fmt.Errorf("rpc method %q not found for service %q", methodName, service.FullName())
		}
		rpcOptMap[methodName] = option

	}

	for i := 0; i < methods.Len(); i++ {
		methodDescriptor := methods.Get(i)
		methodOpts, ok := rpcOptMap[methodDescriptor.Name()]
		if !ok {
			methodOpts = &autocliv1.RpcCommandOptions{}
		}

		if methodOpts.Skip {
			continue
		}

		if !util.IsSupportedVersion(util.DescriptorDocs(methodDescriptor)) {
			continue
		}

		methodCmd, err := b.BuildMsgMethodCommand(methodDescriptor, methodOpts)
		if err != nil {
			return err
		}

		if findSubCommand(cmd, methodCmd.Name()) != nil {
			// do not overwrite existing commands
			// we do not display a warning because you may want to overwrite an autocli command
			continue
		}

		if methodCmd != nil {
			cmd.AddCommand(methodCmd)
		}
	}

	return nil
}

// BuildMsgMethodCommand returns a command that outputs the JSON representation of the message.
func (b *Builder) BuildMsgMethodCommand(descriptor protoreflect.MethodDescriptor, options *autocliv1.RpcCommandOptions) (*cobra.Command, error) {
	cmd, err := b.buildMethodCommandCommon(descriptor, options, func(cmd *cobra.Command, input protoreflect.Message) error {
		cmd.SetContext(context.WithValue(context.Background(), client.ClientContextKey, &b.ClientCtx))

		clientCtx, err := client.GetClientTxContext(cmd)
		if err != nil {
			return err
		}

		clientCtx = clientCtx.WithCmdContext(cmd.Context())
		clientCtx = clientCtx.WithOutput(cmd.OutOrStdout())

		// enable sign mode textual
		// the config is always overwritten as we need to have set the flags to the client context
		// this ensures that the context has the correct client.
		if !clientCtx.Offline {
			b.TxConfigOpts.EnabledSignModes = append(b.TxConfigOpts.EnabledSignModes, signing.SignMode_SIGN_MODE_TEXTUAL)
			b.TxConfigOpts.TextualCoinMetadataQueryFn = authtxconfig.NewGRPCCoinMetadataQueryFn(clientCtx)

			txConfig, err := authtx.NewTxConfigWithOptions(
				codec.NewProtoCodec(clientCtx.InterfaceRegistry),
				b.TxConfigOpts,
			)
			if err != nil {
				return err
			}

			clientCtx = clientCtx.WithTxConfig(txConfig)
		}

		// set signer to signer field if empty
		fd := input.Descriptor().Fields().ByName(protoreflect.Name(flag.GetSignerFieldName(input.Descriptor())))
		if addr := input.Get(fd).String(); addr == "" {
			addressCodec := b.Builder.AddressCodec

			scalarType, ok := flag.GetScalarType(fd)
			if ok {
				// override address codec if validator or consensus address
				switch scalarType {
				case flag.ValidatorAddressStringScalarType:
					addressCodec = b.Builder.ValidatorAddressCodec
				case flag.ConsensusAddressStringScalarType:
					addressCodec = b.Builder.ConsensusAddressCodec
				}
			}

			signerFromFlag := clientCtx.GetFromAddress()
			signer, err := addressCodec.BytesToString(signerFromFlag.Bytes())
			if err != nil {
				return fmt.Errorf("failed to set signer on message, got %v: %w", signerFromFlag, err)
			}

			input.Set(fd, protoreflect.ValueOfString(signer))
		}

		// AutoCLI uses protov2 messages, while the SDK only supports proto v1 messages.
		// Here we use dynamicpb, to create a proto v1 compatible message.
		// The SDK codec will handle protov2 -> protov1 (marshal)
		msg := dynamicpb.NewMessage(input.Descriptor())
		proto.Merge(msg, input.Interface())

		return clienttx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
	})

	if b.AddTxConnFlags != nil {
		b.AddTxConnFlags(cmd)
	}

	// silence usage only for inner txs & queries commands
	if cmd != nil {
		cmd.SilenceUsage = true
	}

	return cmd, err
}
