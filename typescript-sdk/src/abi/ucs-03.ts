export const ucs03ZkgmAbi = [
  {
    type: "function",
    name: "ACK_EMPTY",
    inputs: [],
    outputs: [{ name: "", type: "bytes", internalType: "bytes" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "ACK_ERR_ONLYMAKER",
    inputs: [],
    outputs: [{ name: "", type: "bytes", internalType: "bytes" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "ACK_FAILURE",
    inputs: [],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "ACK_SUCCESS",
    inputs: [],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "FILL_TYPE_MARKETMAKER",
    inputs: [],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "FILL_TYPE_PROTOCOL",
    inputs: [],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "IBC_VERSION",
    inputs: [],
    outputs: [{ name: "", type: "bytes32", internalType: "bytes32" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "OP_BATCH",
    inputs: [],
    outputs: [{ name: "", type: "uint8", internalType: "uint8" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "OP_FORWARD",
    inputs: [],
    outputs: [{ name: "", type: "uint8", internalType: "uint8" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "OP_FUNGIBLE_ASSET_ORDER",
    inputs: [],
    outputs: [{ name: "", type: "uint8", internalType: "uint8" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "OP_MULTIPLEX",
    inputs: [],
    outputs: [{ name: "", type: "uint8", internalType: "uint8" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "ZKGM_VERSION_0",
    inputs: [],
    outputs: [{ name: "", type: "uint8", internalType: "uint8" }],
    stateMutability: "view"
  },
  { type: "error", name: "ErrBatchMustBeSync", inputs: [] },
  {
    type: "error",
    name: "ErrInfiniteGame",
    inputs: []
  },
  { type: "error", name: "ErrInvalidAmount", inputs: [] },
  {
    type: "error",
    name: "ErrInvalidAssetName",
    inputs: []
  },
  { type: "error", name: "ErrInvalidAssetOrigin", inputs: [] },
  {
    type: "error",
    name: "ErrInvalidAssetSymbol",
    inputs: []
  },
  { type: "error", name: "ErrInvalidBatchInstruction", inputs: [] },
  {
    type: "error",
    name: "ErrInvalidFillType",
    inputs: []
  },
  { type: "error", name: "ErrInvalidHops", inputs: [] },
  {
    type: "error",
    name: "ErrInvalidIBCVersion",
    inputs: []
  },
  { type: "error", name: "ErrInvalidMultiplexSender", inputs: [] },
  {
    type: "error",
    name: "ErrOnlyMaker",
    inputs: []
  },
  { type: "error", name: "ErrUnauthorized", inputs: [] },
  {
    type: "error",
    name: "ErrUnimplemented",
    inputs: []
  },
  { type: "error", name: "ErrUnknownOpcode", inputs: [] },
  {
    type: "error",
    name: "ErrUnsupportedVersion",
    inputs: []
  },
  { type: "constructor", inputs: [], stateMutability: "nonpayable" },
  {
    type: "function",
    name: "UPGRADE_INTERFACE_VERSION",
    inputs: [],
    outputs: [{ name: "", type: "string", internalType: "string" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "call",
    inputs: [
      { name: "channelId", type: "uint32", internalType: "uint32" },
      {
        name: "contractAddress",
        type: "bytes",
        internalType: "bytes"
      },
      { name: "contractCalldata", type: "bytes", internalType: "bytes" },
      {
        name: "timeoutHeight",
        type: "uint64",
        internalType: "uint64"
      },
      { name: "timeoutTimestamp", type: "uint64", internalType: "uint64" },
      {
        name: "salt",
        type: "bytes32",
        internalType: "bytes32"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "channelBalance",
    inputs: [
      { name: "", type: "uint32", internalType: "uint32" },
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "execute",
    inputs: [
      {
        name: "ibcPacket",
        type: "tuple",
        internalType: "struct IBCPacket",
        components: [
          {
            name: "sourceChannelId",
            type: "uint32",
            internalType: "uint32"
          },
          { name: "destinationChannelId", type: "uint32", internalType: "uint32" },
          {
            name: "data",
            type: "bytes",
            internalType: "bytes"
          },
          { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
          {
            name: "timeoutTimestamp",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      },
      { name: "relayer", type: "address", internalType: "address" },
      {
        name: "relayerMsg",
        type: "bytes",
        internalType: "bytes"
      }
    ],
    outputs: [{ name: "", type: "bytes", internalType: "bytes" }],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "ibcAddress",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "address" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "ibcHandler",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "contract IIBCPacket" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "inFlightPacket",
    inputs: [{ name: "", type: "bytes32", internalType: "bytes32" }],
    outputs: [
      { name: "sourceChannelId", type: "uint32", internalType: "uint32" },
      {
        name: "destinationChannelId",
        type: "uint32",
        internalType: "uint32"
      },
      { name: "data", type: "bytes", internalType: "bytes" },
      {
        name: "timeoutHeight",
        type: "uint64",
        internalType: "uint64"
      },
      { name: "timeoutTimestamp", type: "uint64", internalType: "uint64" }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "initialize",
    inputs: [
      { name: "_ibcHandler", type: "address", internalType: "contract IIBCPacket" },
      {
        name: "admin",
        type: "address",
        internalType: "address"
      },
      { name: "_weth", type: "address", internalType: "contract IWETH" }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "onAcknowledgementPacket",
    inputs: [
      {
        name: "ibcPacket",
        type: "tuple",
        internalType: "struct IBCPacket",
        components: [
          {
            name: "sourceChannelId",
            type: "uint32",
            internalType: "uint32"
          },
          { name: "destinationChannelId", type: "uint32", internalType: "uint32" },
          {
            name: "data",
            type: "bytes",
            internalType: "bytes"
          },
          { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
          {
            name: "timeoutTimestamp",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      },
      { name: "ack", type: "bytes", internalType: "bytes" },
      {
        name: "relayer",
        type: "address",
        internalType: "address"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "onChanCloseConfirm",
    inputs: [
      { name: "", type: "uint32", internalType: "uint32" },
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "onChanCloseInit",
    inputs: [
      { name: "", type: "uint32", internalType: "uint32" },
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "onChanOpenAck",
    inputs: [
      { name: "channelId", type: "uint32", internalType: "uint32" },
      {
        name: "",
        type: "uint32",
        internalType: "uint32"
      },
      { name: "", type: "string", internalType: "string" },
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "onChanOpenConfirm",
    inputs: [
      { name: "channelId", type: "uint32", internalType: "uint32" },
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "onChanOpenInit",
    inputs: [
      { name: "", type: "uint32", internalType: "uint32" },
      {
        name: "",
        type: "uint32",
        internalType: "uint32"
      },
      { name: "version", type: "string", internalType: "string" },
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "onChanOpenTry",
    inputs: [
      { name: "", type: "uint32", internalType: "uint32" },
      {
        name: "",
        type: "uint32",
        internalType: "uint32"
      },
      { name: "", type: "uint32", internalType: "uint32" },
      {
        name: "version",
        type: "string",
        internalType: "string"
      },
      { name: "counterpartyVersion", type: "string", internalType: "string" },
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "onRecvIntentPacket",
    inputs: [
      {
        name: "",
        type: "tuple",
        internalType: "struct IBCPacket",
        components: [
          {
            name: "sourceChannelId",
            type: "uint32",
            internalType: "uint32"
          },
          { name: "destinationChannelId", type: "uint32", internalType: "uint32" },
          {
            name: "data",
            type: "bytes",
            internalType: "bytes"
          },
          { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
          {
            name: "timeoutTimestamp",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      },
      { name: "", type: "address", internalType: "address" },
      {
        name: "",
        type: "bytes",
        internalType: "bytes"
      }
    ],
    outputs: [{ name: "", type: "bytes", internalType: "bytes" }],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "onRecvPacket",
    inputs: [
      {
        name: "operand",
        type: "tuple",
        internalType: "struct IBCPacket",
        components: [
          {
            name: "sourceChannelId",
            type: "uint32",
            internalType: "uint32"
          },
          { name: "destinationChannelId", type: "uint32", internalType: "uint32" },
          {
            name: "data",
            type: "bytes",
            internalType: "bytes"
          },
          { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
          {
            name: "timeoutTimestamp",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      },
      { name: "relayer", type: "address", internalType: "address" },
      {
        name: "relayerMsg",
        type: "bytes",
        internalType: "bytes"
      }
    ],
    outputs: [{ name: "", type: "bytes", internalType: "bytes" }],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "onTimeoutPacket",
    inputs: [
      {
        name: "ibcPacket",
        type: "tuple",
        internalType: "struct IBCPacket",
        components: [
          {
            name: "sourceChannelId",
            type: "uint32",
            internalType: "uint32"
          },
          { name: "destinationChannelId", type: "uint32", internalType: "uint32" },
          {
            name: "data",
            type: "bytes",
            internalType: "bytes"
          },
          { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
          {
            name: "timeoutTimestamp",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      },
      { name: "relayer", type: "address", internalType: "address" }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "owner",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "address" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "paused",
    inputs: [],
    outputs: [{ name: "", type: "bool", internalType: "bool" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "predictWrappedToken",
    inputs: [
      { name: "path", type: "uint256", internalType: "uint256" },
      {
        name: "channel",
        type: "uint32",
        internalType: "uint32"
      },
      { name: "token", type: "bytes", internalType: "bytes" }
    ],
    outputs: [
      { name: "", type: "address", internalType: "address" },
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "proxiableUUID",
    inputs: [],
    outputs: [{ name: "", type: "bytes32", internalType: "bytes32" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "renounceOwnership",
    inputs: [],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "send",
    inputs: [
      { name: "channelId", type: "uint32", internalType: "uint32" },
      {
        name: "timeoutHeight",
        type: "uint64",
        internalType: "uint64"
      },
      { name: "timeoutTimestamp", type: "uint64", internalType: "uint64" },
      {
        name: "salt",
        type: "bytes32",
        internalType: "bytes32"
      },
      {
        name: "instruction",
        type: "tuple",
        internalType: "struct Instruction",
        components: [
          { name: "version", type: "uint8", internalType: "uint8" },
          {
            name: "opcode",
            type: "uint8",
            internalType: "uint8"
          },
          { name: "operand", type: "bytes", internalType: "bytes" }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "setWeth",
    inputs: [{ name: "_weth", type: "address", internalType: "contract IWETH" }],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "tokenOrigin",
    inputs: [{ name: "", type: "address", internalType: "address" }],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "transfer",
    inputs: [
      { name: "channelId", type: "uint32", internalType: "uint32" },
      {
        name: "receiver",
        type: "bytes",
        internalType: "bytes"
      },
      { name: "baseToken", type: "address", internalType: "address" },
      {
        name: "baseAmount",
        type: "uint256",
        internalType: "uint256"
      },
      { name: "quoteToken", type: "bytes", internalType: "bytes" },
      {
        name: "quoteAmount",
        type: "uint256",
        internalType: "uint256"
      },
      { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
      {
        name: "timeoutTimestamp",
        type: "uint64",
        internalType: "uint64"
      },
      { name: "salt", type: "bytes32", internalType: "bytes32" }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "transferAndCall",
    inputs: [
      { name: "channelId", type: "uint32", internalType: "uint32" },
      {
        name: "receiver",
        type: "bytes",
        internalType: "bytes"
      },
      { name: "baseToken", type: "address", internalType: "address" },
      {
        name: "baseAmount",
        type: "uint256",
        internalType: "uint256"
      },
      { name: "quoteToken", type: "bytes", internalType: "bytes" },
      {
        name: "quoteAmount",
        type: "uint256",
        internalType: "uint256"
      },
      { name: "contractAddress", type: "bytes", internalType: "bytes" },
      {
        name: "contractCalldata",
        type: "bytes",
        internalType: "bytes"
      },
      { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
      {
        name: "timeoutTimestamp",
        type: "uint64",
        internalType: "uint64"
      },
      { name: "salt", type: "bytes32", internalType: "bytes32" }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "transferOwnership",
    inputs: [{ name: "newOwner", type: "address", internalType: "address" }],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "transferV2",
    inputs: [
      { name: "channelId", type: "uint32", internalType: "uint32" },
      {
        name: "receiver",
        type: "bytes",
        internalType: "bytes"
      },
      { name: "baseToken", type: "address", internalType: "address" },
      {
        name: "baseAmount",
        type: "uint256",
        internalType: "uint256"
      },
      { name: "quoteToken", type: "bytes", internalType: "bytes" },
      {
        name: "quoteAmount",
        type: "uint256",
        internalType: "uint256"
      },
      { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
      {
        name: "timeoutTimestamp",
        type: "uint64",
        internalType: "uint64"
      },
      { name: "salt", type: "bytes32", internalType: "bytes32" },
      {
        name: "wethQuoteToken",
        type: "bytes",
        internalType: "bytes"
      }
    ],
    outputs: [],
    stateMutability: "payable"
  },
  {
    type: "function",
    name: "upgradeToAndCall",
    inputs: [
      { name: "newImplementation", type: "address", internalType: "address" },
      {
        name: "data",
        type: "bytes",
        internalType: "bytes"
      }
    ],
    outputs: [],
    stateMutability: "payable"
  },
  {
    type: "function",
    name: "weth",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "contract IWETH" }],
    stateMutability: "view"
  },
  {
    type: "event",
    name: "Initialized",
    inputs: [{ name: "version", type: "uint64", indexed: false, internalType: "uint64" }],
    anonymous: false
  },
  {
    type: "event",
    name: "OwnershipTransferred",
    inputs: [
      {
        name: "previousOwner",
        type: "address",
        indexed: true,
        internalType: "address"
      },
      { name: "newOwner", type: "address", indexed: true, internalType: "address" }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "Paused",
    inputs: [{ name: "account", type: "address", indexed: false, internalType: "address" }],
    anonymous: false
  },
  {
    type: "event",
    name: "Unpaused",
    inputs: [{ name: "account", type: "address", indexed: false, internalType: "address" }],
    anonymous: false
  },
  {
    type: "event",
    name: "Upgraded",
    inputs: [{ name: "implementation", type: "address", indexed: true, internalType: "address" }],
    anonymous: false
  },
  {
    type: "error",
    name: "AddressEmptyCode",
    inputs: [{ name: "target", type: "address", internalType: "address" }]
  },
  {
    type: "error",
    name: "AddressInsufficientBalance",
    inputs: [{ name: "account", type: "address", internalType: "address" }]
  },
  {
    type: "error",
    name: "ERC1967InvalidImplementation",
    inputs: [{ name: "implementation", type: "address", internalType: "address" }]
  },
  { type: "error", name: "ERC1967NonPayable", inputs: [] },
  {
    type: "error",
    name: "EnforcedPause",
    inputs: []
  },
  { type: "error", name: "ErrBatchMustBeSync", inputs: [] },
  {
    type: "error",
    name: "ErrInfiniteGame",
    inputs: []
  },
  { type: "error", name: "ErrInvalidAmount", inputs: [] },
  {
    type: "error",
    name: "ErrInvalidAssetName",
    inputs: []
  },
  { type: "error", name: "ErrInvalidAssetOrigin", inputs: [] },
  {
    type: "error",
    name: "ErrInvalidAssetSymbol",
    inputs: []
  },
  { type: "error", name: "ErrInvalidBatchInstruction", inputs: [] },
  {
    type: "error",
    name: "ErrInvalidFillType",
    inputs: []
  },
  { type: "error", name: "ErrInvalidHops", inputs: [] },
  {
    type: "error",
    name: "ErrInvalidIBCVersion",
    inputs: []
  },
  { type: "error", name: "ErrInvalidMultiplexSender", inputs: [] },
  {
    type: "error",
    name: "ErrNotIBC",
    inputs: []
  },
  { type: "error", name: "ErrNotImplemented", inputs: [] },
  {
    type: "error",
    name: "ErrOnlyMaker",
    inputs: []
  },
  { type: "error", name: "ErrUnauthorized", inputs: [] },
  {
    type: "error",
    name: "ErrUnimplemented",
    inputs: []
  },
  { type: "error", name: "ErrUnknownOpcode", inputs: [] },
  {
    type: "error",
    name: "ErrUnsupportedVersion",
    inputs: []
  },
  { type: "error", name: "ExpectedPause", inputs: [] },
  {
    type: "error",
    name: "FailedInnerCall",
    inputs: []
  },
  { type: "error", name: "InvalidInitialization", inputs: [] },
  {
    type: "error",
    name: "NotInitializing",
    inputs: []
  },
  {
    type: "error",
    name: "OwnableInvalidOwner",
    inputs: [{ name: "owner", type: "address", internalType: "address" }]
  },
  {
    type: "error",
    name: "OwnableUnauthorizedAccount",
    inputs: [{ name: "account", type: "address", internalType: "address" }]
  },
  {
    type: "error",
    name: "SafeERC20FailedOperation",
    inputs: [{ name: "token", type: "address", internalType: "address" }]
  },
  { type: "error", name: "UUPSUnauthorizedCallContext", inputs: [] },
  {
    type: "error",
    name: "UUPSUnsupportedProxiableUUID",
    inputs: [{ name: "slot", type: "bytes32", internalType: "bytes32" }]
  }
]
