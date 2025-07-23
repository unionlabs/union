/**
 * This module handles interaction with the [UCS03](https://docs.union.build/ucs/03/) protocol.
 *
 * @since 2.0.0
 */

import * as A from "effect/Array"
import * as Data from "effect/Data"
import { constant } from "effect/Function"
import * as S from "effect/Schema"
import { encodeAbiParameters } from "viem"
import { Hex, HexChecksum } from "./schema/hex.js"

/**
 * Contract ABI
 *
 * @category abis
 * @since 2.0.0
 */
export const Abi = [
  {
    type: "function",
    name: "ensureExported",
    inputs: [{
      name: "",
      type: "tuple",
      internalType: "struct ZkgmPacket",
      components: [{ name: "salt", type: "bytes32", internalType: "bytes32" }, {
        name: "path",
        type: "uint256",
        internalType: "uint256",
      }, {
        name: "instruction",
        type: "tuple",
        internalType: "struct Instruction",
        components: [{ name: "version", type: "uint8", internalType: "uint8" }, {
          name: "opcode",
          type: "uint8",
          internalType: "uint8",
        }, { name: "operand", type: "bytes", internalType: "bytes" }],
      }],
    }, {
      name: "",
      type: "tuple",
      internalType: "struct Instruction",
      components: [{ name: "version", type: "uint8", internalType: "uint8" }, {
        name: "opcode",
        type: "uint8",
        internalType: "uint8",
      }, { name: "operand", type: "bytes", internalType: "bytes" }],
    }, {
      name: "",
      type: "tuple",
      internalType: "struct Forward",
      components: [
        { name: "path", type: "uint256", internalType: "uint256" },
        { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
        { name: "timeoutTimestamp", type: "uint64", internalType: "uint64" },
        {
          name: "instruction",
          type: "tuple",
          internalType: "struct Instruction",
          components: [{ name: "version", type: "uint8", internalType: "uint8" }, {
            name: "opcode",
            type: "uint8",
            internalType: "uint8",
          }, { name: "operand", type: "bytes", internalType: "bytes" }],
        },
      ],
    }, {
      name: "",
      type: "tuple",
      internalType: "struct Multiplex",
      components: [
        { name: "sender", type: "bytes", internalType: "bytes" },
        { name: "eureka", type: "bool", internalType: "bool" },
        { name: "contractAddress", type: "bytes", internalType: "bytes" },
        { name: "contractCalldata", type: "bytes", internalType: "bytes" },
      ],
    }, {
      name: "",
      type: "tuple",
      internalType: "struct Batch",
      components: [{
        name: "instructions",
        type: "tuple[]",
        internalType: "struct Instruction[]",
        components: [{ name: "version", type: "uint8", internalType: "uint8" }, {
          name: "opcode",
          type: "uint8",
          internalType: "uint8",
        }, { name: "operand", type: "bytes", internalType: "bytes" }],
      }],
    }, {
      name: "",
      type: "tuple",
      internalType: "struct FungibleAssetOrder",
      components: [
        { name: "sender", type: "bytes", internalType: "bytes" },
        { name: "receiver", type: "bytes", internalType: "bytes" },
        { name: "baseToken", type: "bytes", internalType: "bytes" },
        { name: "baseAmount", type: "uint256", internalType: "uint256" },
        { name: "baseTokenSymbol", type: "string", internalType: "string" },
        { name: "baseTokenName", type: "string", internalType: "string" },
        { name: "baseTokenDecimals", type: "uint8", internalType: "uint8" },
        { name: "baseTokenPath", type: "uint256", internalType: "uint256" },
        { name: "quoteToken", type: "bytes", internalType: "bytes" },
        { name: "quoteAmount", type: "uint256", internalType: "uint256" },
      ],
    }, {
      name: "",
      type: "tuple",
      internalType: "struct Ack",
      components: [{ name: "tag", type: "uint256", internalType: "uint256" }, {
        name: "innerAck",
        type: "bytes",
        internalType: "bytes",
      }],
    }, {
      name: "",
      type: "tuple",
      internalType: "struct BatchAck",
      components: [{ name: "acknowledgements", type: "bytes[]", internalType: "bytes[]" }],
    }, {
      name: "",
      type: "tuple",
      internalType: "struct FungibleAssetOrderAck",
      components: [{ name: "fillType", type: "uint256", internalType: "uint256" }, {
        name: "marketMaker",
        type: "bytes",
        internalType: "bytes",
      }],
    }],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "constructor",
    inputs: [
      { name: "_ibcHandler", type: "address", internalType: "contract IIBCModulePacket" },
      { name: "_sendImpl", type: "address", internalType: "contract UCS03ZkgmSendImpl" },
      { name: "_stakeImpl", type: "address", internalType: "contract UCS03ZkgmStakeImpl" },
      {
        name: "_faoImpl",
        type: "address",
        internalType: "contract UCS03ZkgmFungibleAssetOrderImpl",
      },
    ],
    stateMutability: "nonpayable",
  },
  { type: "receive", stateMutability: "payable" },
  {
    type: "function",
    name: "EXEC_MIN_GAS",
    inputs: [],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "FAO_IMPL",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "address" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "IBC_HANDLER",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "contract IIBCModulePacket" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "SEND_IMPL",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "address" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "STAKE_IMPL",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "address" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "UPGRADE_INTERFACE_VERSION",
    inputs: [],
    outputs: [{ name: "", type: "string", internalType: "string" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "authority",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "address" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "channelBalance",
    inputs: [{ name: "", type: "uint32", internalType: "uint32" }, {
      name: "",
      type: "uint256",
      internalType: "uint256",
    }, { name: "", type: "address", internalType: "address" }],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "channelBalanceV2",
    inputs: [
      { name: "", type: "uint32", internalType: "uint32" },
      { name: "", type: "uint256", internalType: "uint256" },
      { name: "", type: "address", internalType: "address" },
      { name: "", type: "bytes32", internalType: "bytes32" },
    ],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "channelGovernanceToken",
    inputs: [{ name: "", type: "uint32", internalType: "uint32" }],
    outputs: [{ name: "unwrappedToken", type: "bytes", internalType: "bytes" }, {
      name: "metadataImage",
      type: "bytes32",
      internalType: "bytes32",
    }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "execute",
    inputs: [
      { name: "caller", type: "address", internalType: "address" },
      {
        name: "ibcPacket",
        type: "tuple",
        internalType: "struct IBCPacket",
        components: [
          { name: "sourceChannelId", type: "uint32", internalType: "uint32" },
          { name: "destinationChannelId", type: "uint32", internalType: "uint32" },
          { name: "data", type: "bytes", internalType: "bytes" },
          { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
          { name: "timeoutTimestamp", type: "uint64", internalType: "uint64" },
        ],
      },
      { name: "relayer", type: "address", internalType: "address" },
      { name: "relayerMsg", type: "bytes", internalType: "bytes" },
      { name: "intent", type: "bool", internalType: "bool" },
    ],
    outputs: [{ name: "", type: "bytes", internalType: "bytes" }],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "getBucket",
    inputs: [{ name: "token", type: "address", internalType: "address" }],
    outputs: [{
      name: "",
      type: "tuple",
      internalType: "struct TokenBucket.Bucket",
      components: [
        { name: "capacity", type: "uint256", internalType: "uint256" },
        { name: "available", type: "uint256", internalType: "uint256" },
        { name: "refillRate", type: "uint256", internalType: "uint256" },
        { name: "lastRefill", type: "uint256", internalType: "uint256" },
      ],
    }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "getGovernanceToken",
    inputs: [{ name: "channelId", type: "uint32", internalType: "uint32" }],
    outputs: [{ name: "", type: "address", internalType: "contract ZkgmERC20" }, {
      name: "",
      type: "tuple",
      internalType: "struct GovernanceToken",
      components: [{ name: "unwrappedToken", type: "bytes", internalType: "bytes" }, {
        name: "metadataImage",
        type: "bytes32",
        internalType: "bytes32",
      }],
    }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "gitRev",
    inputs: [],
    outputs: [{ name: "", type: "string", internalType: "string" }],
    stateMutability: "pure",
  },
  {
    type: "function",
    name: "ibcAddress",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "address" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "inFlightPacket",
    inputs: [{ name: "", type: "bytes32", internalType: "bytes32" }],
    outputs: [
      { name: "sourceChannelId", type: "uint32", internalType: "uint32" },
      { name: "destinationChannelId", type: "uint32", internalType: "uint32" },
      { name: "data", type: "bytes", internalType: "bytes" },
      { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
      { name: "timeoutTimestamp", type: "uint64", internalType: "uint64" },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "initialize",
    inputs: [{ name: "_authority", type: "address", internalType: "address" }],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "isConsumingScheduledOp",
    inputs: [],
    outputs: [{ name: "", type: "bytes4", internalType: "bytes4" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "metadataImageOf",
    inputs: [{ name: "", type: "address", internalType: "address" }],
    outputs: [{ name: "", type: "bytes32", internalType: "bytes32" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "onAcknowledgementPacket",
    inputs: [
      { name: "caller", type: "address", internalType: "address" },
      {
        name: "ibcPacket",
        type: "tuple",
        internalType: "struct IBCPacket",
        components: [
          { name: "sourceChannelId", type: "uint32", internalType: "uint32" },
          { name: "destinationChannelId", type: "uint32", internalType: "uint32" },
          { name: "data", type: "bytes", internalType: "bytes" },
          { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
          { name: "timeoutTimestamp", type: "uint64", internalType: "uint64" },
        ],
      },
      { name: "ack", type: "bytes", internalType: "bytes" },
      { name: "relayer", type: "address", internalType: "address" },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onChanCloseConfirm",
    inputs: [{ name: "", type: "address", internalType: "address" }, {
      name: "",
      type: "uint32",
      internalType: "uint32",
    }, { name: "", type: "address", internalType: "address" }],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onChanCloseInit",
    inputs: [{ name: "", type: "address", internalType: "address" }, {
      name: "",
      type: "uint32",
      internalType: "uint32",
    }, { name: "", type: "address", internalType: "address" }],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onChanOpenAck",
    inputs: [
      { name: "", type: "address", internalType: "address" },
      { name: "channelId", type: "uint32", internalType: "uint32" },
      { name: "", type: "uint32", internalType: "uint32" },
      { name: "counterpartyVersion", type: "string", internalType: "string" },
      { name: "", type: "address", internalType: "address" },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onChanOpenConfirm",
    inputs: [{ name: "", type: "address", internalType: "address" }, {
      name: "channelId",
      type: "uint32",
      internalType: "uint32",
    }, { name: "", type: "address", internalType: "address" }],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onChanOpenInit",
    inputs: [
      { name: "", type: "address", internalType: "address" },
      { name: "", type: "uint32", internalType: "uint32" },
      { name: "", type: "uint32", internalType: "uint32" },
      { name: "version", type: "string", internalType: "string" },
      { name: "", type: "address", internalType: "address" },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onChanOpenTry",
    inputs: [
      { name: "", type: "address", internalType: "address" },
      { name: "", type: "uint32", internalType: "uint32" },
      { name: "", type: "uint32", internalType: "uint32" },
      { name: "", type: "uint32", internalType: "uint32" },
      { name: "version", type: "string", internalType: "string" },
      { name: "counterpartyVersion", type: "string", internalType: "string" },
      { name: "", type: "address", internalType: "address" },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onRecvIntentPacket",
    inputs: [
      { name: "caller", type: "address", internalType: "address" },
      {
        name: "packet",
        type: "tuple",
        internalType: "struct IBCPacket",
        components: [
          { name: "sourceChannelId", type: "uint32", internalType: "uint32" },
          { name: "destinationChannelId", type: "uint32", internalType: "uint32" },
          { name: "data", type: "bytes", internalType: "bytes" },
          { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
          { name: "timeoutTimestamp", type: "uint64", internalType: "uint64" },
        ],
      },
      { name: "relayer", type: "address", internalType: "address" },
      { name: "relayerMsg", type: "bytes", internalType: "bytes" },
    ],
    outputs: [{ name: "", type: "bytes", internalType: "bytes" }],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onRecvPacket",
    inputs: [
      { name: "caller", type: "address", internalType: "address" },
      {
        name: "packet",
        type: "tuple",
        internalType: "struct IBCPacket",
        components: [
          { name: "sourceChannelId", type: "uint32", internalType: "uint32" },
          { name: "destinationChannelId", type: "uint32", internalType: "uint32" },
          { name: "data", type: "bytes", internalType: "bytes" },
          { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
          { name: "timeoutTimestamp", type: "uint64", internalType: "uint64" },
        ],
      },
      { name: "relayer", type: "address", internalType: "address" },
      { name: "relayerMsg", type: "bytes", internalType: "bytes" },
    ],
    outputs: [{ name: "", type: "bytes", internalType: "bytes" }],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "onTimeoutPacket",
    inputs: [{ name: "caller", type: "address", internalType: "address" }, {
      name: "ibcPacket",
      type: "tuple",
      internalType: "struct IBCPacket",
      components: [
        { name: "sourceChannelId", type: "uint32", internalType: "uint32" },
        { name: "destinationChannelId", type: "uint32", internalType: "uint32" },
        { name: "data", type: "bytes", internalType: "bytes" },
        { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
        { name: "timeoutTimestamp", type: "uint64", internalType: "uint64" },
      ],
    }, { name: "relayer", type: "address", internalType: "address" }],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "overwriteGovernanceToken",
    inputs: [{ name: "channelId", type: "uint32", internalType: "uint32" }, {
      name: "governanceToken",
      type: "tuple",
      internalType: "struct GovernanceToken",
      components: [{ name: "unwrappedToken", type: "bytes", internalType: "bytes" }, {
        name: "metadataImage",
        type: "bytes32",
        internalType: "bytes32",
      }],
    }],
    outputs: [],
    stateMutability: "nonpayable",
  },
  { type: "function", name: "pause", inputs: [], outputs: [], stateMutability: "nonpayable" },
  {
    type: "function",
    name: "paused",
    inputs: [],
    outputs: [{ name: "", type: "bool", internalType: "bool" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "predictStakeManagerAddress",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "contract ZkgmERC721" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "predictWrappedToken",
    inputs: [{ name: "path", type: "uint256", internalType: "uint256" }, {
      name: "channel",
      type: "uint32",
      internalType: "uint32",
    }, { name: "token", type: "bytes", internalType: "bytes" }],
    outputs: [{ name: "", type: "address", internalType: "address" }, {
      name: "",
      type: "bytes32",
      internalType: "bytes32",
    }],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "predictWrappedTokenFromMetadataImageV2",
    inputs: [
      { name: "path", type: "uint256", internalType: "uint256" },
      { name: "channel", type: "uint32", internalType: "uint32" },
      { name: "token", type: "bytes", internalType: "bytes" },
      { name: "metadataImage", type: "bytes32", internalType: "bytes32" },
    ],
    outputs: [{ name: "", type: "address", internalType: "address" }, {
      name: "",
      type: "bytes32",
      internalType: "bytes32",
    }],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "predictWrappedTokenV2",
    inputs: [
      { name: "path", type: "uint256", internalType: "uint256" },
      { name: "channel", type: "uint32", internalType: "uint32" },
      { name: "token", type: "bytes", internalType: "bytes" },
      {
        name: "metadata",
        type: "tuple",
        internalType: "struct FungibleAssetMetadata",
        components: [{ name: "implementation", type: "bytes", internalType: "bytes" }, {
          name: "initializer",
          type: "bytes",
          internalType: "bytes",
        }],
      },
    ],
    outputs: [{ name: "", type: "address", internalType: "address" }, {
      name: "",
      type: "bytes32",
      internalType: "bytes32",
    }],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "proxiableUUID",
    inputs: [],
    outputs: [{ name: "", type: "bytes32", internalType: "bytes32" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "registerGovernanceToken",
    inputs: [{ name: "channelId", type: "uint32", internalType: "uint32" }, {
      name: "governanceToken",
      type: "tuple",
      internalType: "struct GovernanceToken",
      components: [{ name: "unwrappedToken", type: "bytes", internalType: "bytes" }, {
        name: "metadataImage",
        type: "bytes32",
        internalType: "bytes32",
      }],
    }],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "send",
    inputs: [
      { name: "channelId", type: "uint32", internalType: "uint32" },
      { name: "timeoutHeight", type: "uint64", internalType: "uint64" },
      { name: "timeoutTimestamp", type: "uint64", internalType: "uint64" },
      { name: "salt", type: "bytes32", internalType: "bytes32" },
      {
        name: "instruction",
        type: "tuple",
        internalType: "struct Instruction",
        components: [{ name: "version", type: "uint8", internalType: "uint8" }, {
          name: "opcode",
          type: "uint8",
          internalType: "uint8",
        }, { name: "operand", type: "bytes", internalType: "bytes" }],
      },
    ],
    outputs: [],
    stateMutability: "payable",
  },
  {
    type: "function",
    name: "setAuthority",
    inputs: [{ name: "newAuthority", type: "address", internalType: "address" }],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "setBucketConfig",
    inputs: [
      { name: "token", type: "address", internalType: "address" },
      { name: "capacity", type: "uint256", internalType: "uint256" },
      { name: "refillRate", type: "uint256", internalType: "uint256" },
      { name: "reset", type: "bool", internalType: "bool" },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "stakes",
    inputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    outputs: [
      { name: "state", type: "uint8", internalType: "enum ZkgmStakeState" },
      { name: "channelId", type: "uint32", internalType: "uint32" },
      { name: "validator", type: "bytes", internalType: "bytes" },
      { name: "amount", type: "uint256", internalType: "uint256" },
      { name: "unstakingCompletion", type: "uint256", internalType: "uint256" },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "tokenOrigin",
    inputs: [{ name: "", type: "address", internalType: "address" }],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view",
  },
  { type: "function", name: "unpause", inputs: [], outputs: [], stateMutability: "nonpayable" },
  {
    type: "function",
    name: "upgradeToAndCall",
    inputs: [{ name: "newImplementation", type: "address", internalType: "address" }, {
      name: "data",
      type: "bytes",
      internalType: "bytes",
    }],
    outputs: [],
    stateMutability: "payable",
  },
  {
    type: "event",
    name: "AuthorityUpdated",
    inputs: [{ name: "authority", type: "address", indexed: false, internalType: "address" }],
    anonymous: false,
  },
  {
    type: "event",
    name: "Initialized",
    inputs: [{ name: "version", type: "uint64", indexed: false, internalType: "uint64" }],
    anonymous: false,
  },
  {
    type: "event",
    name: "Paused",
    inputs: [{ name: "account", type: "address", indexed: false, internalType: "address" }],
    anonymous: false,
  },
  {
    type: "event",
    name: "TokenBucketUpdate",
    inputs: [{ name: "token", type: "address", indexed: true, internalType: "address" }, {
      name: "capacity",
      type: "uint256",
      indexed: false,
      internalType: "uint256",
    }, { name: "refillRate", type: "uint256", indexed: false, internalType: "uint256" }],
    anonymous: false,
  },
  {
    type: "event",
    name: "Unpaused",
    inputs: [{ name: "account", type: "address", indexed: false, internalType: "address" }],
    anonymous: false,
  },
  {
    type: "event",
    name: "Upgraded",
    inputs: [{ name: "implementation", type: "address", indexed: true, internalType: "address" }],
    anonymous: false,
  },
  {
    type: "error",
    name: "AccessManagedInvalidAuthority",
    inputs: [{ name: "authority", type: "address", internalType: "address" }],
  },
  {
    type: "error",
    name: "AccessManagedRequiredDelay",
    inputs: [{ name: "caller", type: "address", internalType: "address" }, {
      name: "delay",
      type: "uint32",
      internalType: "uint32",
    }],
  },
  {
    type: "error",
    name: "AccessManagedUnauthorized",
    inputs: [{ name: "caller", type: "address", internalType: "address" }],
  },
  {
    type: "error",
    name: "AddressEmptyCode",
    inputs: [{ name: "target", type: "address", internalType: "address" }],
  },
  {
    type: "error",
    name: "ERC1967InvalidImplementation",
    inputs: [{ name: "implementation", type: "address", internalType: "address" }],
  },
  { type: "error", name: "ERC1967NonPayable", inputs: [] },
  { type: "error", name: "EnforcedPause", inputs: [] },
  { type: "error", name: "ErrAsyncMultiplexUnsupported", inputs: [] },
  { type: "error", name: "ErrBatchMustBeSync", inputs: [] },
  { type: "error", name: "ErrChannelGovernanceTokenAlreadySet", inputs: [] },
  { type: "error", name: "ErrChannelGovernanceTokenNotSet", inputs: [] },
  { type: "error", name: "ErrInfiniteGame", inputs: [] },
  { type: "error", name: "ErrInvalidBatchInstruction", inputs: [] },
  { type: "error", name: "ErrInvalidForwardDestinationChannelId", inputs: [] },
  { type: "error", name: "ErrInvalidForwardInstruction", inputs: [] },
  { type: "error", name: "ErrInvalidHops", inputs: [] },
  { type: "error", name: "ErrInvalidIBCVersion", inputs: [] },
  { type: "error", name: "ErrNotIBC", inputs: [] },
  { type: "error", name: "ErrOnlyMaker", inputs: [] },
  { type: "error", name: "ErrTokenBucketRateLimitExceeded", inputs: [] },
  { type: "error", name: "ErrTokenBucketZeroCapacity", inputs: [] },
  { type: "error", name: "ErrTokenBucketZeroRefillRate", inputs: [] },
  { type: "error", name: "ErrUnauthorized", inputs: [] },
  { type: "error", name: "ErrUnknownOpcode", inputs: [] },
  { type: "error", name: "ExpectedPause", inputs: [] },
  { type: "error", name: "FailedCall", inputs: [] },
  { type: "error", name: "InvalidInitialization", inputs: [] },
  { type: "error", name: "NotInitializing", inputs: [] },
  { type: "error", name: "UUPSUnauthorizedCallContext", inputs: [] },
  {
    type: "error",
    name: "UUPSUnsupportedProxiableUUID",
    inputs: [{ name: "slot", type: "bytes32", internalType: "bytes32" }],
  },
] as const

type EnsureExported = Extract<(typeof Abi)[number], { name: "ensureExported" }> extends infer R
  ? [R] extends [never] ? ["Abi does not contain a function named `ensureExported`"]
  : R
  : never

type Input = EnsureExported["inputs"][number]

type StructName = Input extends { internalType: `struct ${infer N extends string}` } ? N : never

type StructMap = {
  [N in StructName]: Extract<Input, { internalType: `struct ${N}` }> extends infer U
    ? U extends { components: infer C } ? C : never
    : never
}

const byStructName = <const S extends keyof StructMap>(name: S): StructMap[S] => {
  const isEnsureExported = (a: unknown): a is EnsureExported =>
    typeof a === "object" && a !== null && "name" in a && a.name === "ensureExported"
  const isNamed = (
    a: unknown,
  ): a is Extract<EnsureExported["inputs"][number], { internalType: `struct ${S}` }> =>
    typeof a === "object" && a !== null && "internalType" in a
    && a.internalType === `struct ${name}`

  return Abi
    .find(isEnsureExported)!
    .inputs
    .find(isNamed)!
    .components as StructMap[S]
}

/**
 * @category abis
 * @since 2.0.0
 */
export const FungibleAssetOrderAbi = constant(byStructName("FungibleAssetOrder"))
/**
 * @category abis
 * @since 2.0.0
 */
export const InstructionAbi = constant(byStructName("Instruction"))
/**
 * @category abis
 * @since 2.0.0
 */
export const ZkgmPacketAbi = constant(byStructName("ZkgmPacket"))
/**
 * @category abis
 * @since 2.0.0
 */
export const ForwardAbi = constant(byStructName("Forward"))
/**
 * @category abis
 * @since 2.0.0
 */
export const MultiplexAbi = constant(byStructName("Multiplex"))
/**
 * @category abis
 * @since 2.0.0
 */
export const BatchAbi = constant(byStructName("Batch"))
/**
 * @category abis
 * @since 2.0.0
 */
export const AckAbi = constant(byStructName("Ack"))
/**
 * @category abis
 * @since 2.0.0
 */
export const BatchAckAbi = constant(byStructName("BatchAck"))
/**
 * @category abis
 * @since 2.0.0
 */
export const FungibleAssetOrderAckAbi = constant(byStructName("FungibleAssetOrderAck"))

/**
 * @category models
 * @since 2.0.0
 */
const Version = S.NonNegativeInt.pipe(
  S.between(0, 0),
)
/**
 * @category models
 * @since 2.0.0
 */
type Version = typeof Version.Type

/**
 * @category models
 * @since 2.0.0
 */
const OpCode = S.NonNegativeInt
/**
 * @category models
 * @since 2.0.0
 */
type OpCode = typeof OpCode.Type

/**
 * @category models
 * @since 2.0.0
 */
const MultiplexOperand = S.Union(
  S.Tuple(Hex, S.Boolean, Hex, Hex),
)
/**
 * @category models
 * @since 2.0.0
 */
type MultiplexOperand = typeof MultiplexOperand.Type

/**
 * @category models
 * @since 2.0.0
 */
const FungibleAssetOrderOperand = S.Union(
  S.Tuple(
    Hex,
    Hex,
    Hex,
    S.BigIntFromSelf,
    S.String,
    S.String,
    S.Uint8,
    S.BigIntFromSelf,
    HexChecksum,
    S.BigIntFromSelf,
  ),
)
/**
 * @category models
 * @since 2.0.0
 */
type FungibleAssetOrderOperand = typeof FungibleAssetOrderOperand.Type

/**
 * @category models
 * @since 2.0.0
 */
export const Operand = S.Union(
  // [`0x${string}`, bigint, { version: number; opcode: number; operand: `0x${string}`; }]
  S.Tuple(Hex, S.BigIntFromSelf, S.Struct({ version: Version, opcode: OpCode, operand: Hex })),
  // [number, number, `0x${string}`]
  S.Tuple(S.Number, S.Number, Hex),
  // [bigint, bigint, bigint, { version: number; opcode: number; operand: `0x${string}`; }]
  S.Tuple(
    S.BigIntFromSelf,
    S.BigIntFromSelf,
    S.BigIntFromSelf,
    S.Struct({ version: Version, opcode: OpCode, operand: Hex }),
  ),
  MultiplexOperand,
  // [readonly { version: number; opcode: number; operand: `0x${string}`; }[]]
  S.Tuple(S.Array(S.Struct({ version: Version, opcode: OpCode, operand: Hex }))),
  FungibleAssetOrderOperand,
  // [bigint, `0x${string}`]
  S.Tuple(S.BigIntFromSelf, Hex),
  // [readonly `0x${string}`[]]
  S.Tuple(S.NonEmptyArray(Hex)),
)
/**
 * @category models
 * @since 2.0.0
 */
export type Operand = typeof Operand.Type

/**
 * @category models
 * @since 2.0.0
 */
export class Forward extends S.TaggedClass<Forward>()("Forward", {
  opcode: S.Literal(0).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 0 as const,
      decoding: () => 0 as const,
    }),
  ),
  version: S.Literal(0).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 0 as const,
      decoding: () => 0 as const,
    }),
  ),
  operand: S.Tuple(
    // TODO(ehegnes): Check bitwidth constraint
    S.PositiveBigIntFromSelf.annotations({
      description: "Path",
    }),
    S.PositiveBigIntFromSelf.annotations({
      description: "Timeout Height",
    }),
    S.PositiveBigIntFromSelf.annotations({
      description: "Timeout Timestamp",
    }),
    S.suspend((): S.Schema<Schema, SchemaEncoded> => Schema),
  ),
}) {
  static fromOperand = (operand: typeof this.Type.operand) => this.make({ operand })
}

/**
 * @category models
 * @since 2.0.0
 */
export class Multiplex extends S.TaggedClass<Multiplex>()("Multiplex", {
  opcode: S.Literal(1).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 1 as const,
      decoding: () => 1 as const,
    }),
  ),
  version: S.Literal(0).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 0 as const,
      decoding: () => 0 as const,
    }),
  ),
  operand: MultiplexOperand,
}) {
  static fromOperand = (operand: typeof this.Type.operand) => this.make({ operand })
}

/**
 * @category models
 * @since 2.0.0
 */
export class Batch extends S.TaggedClass<Batch>()("Batch", {
  opcode: S.Literal(2).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 2 as const,
      decoding: () => 2 as const,
    }),
  ),
  version: S.Literal(0).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 0 as const,
      decoding: () => 0 as const,
    }),
  ),
  operand: S.NonEmptyArray(S.suspend((): S.Schema<Schema, SchemaEncoded> => Schema)),
}) {
  static fromOperand = (operand: typeof this.Type.operand) => this.make({ operand })
}

/**
 * @category models
 * @since 2.0.0
 */
export class FungibleAssetOrderV1
  extends S.TaggedClass<FungibleAssetOrderV1>()("FungibleAssetOrder", {
    opcode: S.Literal(3).pipe(
      S.optional,
      S.withDefaults({
        constructor: () => 3 as const,
        decoding: () => 3 as const,
      }),
    ),
    version: S.Literal(1).pipe(
      S.optional,
      S.withDefaults({
        constructor: () => 1 as const,
        decoding: () => 1 as const,
      }),
    ),
    operand: FungibleAssetOrderOperand,
  })
{
  static fromOperand = (operand: typeof this.Type.operand) => this.make({ operand })
}

export class FungibleAssetOrderV2
  extends S.TaggedClass<FungibleAssetOrderV2>()("FungibleAssetOrder", {
    opcode: S.Literal(3).pipe(
      S.optional,
      S.withDefaults({
        constructor: () => 3 as const,
        decoding: () => 3 as const,
      }),
    ),
    version: S.Literal(2).pipe(
      S.optional,
      S.withDefaults({
        constructor: () => 2 as const,
        decoding: () => 2 as const,
      }),
    ),
    operand: FungibleAssetOrderOperand,
  })
{
  static fromOperand = (operand: typeof this.Type.operand) => this.make({ operand })
}

export const FungibleAssetOrder = S.Union(FungibleAssetOrderV1, FungibleAssetOrderV2)
export type FungibleAssetOrder = typeof FungibleAssetOrder.Type

const faov1 = FungibleAssetOrderV1.fromOperand([] as unknown as any)
declare const f: (a: FungibleAssetOrder) => void
const g = f(faov1)

/**
 * @category models
 * @since 2.0.0
 */
export type Schema = Forward | Multiplex | Batch | FungibleAssetOrder

/**
 * @category models
 * @since 2.0.0
 */
type SchemaEncoded =
  | {
    readonly _tag: "Forward"
    readonly opcode?: 0 | undefined
    readonly version?: 0 | undefined
    readonly operand: readonly [bigint, bigint, bigint, SchemaEncoded]
  }
  | typeof Multiplex.Encoded
  | {
    readonly _tag: "Batch"
    readonly opcode?: 2 | undefined
    readonly version?: 0 | undefined
    readonly operand: A.NonEmptyReadonlyArray<SchemaEncoded>
  }
  | typeof FungibleAssetOrder.Encoded

/**
 * @category models
 * @since 2.0.0
 */
export const Schema = S.Union(Forward, Multiplex, Batch, FungibleAssetOrder)

/**
 * @category models
 * @since 2.0.0
 */
export const Instruction = Data.taggedEnum<Instruction>()
/**
 * @category models
 * @since 2.0.0
 */
export type Instruction = typeof Schema.Type

/**
 * Encodes an {@link Instruction} as as {@link Hex} for dispatching.
 *
 * @example
 *
 * @category utils
 * @since 2.0.0
 */
export const encode: (_: Instruction) => Hex = Instruction.$match({
  Forward: ({ operand }) =>
    encodeAbiParameters(ForwardAbi(), [
      operand[0],
      operand[1],
      operand[2],
      {
        opcode: operand[3].opcode,
        version: operand[3].version,
        operand: encode(operand[3]),
      },
    ]),
  Multiplex: ({ operand }) => encodeAbiParameters(MultiplexAbi(), operand),
  Batch: ({ operand }) =>
    encodeAbiParameters(BatchAbi(), [
      operand.map((i: Schema) => ({
        version: i.version,
        opcode: i.opcode,
        operand: encode(i),
      })),
    ]),
  FungibleAssetOrder: ({ operand }) => encodeAbiParameters(FungibleAssetOrderAbi(), operand),
})
