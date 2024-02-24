export const usc01relayAbi = <const>[
  {
    type: "constructor",
    inputs: [
      {
        name: "_ibcHandler",
        type: "address",
        internalType: "contract IBCHandler",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "getCounterpartyEndpoint",
    inputs: [
      {
        name: "sourcePort",
        type: "string",
        internalType: "string",
      },
      {
        name: "sourceChannel",
        type: "string",
        internalType: "string",
      },
    ],
    outputs: [
      {
        name: "",
        type: "tuple",
        internalType: "struct IbcCoreChannelV1Counterparty.Data",
        components: [
          {
            name: "port_id",
            type: "string",
            internalType: "string",
          },
          {
            name: "channel_id",
            type: "string",
            internalType: "string",
          },
        ],
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "getDenomAddress",
    inputs: [
      {
        name: "sourcePort",
        type: "string",
        internalType: "string",
      },
      {
        name: "sourceChannel",
        type: "string",
        internalType: "string",
      },
      {
        name: "denom",
        type: "string",
        internalType: "string",
      },
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "getOutstanding",
    inputs: [
      {
        name: "sourcePort",
        type: "string",
        internalType: "string",
      },
      {
        name: "sourceChannel",
        type: "string",
        internalType: "string",
      },
      {
        name: "token",
        type: "address",
        internalType: "address",
      },
    ],
    outputs: [
      {
        name: "",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "ibcAddress",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "onAcknowledgementPacket",
    inputs: [
      {
        name: "ibcPacket",
        type: "tuple",
        internalType: "struct IbcCoreChannelV1Packet.Data",
        components: [
          {
            name: "sequence",
            type: "uint64",
            internalType: "uint64",
          },
          {
            name: "source_port",
            type: "string",
            internalType: "string",
          },
          {
            name: "source_channel",
            type: "string",
            internalType: "string",
          },
          {
            name: "destination_port",
            type: "string",
            internalType: "string",
          },
          {
            name: "destination_channel",
            type: "string",
            internalType: "string",
          },
          {
            name: "data",
            type: "bytes",
            internalType: "bytes",
          },
          {
            name: "timeout_height",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64",
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64",
              },
            ],
          },
          {
            name: "timeout_timestamp",
            type: "uint64",
            internalType: "uint64",
          },
        ],
      },
      {
        name: "acknowledgement",
        type: "bytes",
        internalType: "bytes",
      },
      {
        name: "_relayer",
        type: "address",
        internalType: "address",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onChanCloseConfirm",
    inputs: [
      {
        name: "_portId",
        type: "string",
        internalType: "string",
      },
      {
        name: "_channelId",
        type: "string",
        internalType: "string",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onChanCloseInit",
    inputs: [
      {
        name: "_portId",
        type: "string",
        internalType: "string",
      },
      {
        name: "_channelId",
        type: "string",
        internalType: "string",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onChanOpenAck",
    inputs: [
      {
        name: "portId",
        type: "string",
        internalType: "string",
      },
      {
        name: "channelId",
        type: "string",
        internalType: "string",
      },
      {
        name: "counterpartyChannelId",
        type: "string",
        internalType: "string",
      },
      {
        name: "counterpartyVersion",
        type: "string",
        internalType: "string",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onChanOpenConfirm",
    inputs: [
      {
        name: "_portId",
        type: "string",
        internalType: "string",
      },
      {
        name: "_channelId",
        type: "string",
        internalType: "string",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onChanOpenInit",
    inputs: [
      {
        name: "order",
        type: "uint8",
        internalType: "enum IbcCoreChannelV1GlobalEnums.Order",
      },
      {
        name: "_connectionHops",
        type: "string[]",
        internalType: "string[]",
      },
      {
        name: "portId",
        type: "string",
        internalType: "string",
      },
      {
        name: "channelId",
        type: "string",
        internalType: "string",
      },
      {
        name: "counterpartyEndpoint",
        type: "tuple",
        internalType: "struct IbcCoreChannelV1Counterparty.Data",
        components: [
          {
            name: "port_id",
            type: "string",
            internalType: "string",
          },
          {
            name: "channel_id",
            type: "string",
            internalType: "string",
          },
        ],
      },
      {
        name: "version",
        type: "string",
        internalType: "string",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onChanOpenTry",
    inputs: [
      {
        name: "order",
        type: "uint8",
        internalType: "enum IbcCoreChannelV1GlobalEnums.Order",
      },
      {
        name: "_connectionHops",
        type: "string[]",
        internalType: "string[]",
      },
      {
        name: "portId",
        type: "string",
        internalType: "string",
      },
      {
        name: "channelId",
        type: "string",
        internalType: "string",
      },
      {
        name: "counterpartyEndpoint",
        type: "tuple",
        internalType: "struct IbcCoreChannelV1Counterparty.Data",
        components: [
          {
            name: "port_id",
            type: "string",
            internalType: "string",
          },
          {
            name: "channel_id",
            type: "string",
            internalType: "string",
          },
        ],
      },
      {
        name: "version",
        type: "string",
        internalType: "string",
      },
      {
        name: "counterpartyVersion",
        type: "string",
        internalType: "string",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onRecvPacket",
    inputs: [
      {
        name: "ibcPacket",
        type: "tuple",
        internalType: "struct IbcCoreChannelV1Packet.Data",
        components: [
          {
            name: "sequence",
            type: "uint64",
            internalType: "uint64",
          },
          {
            name: "source_port",
            type: "string",
            internalType: "string",
          },
          {
            name: "source_channel",
            type: "string",
            internalType: "string",
          },
          {
            name: "destination_port",
            type: "string",
            internalType: "string",
          },
          {
            name: "destination_channel",
            type: "string",
            internalType: "string",
          },
          {
            name: "data",
            type: "bytes",
            internalType: "bytes",
          },
          {
            name: "timeout_height",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64",
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64",
              },
            ],
          },
          {
            name: "timeout_timestamp",
            type: "uint64",
            internalType: "uint64",
          },
        ],
      },
      {
        name: "relayer",
        type: "address",
        internalType: "address",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onRecvPacketProcessing",
    inputs: [
      {
        name: "ibcPacket",
        type: "tuple",
        internalType: "struct IbcCoreChannelV1Packet.Data",
        components: [
          {
            name: "sequence",
            type: "uint64",
            internalType: "uint64",
          },
          {
            name: "source_port",
            type: "string",
            internalType: "string",
          },
          {
            name: "source_channel",
            type: "string",
            internalType: "string",
          },
          {
            name: "destination_port",
            type: "string",
            internalType: "string",
          },
          {
            name: "destination_channel",
            type: "string",
            internalType: "string",
          },
          {
            name: "data",
            type: "bytes",
            internalType: "bytes",
          },
          {
            name: "timeout_height",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64",
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64",
              },
            ],
          },
          {
            name: "timeout_timestamp",
            type: "uint64",
            internalType: "uint64",
          },
        ],
      },
      {
        name: "relayer",
        type: "address",
        internalType: "address",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onTimeoutPacket",
    inputs: [
      {
        name: "ibcPacket",
        type: "tuple",
        internalType: "struct IbcCoreChannelV1Packet.Data",
        components: [
          {
            name: "sequence",
            type: "uint64",
            internalType: "uint64",
          },
          {
            name: "source_port",
            type: "string",
            internalType: "string",
          },
          {
            name: "source_channel",
            type: "string",
            internalType: "string",
          },
          {
            name: "destination_port",
            type: "string",
            internalType: "string",
          },
          {
            name: "destination_channel",
            type: "string",
            internalType: "string",
          },
          {
            name: "data",
            type: "bytes",
            internalType: "bytes",
          },
          {
            name: "timeout_height",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64",
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64",
              },
            ],
          },
          {
            name: "timeout_timestamp",
            type: "uint64",
            internalType: "uint64",
          },
        ],
      },
      {
        name: "_relayer",
        type: "address",
        internalType: "address",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "send",
    inputs: [
      {
        name: "sourcePort",
        type: "string",
        internalType: "string",
      },
      {
        name: "sourceChannel",
        type: "string",
        internalType: "string",
      },
      {
        name: "receiver",
        type: "bytes",
        internalType: "bytes",
      },
      {
        name: "tokens",
        type: "tuple[]",
        internalType: "struct LocalToken[]",
        components: [
          {
            name: "denom",
            type: "address",
            internalType: "address",
          },
          {
            name: "amount",
            type: "uint128",
            internalType: "uint128",
          },
        ],
      },
      {
        name: "counterpartyTimeoutRevisionNumber",
        type: "uint64",
        internalType: "uint64",
      },
      {
        name: "counterpartyTimeoutRevisionHeight",
        type: "uint64",
        internalType: "uint64",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "event",
    name: "DenomCreated",
    inputs: [
      {
        name: "denom",
        type: "string",
        indexed: false,
        internalType: "string",
      },
      {
        name: "token",
        type: "address",
        indexed: false,
        internalType: "address",
      },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "Received",
    inputs: [
      {
        name: "sender",
        type: "string",
        indexed: false,
        internalType: "string",
      },
      {
        name: "receiver",
        type: "address",
        indexed: false,
        internalType: "address",
      },
      {
        name: "denom",
        type: "string",
        indexed: false,
        internalType: "string",
      },
      {
        name: "token",
        type: "address",
        indexed: false,
        internalType: "address",
      },
      {
        name: "amount",
        type: "uint256",
        indexed: false,
        internalType: "uint256",
      },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "Refunded",
    inputs: [
      {
        name: "sender",
        type: "address",
        indexed: false,
        internalType: "address",
      },
      {
        name: "receiver",
        type: "string",
        indexed: false,
        internalType: "string",
      },
      {
        name: "denom",
        type: "string",
        indexed: false,
        internalType: "string",
      },
      {
        name: "token",
        type: "address",
        indexed: false,
        internalType: "address",
      },
      {
        name: "amount",
        type: "uint256",
        indexed: false,
        internalType: "uint256",
      },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "Sent",
    inputs: [
      {
        name: "sender",
        type: "address",
        indexed: false,
        internalType: "address",
      },
      {
        name: "receiver",
        type: "string",
        indexed: false,
        internalType: "string",
      },
      {
        name: "denom",
        type: "string",
        indexed: false,
        internalType: "string",
      },
      {
        name: "token",
        type: "address",
        indexed: false,
        internalType: "address",
      },
      {
        name: "amount",
        type: "uint256",
        indexed: false,
        internalType: "uint256",
      },
    ],
    anonymous: false,
  },
  {
    type: "error",
    name: "AddressEmptyCode",
    inputs: [
      {
        name: "target",
        type: "address",
        internalType: "address",
      },
    ],
  },
  {
    type: "error",
    name: "AddressInsufficientBalance",
    inputs: [
      {
        name: "account",
        type: "address",
        internalType: "address",
      },
    ],
  },
  {
    type: "error",
    name: "ErrInvalidAcknowledgement",
    inputs: [],
  },
  {
    type: "error",
    name: "ErrInvalidBytesAddress",
    inputs: [],
  },
  {
    type: "error",
    name: "ErrInvalidCounterpartyProtocolVersion",
    inputs: [],
  },
  {
    type: "error",
    name: "ErrInvalidHexAddress",
    inputs: [],
  },
  {
    type: "error",
    name: "ErrInvalidProtocolOrdering",
    inputs: [],
  },
  {
    type: "error",
    name: "ErrInvalidProtocolVersion",
    inputs: [],
  },
  {
    type: "error",
    name: "ErrNotIBC",
    inputs: [],
  },
  {
    type: "error",
    name: "ErrUnauthorized",
    inputs: [],
  },
  {
    type: "error",
    name: "ErrUnstoppable",
    inputs: [],
  },
  {
    type: "error",
    name: "FailedInnerCall",
    inputs: [],
  },
  {
    type: "error",
    name: "SafeERC20FailedOperation",
    inputs: [
      {
        name: "token",
        type: "address",
        internalType: "address",
      },
    ],
  },
];
