package autocli

import (
	"context"
	"fmt"

	gogoproto "github.com/cosmos/gogoproto/proto"
	"github.com/spf13/cobra"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/dynamicpb"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	"cosmossdk.io/client/v2/autocli/flag"
	"cosmossdk.io/client/v2/internal/flags"
	"cosmossdk.io/client/v2/internal/util"
	addresscodec "cosmossdk.io/core/address"

	// the following will be extracted to a separate module
	// https://github.com/cosmos/cosmos-sdk/issues/14403
	govcli "cosmossdk.io/x/gov/client/cli"
	govtypes "cosmossdk.io/x/gov/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"

	"github.com/cosmos/cosmos-sdk/client"
	clienttx "github.com/cosmos/cosmos-sdk/client/tx"
)

// BuildMsgCommand builds the msg commands for all the provided modules. If a custom command is provided for a
// module, this is used instead of any automatically generated CLI commands. This allows apps to a fully dynamic client
// with a more customized experience if a binary with custom commands is downloaded.
func (b *Builder) BuildMsgCommand(ctx context.Context, appOptions AppOptions, customCmds map[string]*cobra.Command) (*cobra.Command, error) {
	msgCmd := topLevelCmd(ctx, "tx", "Transaction subcommands")

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
			short := subCmdDescriptor.Short
			if short == "" {
				short = fmt.Sprintf("Tx commands for the %s service", subCmdDescriptor.Service)
			}
			subCmd = topLevelCmd(cmd.Context(), cmdName, short)
		}

		// Add recursive sub-commands if there are any. This is used for nested services.
		if err := b.AddMsgServiceCommands(subCmd, subCmdDescriptor); err != nil {
			return err
		}

		if !subCmdDescriptor.EnhanceCustomCommand {
			cmd.AddCommand(subCmd)
		}
	}

	if cmdDescriptor.Service == "" {
		// skip empty command descriptor
		return nil
	}

	descriptor, err := b.FileResolver.FindDescriptorByName(protoreflect.FullName(cmdDescriptor.Service))
	if err != nil {
		return fmt.Errorf("can't find service %s: %w", cmdDescriptor.Service, err)
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

		if !util.IsSupportedVersion(methodDescriptor) {
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

		cmd.AddCommand(methodCmd)
	}

	return nil
}

// BuildMsgMethodCommand returns a command that outputs the JSON representation of the message.
func (b *Builder) BuildMsgMethodCommand(descriptor protoreflect.MethodDescriptor, options *autocliv1.RpcCommandOptions) (*cobra.Command, error) {
	execFunc := func(cmd *cobra.Command, input protoreflect.Message) error {
		clientCtx, err := client.GetClientTxContext(cmd)
		if err != nil {
			return err
		}

		clientCtx = clientCtx.WithCmdContext(cmd.Context())
		clientCtx = clientCtx.WithOutput(cmd.OutOrStdout())

		fd := input.Descriptor().Fields().ByName(protoreflect.Name(flag.GetSignerFieldName(input.Descriptor())))
		addressCodec := b.Builder.AddressCodec

		// handle gov proposals commands
		skipProposal, _ := cmd.Flags().GetBool(flags.FlagNoProposal)
		if options.GovProposal && !skipProposal {
			return b.handleGovProposal(cmd, input, clientCtx, addressCodec, fd)
		}

		// set signer to signer field if empty
		if addr := input.Get(fd).String(); addr == "" {
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
	}

	cmd, err := b.buildMethodCommandCommon(descriptor, options, execFunc)
	if err != nil {
		return nil, err
	}

	if b.AddTxConnFlags != nil {
		b.AddTxConnFlags(cmd)
	}

	// silence usage only for inner txs & queries commands
	cmd.SilenceUsage = true

	// set gov proposal flags if command is a gov proposal
	if options.GovProposal {
		govcli.AddGovPropFlagsToCmd(cmd)
		cmd.Flags().Bool(flags.FlagNoProposal, false, "Skip gov proposal and submit a normal transaction")
	}

	return cmd, nil
}

// handleGovProposal sets the authority field of the message to the gov module address and creates a gov proposal.
func (b *Builder) handleGovProposal(
	cmd *cobra.Command,
	input protoreflect.Message,
	clientCtx client.Context,
	addressCodec addresscodec.Codec,
	fd protoreflect.FieldDescriptor,
) error {
	govAuthority := authtypes.NewModuleAddress(govtypes.ModuleName)
	authority, err := addressCodec.BytesToString(govAuthority.Bytes())
	if err != nil {
		return fmt.Errorf("failed to convert gov authority: %w", err)
	}
	input.Set(fd, protoreflect.ValueOfString(authority))

	signerFromFlag := clientCtx.GetFromAddress()
	signer, err := addressCodec.BytesToString(signerFromFlag.Bytes())
	if err != nil {
		return fmt.Errorf("failed to set signer on message, got %q: %w", signerFromFlag, err)
	}

	proposal, err := govcli.ReadGovPropCmdFlags(signer, cmd.Flags())
	if err != nil {
		return err
	}

	// AutoCLI uses protov2 messages, while the SDK only supports proto v1 messages.
	// Here we use dynamicpb, to create a proto v1 compatible message.
	// The SDK codec will handle protov2 -> protov1 (marshal)
	msg := dynamicpb.NewMessage(input.Descriptor())
	proto.Merge(msg, input.Interface())

	if err := proposal.SetMsgs([]gogoproto.Message{msg}); err != nil {
		return fmt.Errorf("failed to set msg in proposal %w", err)
	}

	return clienttx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposal)
}
