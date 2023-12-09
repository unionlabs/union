package autocli

import (
	"context"
	"fmt"
	"io"
	"time"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	"cosmossdk.io/x/tx/signing/aminojson"
	"github.com/cockroachdb/errors"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/spf13/cobra"
	"google.golang.org/protobuf/reflect/protoreflect"

	"cosmossdk.io/client/v2/internal/flags"
	"cosmossdk.io/client/v2/internal/util"
)

// BuildQueryCommand builds the query commands for all the provided modules. If a custom command is provided for a
// module, this is used instead of any automatically generated CLI commands. This allows apps to a fully dynamic client
// with a more customized experience if a binary with custom commands is downloaded.
func (b *Builder) BuildQueryCommand(appOptions AppOptions, customCmds map[string]*cobra.Command) (*cobra.Command, error) {
	queryCmd := topLevelCmd("query", "Querying subcommands")
	queryCmd.Aliases = []string{"q"}

	if err := b.enhanceCommandCommon(queryCmd, queryCmdType, appOptions, customCmds); err != nil {
		return nil, err
	}

	return queryCmd, nil
}

// AddQueryServiceCommands adds a sub-command to the provided command for each
// method in the specified service and returns the command. This can be used in
// order to add auto-generated commands to an existing command.
func (b *Builder) AddQueryServiceCommands(cmd *cobra.Command, cmdDescriptor *autocliv1.ServiceCommandDescriptor) error {
	for cmdName, subCmdDesc := range cmdDescriptor.SubCommands {
		subCmd := findSubCommand(cmd, cmdName)
		if subCmd == nil {
			subCmd = topLevelCmd(cmdName, fmt.Sprintf("Querying commands for the %s service", subCmdDesc.Service))
		}

		if err := b.AddQueryServiceCommands(subCmd, subCmdDesc); err != nil {
			return err
		}

		cmd.AddCommand(subCmd)
	}

	// skip empty command descriptors
	if cmdDescriptor.Service == "" {
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
		name := protoreflect.Name(option.RpcMethod)
		rpcOptMap[name] = option
		// make sure method exists
		if m := methods.ByName(name); m == nil {
			return fmt.Errorf("rpc method %q not found for service %q", name, service.FullName())
		}
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

		methodCmd, err := b.BuildQueryMethodCommand(methodDescriptor, methodOpts)
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

// BuildQueryMethodCommand creates a gRPC query command for the given service method. This can be used to auto-generate
// just a single command for a single service rpc method.
func (b *Builder) BuildQueryMethodCommand(descriptor protoreflect.MethodDescriptor, options *autocliv1.RpcCommandOptions) (*cobra.Command, error) {
	getClientConn := b.GetClientConn
	serviceDescriptor := descriptor.Parent().(protoreflect.ServiceDescriptor)
	methodName := fmt.Sprintf("/%s/%s", serviceDescriptor.FullName(), descriptor.Name())
	outputType := util.ResolveMessageType(b.TypeResolver, descriptor.Output())
	encoderOptions := aminojson.EncoderOptions{
		Indent:          "  ",
		DoNotSortFields: true,
		TypeResolver:    b.TypeResolver,
		FileResolver:    b.FileResolver,
	}

	cmd, err := b.buildMethodCommandCommon(descriptor, options, func(cmd *cobra.Command, input protoreflect.Message) error {
		cmd.SetContext(context.WithValue(context.Background(), client.ClientContextKey, &b.ClientCtx))

		clientConn, err := getClientConn(cmd)
		if err != nil {
			return err
		}

		output := outputType.New()
		if err := clientConn.Invoke(cmd.Context(), methodName, input.Interface(), output.Interface()); err != nil {
			return err
		}

		if noIndent, _ := cmd.Flags().GetBool(flags.FlagNoIndent); noIndent {
			encoderOptions.Indent = ""
		}

		enc := encoder(aminojson.NewEncoder(encoderOptions))
		bz, err := enc.Marshal(output.Interface())
		if err != nil {
			return fmt.Errorf("cannot marshal response %v: %w", output.Interface(), err)
		}

		err = b.outOrStdoutFormat(cmd, bz)
		return err
	})
	if err != nil {
		return nil, err
	}

	if b.AddQueryConnFlags != nil {
		b.AddQueryConnFlags(cmd)

		cmd.Flags().BoolP(flags.FlagNoIndent, "", false, "Do not indent JSON output")
	}

	// silence usage only for inner txs & queries commands
	if cmd != nil {
		cmd.SilenceUsage = true
	}

	return cmd, nil
}

func encoder(encoder aminojson.Encoder) aminojson.Encoder {
	return encoder.DefineTypeEncoding("google.protobuf.Duration", func(_ *aminojson.Encoder, msg protoreflect.Message, w io.Writer) error {
		var (
			secondsName protoreflect.Name = "seconds"
			nanosName   protoreflect.Name = "nanos"
		)

		fields := msg.Descriptor().Fields()
		secondsField := fields.ByName(secondsName)
		if secondsField == nil {
			return fmt.Errorf("expected seconds field")
		}

		seconds := msg.Get(secondsField).Int()

		nanosField := fields.ByName(nanosName)
		if nanosField == nil {
			return fmt.Errorf("expected nanos field")
		}

		nanos := msg.Get(nanosField).Int()

		_, err := fmt.Fprintf(w, `"%s"`, (time.Duration(seconds)*time.Second + (time.Duration(nanos) * time.Nanosecond)).String())
		return err
	})
}
