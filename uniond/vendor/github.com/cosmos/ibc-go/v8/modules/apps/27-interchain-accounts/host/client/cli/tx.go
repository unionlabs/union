package cli

import (
	"encoding/json"
	"fmt"
	"slices"

	"github.com/cosmos/gogoproto/proto"
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"

	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
)

const (
	memoFlag     string = "memo"
	encodingFlag string = "encoding"
)

func generatePacketDataCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "generate-packet-data [message]",
		Short: "Generates protobuf or proto3 JSON encoded ICA packet data.",
		Long: `generate-packet-data accepts a message string and serializes it (depending on the
encoding parameter) using protobuf or proto3 JSON into packet data which is outputted to stdout.
It can be used in conjunction with send-tx which submits pre-built packet data containing messages 
to be executed on the host chain. The default encoding format is protobuf if none is specified;
otherwise the encoding flag can be used in combination with either "proto3" or "proto3json".`,
		Example: fmt.Sprintf(`%s tx interchain-accounts host generate-packet-data '{
    "@type":"/cosmos.bank.v1beta1.MsgSend",
    "from_address":"cosmos15ccshhmp0gsx29qpqq6g4zmltnnvgmyu9ueuadh9y2nc5zj0szls5gtddz",
    "to_address":"cosmos10h9stc5v6ntgeygf5xf945njqq5h32r53uquvw",
    "amount": [
        {
            "denom": "stake",
            "amount": "1000"
        }
    ]
}' --memo memo --encoding proto3json


%s tx interchain-accounts host generate-packet-data '[{
    "@type":"/cosmos.bank.v1beta1.MsgSend",
    "from_address":"cosmos15ccshhmp0gsx29qpqq6g4zmltnnvgmyu9ueuadh9y2nc5zj0szls5gtddz",
    "to_address":"cosmos10h9stc5v6ntgeygf5xf945njqq5h32r53uquvw",
    "amount": [
        {
            "denom": "stake",
            "amount": "1000"
        }
    ]
},
{
	"@type": "/cosmos.staking.v1beta1.MsgDelegate",
	"delegator_address": "cosmos15ccshhmp0gsx29qpqq6g4zmltnnvgmyu9ueuadh9y2nc5zj0szls5gtddz",
	"validator_address": "cosmosvaloper1qnk2n4nlkpw9xfqntladh74w6ujtulwnmxnh3k",
	"amount": {
		"denom": "stake",
		"amount": "1000"
	}
}]'`, version.AppName, version.AppName),
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			cdc := codec.NewProtoCodec(clientCtx.InterfaceRegistry)

			memo, err := cmd.Flags().GetString(memoFlag)
			if err != nil {
				return err
			}

			encoding, err := cmd.Flags().GetString(encodingFlag)
			if err != nil {
				return err
			}

			if !slices.Contains([]string{icatypes.EncodingProtobuf, icatypes.EncodingProto3JSON}, encoding) {
				return fmt.Errorf("unsupported encoding type: %s", encoding)
			}

			packetDataBytes, err := generatePacketData(cdc, []byte(args[0]), memo, encoding)
			if err != nil {
				return err
			}

			cmd.Println(string(packetDataBytes))

			return nil
		},
	}

	cmd.Flags().String(memoFlag, "", "optional memo to be included in the interchain accounts packet data")
	cmd.Flags().String(encodingFlag, "", "optional encoding format of the messages in the interchain accounts packet data")
	return cmd
}

// generatePacketData takes in message bytes and a memo and serializes the message into an
// instance of InterchainAccountPacketData which is returned as bytes.
func generatePacketData(cdc *codec.ProtoCodec, msgBytes []byte, memo string, encoding string) ([]byte, error) {
	protoMessages, err := convertBytesIntoProtoMessages(cdc, msgBytes)
	if err != nil {
		return nil, err
	}

	return generateIcaPacketDataFromProtoMessages(cdc, protoMessages, memo, encoding)
}

// convertBytesIntoProtoMessages returns a list of proto messages from bytes. The bytes can be in the form of a single
// message, or a json array of messages.
func convertBytesIntoProtoMessages(cdc *codec.ProtoCodec, msgBytes []byte) ([]proto.Message, error) {
	var rawMessages []json.RawMessage
	if err := json.Unmarshal(msgBytes, &rawMessages); err != nil {
		// if we fail to unmarshal a list of messages, we assume we are just dealing with a single message.
		// in this case we return a list of a single item.
		var msg sdk.Msg
		if err := cdc.UnmarshalInterfaceJSON(msgBytes, &msg); err != nil {
			return nil, err
		}

		return []proto.Message{msg}, nil
	}

	sdkMessages := make([]proto.Message, len(rawMessages))
	for i, anyJSON := range rawMessages {
		var msg sdk.Msg
		if err := cdc.UnmarshalInterfaceJSON(anyJSON, &msg); err != nil {
			return nil, err
		}

		sdkMessages[i] = msg
	}

	return sdkMessages, nil
}

// generateIcaPacketDataFromProtoMessages generates ica packet data as bytes from a given set of proto encoded sdk messages and a memo.
func generateIcaPacketDataFromProtoMessages(cdc *codec.ProtoCodec, sdkMessages []proto.Message, memo string, encoding string) ([]byte, error) {
	icaPacketDataBytes, err := icatypes.SerializeCosmosTx(cdc, sdkMessages, encoding)
	if err != nil {
		return nil, err
	}

	icaPacketData := icatypes.InterchainAccountPacketData{
		Type: icatypes.EXECUTE_TX,
		Data: icaPacketDataBytes,
		Memo: memo,
	}

	if err := icaPacketData.ValidateBasic(); err != nil {
		return nil, err
	}

	return cdc.MarshalJSON(&icaPacketData)
}
