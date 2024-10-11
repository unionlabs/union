package cli

import (
	"fmt"
	"strconv"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

// GetCmdIncentivizedPacket returns the unrelayed incentivized packet for a given packetID
func GetCmdIncentivizedPacket() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "packet [port-id] [channel-id] [sequence]",
		Short:   "Query for an unrelayed incentivized packet by port-id, channel-id and packet sequence.",
		Long:    "Query for an unrelayed incentivized packet by port-id, channel-id and packet sequence.",
		Args:    cobra.ExactArgs(3),
		Example: fmt.Sprintf("%s query ibc-fee packet", version.AppName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			portID, channelID := args[0], args[1]
			seq, err := strconv.ParseUint(args[2], 10, 64)
			if err != nil {
				return err
			}

			packetID := channeltypes.NewPacketID(portID, channelID, seq)

			if err := packetID.Validate(); err != nil {
				return err
			}

			req := &types.QueryIncentivizedPacketRequest{
				PacketId:    packetID,
				QueryHeight: uint64(clientCtx.Height),
			}

			queryClient := types.NewQueryClient(clientCtx)

			res, err := queryClient.IncentivizedPacket(cmd.Context(), req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// GetCmdIncentivizedPackets returns all of the unrelayed incentivized packets
func GetCmdIncentivizedPackets() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "packets",
		Short:   "Query for all of the unrelayed incentivized packets and associated fees across all channels.",
		Long:    "Query for all of the unrelayed incentivized packets and associated fees across all channels.",
		Args:    cobra.NoArgs,
		Example: fmt.Sprintf("%s query ibc-fee packets", version.AppName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			req := &types.QueryIncentivizedPacketsRequest{
				Pagination:  pageReq,
				QueryHeight: uint64(clientCtx.Height),
			}

			queryClient := types.NewQueryClient(clientCtx)

			res, err := queryClient.IncentivizedPackets(cmd.Context(), req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "packets")

	return cmd
}

// GetCmdTotalRecvFees returns the command handler for the Query/TotalRecvFees rpc.
func GetCmdTotalRecvFees() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "total-recv-fees [port-id] [channel-id] [sequence]",
		Short:   "Query the total receive fees for a packet",
		Long:    "Query the total receive fees for a packet",
		Args:    cobra.ExactArgs(3),
		Example: fmt.Sprintf("%s query ibc-fee total-recv-fees transfer channel-5 100", version.AppName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			portID, channelID := args[0], args[1]
			seq, err := strconv.ParseUint(args[2], 10, 64)
			if err != nil {
				return err
			}

			packetID := channeltypes.NewPacketID(portID, channelID, seq)

			if err := packetID.Validate(); err != nil {
				return err
			}

			queryClient := types.NewQueryClient(clientCtx)

			req := &types.QueryTotalRecvFeesRequest{
				PacketId: packetID,
			}

			res, err := queryClient.TotalRecvFees(cmd.Context(), req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// GetCmdTotalAckFees returns the command handler for the Query/TotalAckFees rpc.
func GetCmdTotalAckFees() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "total-ack-fees [port-id] [channel-id] [sequence]",
		Short:   "Query the total acknowledgement fees for a packet",
		Long:    "Query the total acknowledgement fees for a packet",
		Args:    cobra.ExactArgs(3),
		Example: fmt.Sprintf("%s query ibc-fee total-ack-fees transfer channel-5 100", version.AppName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			portID, channelID := args[0], args[1]
			seq, err := strconv.ParseUint(args[2], 10, 64)
			if err != nil {
				return err
			}

			packetID := channeltypes.NewPacketID(portID, channelID, seq)

			if err := packetID.Validate(); err != nil {
				return err
			}

			queryClient := types.NewQueryClient(clientCtx)

			req := &types.QueryTotalAckFeesRequest{
				PacketId: packetID,
			}

			res, err := queryClient.TotalAckFees(cmd.Context(), req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// GetCmdTotalTimeoutFees returns the command handler for the Query/TotalTimeoutFees rpc.
func GetCmdTotalTimeoutFees() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "total-timeout-fees [port-id] [channel-id] [sequence]",
		Short:   "Query the total timeout fees for a packet",
		Long:    "Query the total timeout fees for a packet",
		Args:    cobra.ExactArgs(3),
		Example: fmt.Sprintf("%s query ibc-fee total-timeout-fees transfer channel-5 100", version.AppName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			portID, channelID := args[0], args[1]
			seq, err := strconv.ParseUint(args[2], 10, 64)
			if err != nil {
				return err
			}

			packetID := channeltypes.NewPacketID(portID, channelID, seq)

			if err := packetID.Validate(); err != nil {
				return err
			}

			queryClient := types.NewQueryClient(clientCtx)

			req := &types.QueryTotalTimeoutFeesRequest{
				PacketId: packetID,
			}

			res, err := queryClient.TotalTimeoutFees(cmd.Context(), req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// GetCmdPayee returns the command handler for the Query/Payee rpc.
func GetCmdPayee() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "payee [channel-id] [relayer]",
		Short:   "Query the relayer payee address on a given channel",
		Long:    "Query the relayer payee address on a given channel",
		Args:    cobra.ExactArgs(2),
		Example: fmt.Sprintf("%s query ibc-fee payee channel-5 cosmos1layxcsmyye0dc0har9sdfzwckaz8sjwlfsj8zs", version.AppName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			if _, err := sdk.AccAddressFromBech32(args[1]); err != nil {
				return err
			}

			queryClient := types.NewQueryClient(clientCtx)

			req := &types.QueryPayeeRequest{
				ChannelId: args[0],
				Relayer:   args[1],
			}

			res, err := queryClient.Payee(cmd.Context(), req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// GetCmdCounterpartyPayee returns the command handler for the Query/CounterpartyPayee rpc.
func GetCmdCounterpartyPayee() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "counterparty-payee [channel-id] [relayer]",
		Short:   "Query the relayer counterparty payee on a given channel",
		Long:    "Query the relayer counterparty payee on a given channel",
		Args:    cobra.ExactArgs(2),
		Example: fmt.Sprintf("%s query ibc-fee counterparty-payee channel-5 cosmos1layxcsmyye0dc0har9sdfzwckaz8sjwlfsj8zs", version.AppName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			if _, err := sdk.AccAddressFromBech32(args[1]); err != nil {
				return err
			}

			queryClient := types.NewQueryClient(clientCtx)

			req := &types.QueryCounterpartyPayeeRequest{
				ChannelId: args[0],
				Relayer:   args[1],
			}

			res, err := queryClient.CounterpartyPayee(cmd.Context(), req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// GetCmdFeeEnabledChannels returns the command handler for the Query/FeeEnabledChannels rpc.
func GetCmdFeeEnabledChannels() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "channels",
		Short:   "Query the ibc-fee enabled channels",
		Long:    "Query the ibc-fee enabled channels",
		Args:    cobra.NoArgs,
		Example: fmt.Sprintf("%s query ibc-fee channels", version.AppName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			req := &types.QueryFeeEnabledChannelsRequest{
				Pagination:  pageReq,
				QueryHeight: uint64(clientCtx.Height),
			}

			queryClient := types.NewQueryClient(clientCtx)

			res, err := queryClient.FeeEnabledChannels(cmd.Context(), req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "channels")

	return cmd
}

// GetCmdFeeEnabledChannel returns the command handler for the Query/FeeEnabledChannel rpc.
func GetCmdFeeEnabledChannel() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "channel [port-id] [channel-id]",
		Short:   "Query the ibc-fee enabled status of a channel",
		Long:    "Query the ibc-fee enabled status of a channel",
		Args:    cobra.ExactArgs(2),
		Example: fmt.Sprintf("%s query ibc-fee channel transfer channel-6", version.AppName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			req := &types.QueryFeeEnabledChannelRequest{
				PortId:    args[0],
				ChannelId: args[1],
			}

			queryClient := types.NewQueryClient(clientCtx)

			res, err := queryClient.FeeEnabledChannel(cmd.Context(), req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// GetCmdIncentivizedPacketsForChannel returns all of the unrelayed incentivized packets on a given channel
func GetCmdIncentivizedPacketsForChannel() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "packets-for-channel [port-id] [channel-id]",
		Short:   "Query for all of the unrelayed incentivized packets on a given channel",
		Long:    "Query for all of the unrelayed incentivized packets on a given channel. These are packets that have not yet been relayed.",
		Args:    cobra.ExactArgs(2),
		Example: fmt.Sprintf("%s query ibc-fee packets-for-channel", version.AppName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			req := &types.QueryIncentivizedPacketsForChannelRequest{
				Pagination:  pageReq,
				PortId:      args[0],
				ChannelId:   args[1],
				QueryHeight: uint64(clientCtx.Height),
			}

			queryClient := types.NewQueryClient(clientCtx)

			res, err := queryClient.IncentivizedPacketsForChannel(cmd.Context(), req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "packets-for-channel")

	return cmd
}
