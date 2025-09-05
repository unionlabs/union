/**
 * This module handles interaction with the [UCS03](https://docs.union.build/ucs/03/) protocol.
 *
 * @since 2.0.0
 */

import { Effect, Match, ParseResult } from "effect"
import * as A from "effect/Array"
import * as Data from "effect/Data"
import { constant, flow, pipe } from "effect/Function"
import * as S from "effect/Schema"
import { decodeAbiParameters, encodeAbiParameters } from "viem"
import { Hex, HexChecksum } from "./schema/hex.js"
import { Uint256FromSelf } from "./schema/uint256.js"
import { Uint64FromSelf } from "./schema/uint64.js"

/**
 * Contract ABI
 *
 * @category abis
 * @since 2.0.0
 */
export const Abi = [
  {
    type: "function",
    name: "ensureCreateWrappedTokenExported",
    inputs: [
      { name: "path", type: "uint256", internalType: "uint256" },
      { name: "channelId", type: "uint32", internalType: "uint32" },
      { name: "baseToken", type: "bytes", internalType: "bytes" },
      { name: "quoteToken", type: "address", internalType: "address" },
      { name: "metadata", type: "bytes", internalType: "bytes" },
      { name: "kind", type: "uint8", internalType: "uint8" },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
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
      internalType: "struct Call",
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
      internalType: "struct TokenOrderV1",
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
      internalType: "struct TokenOrderAck",
      components: [{ name: "fillType", type: "uint256", internalType: "uint256" }, {
        name: "marketMaker",
        type: "bytes",
        internalType: "bytes",
      }],
    }, {
      name: "",
      type: "tuple",
      internalType: "struct TokenOrderV2",
      components: [
        { name: "sender", type: "bytes", internalType: "bytes" },
        { name: "receiver", type: "bytes", internalType: "bytes" },
        { name: "baseToken", type: "bytes", internalType: "bytes" },
        { name: "baseAmount", type: "uint256", internalType: "uint256" },
        { name: "quoteToken", type: "bytes", internalType: "bytes" },
        { name: "quoteAmount", type: "uint256", internalType: "uint256" },
        { name: "kind", type: "uint8", internalType: "uint8" },
        { name: "metadata", type: "bytes", internalType: "bytes" },
      ],
    }, {
      name: "",
      type: "tuple",
      internalType: "struct TokenMetadata",
      components: [{ name: "implementation", type: "bytes", internalType: "bytes" }, {
        name: "initializer",
        type: "bytes",
        internalType: "bytes",
      }],
    }, {
      name: "",
      type: "tuple",
      internalType: "struct SolverMetadata",
      components: [{ name: "solverAddress", type: "bytes", internalType: "bytes" }, {
        name: "metadata",
        type: "bytes",
        internalType: "bytes",
      }],
    }],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "event",
    name: "CreateWrappedToken",
    inputs: [
      { name: "path", type: "uint256", indexed: false, internalType: "uint256" },
      { name: "channelId", type: "uint32", indexed: true, internalType: "uint32" },
      { name: "baseToken", type: "bytes", indexed: false, internalType: "bytes" },
      { name: "quoteToken", type: "address", indexed: true, internalType: "address" },
      { name: "metadata", type: "bytes", indexed: false, internalType: "bytes" },
      { name: "kind", type: "uint8", indexed: false, internalType: "uint8" },
    ],
    anonymous: false,
  },
  {
    type: "constructor",
    inputs: [
      { name: "_ibcHandler", type: "address", internalType: "contract IIBCModulePacket" },
      { name: "_sendImpl", type: "address", internalType: "contract UCS03ZkgmSendImpl" },
      { name: "_stakeImpl", type: "address", internalType: "contract UCS03ZkgmStakeImpl" },
      { name: "_faoImpl", type: "address", internalType: "contract UCS03ZkgmTokenOrderImpl" },
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
    name: "_deprecated_channelBalanceV1",
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
    name: "authority",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "address" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "channelBalanceV2",
    inputs: [
      { name: "", type: "uint32", internalType: "uint32" },
      { name: "", type: "uint256", internalType: "uint256" },
      { name: "", type: "address", internalType: "address" },
      { name: "", type: "bytes", internalType: "bytes" },
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
    name: "decodeRelayerMessage",
    inputs: [{ name: "relayerMsg", type: "bytes", internalType: "bytes" }],
    outputs: [{ name: "", type: "bool", internalType: "bool" }, {
      name: "",
      type: "bytes",
      internalType: "bytes",
    }],
    stateMutability: "pure",
  },
  {
    type: "function",
    name: "decodeZkgmERC20InitializeCall",
    inputs: [{ name: "call", type: "bytes", internalType: "bytes" }],
    outputs: [
      { name: "", type: "address", internalType: "address" },
      { name: "", type: "address", internalType: "address" },
      { name: "", type: "string", internalType: "string" },
      { name: "", type: "string", internalType: "string" },
      { name: "", type: "uint8", internalType: "uint8" },
    ],
    stateMutability: "pure",
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
    name: "migrateV1ToV2",
    inputs: [{
      name: "balanceMigrations",
      type: "tuple[]",
      internalType: "struct V1ToV2Migration[]",
      components: [
        { name: "path", type: "uint256", internalType: "uint256" },
        { name: "channelId", type: "uint32", internalType: "uint32" },
        { name: "baseToken", type: "address", internalType: "address" },
        { name: "quoteToken", type: "bytes", internalType: "bytes" },
      ],
    }, {
      name: "wrappedMigrations",
      type: "tuple[]",
      internalType: "struct V1ToV2WrappedTokenMigration[]",
      components: [
        { name: "path", type: "uint256", internalType: "uint256" },
        { name: "channelId", type: "uint32", internalType: "uint32" },
        { name: "baseToken", type: "bytes", internalType: "bytes" },
        { name: "quoteToken", type: "address", internalType: "address" },
      ],
    }],
    outputs: [],
    stateMutability: "nonpayable",
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
        internalType: "struct TokenMetadata",
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
    name: "CreateWrappedToken",
    inputs: [
      { name: "path", type: "uint256", indexed: false, internalType: "uint256" },
      { name: "channelId", type: "uint32", indexed: true, internalType: "uint32" },
      { name: "baseToken", type: "bytes", indexed: false, internalType: "bytes" },
      { name: "quoteToken", type: "address", indexed: true, internalType: "address" },
      { name: "metadata", type: "bytes", indexed: false, internalType: "bytes" },
      { name: "kind", type: "uint8", indexed: false, internalType: "uint8" },
    ],
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
  { type: "error", name: "ErrAsyncCallUnsupported", inputs: [] },
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
export const TokenOrderV1Abi = constant(byStructName("TokenOrderV1"))
/**
 * @category abis
 * @since 2.0.0
 */
export const TokenOrderV2Abi = constant(byStructName("TokenOrderV2"))
/**
 * @category abis
 * @since 2.0.0
 */
export const TokenMetadataAbi = constant(byStructName("TokenMetadata"))
/**
 * @category abis
 * @since 2.0.0
 */
export const SolverMetadataAbi = constant(byStructName("SolverMetadata"))
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
export const CallAbi = constant(byStructName("Call"))
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
export const TokenOrderOrderAckAbi = constant(byStructName("TokenOrderAck"))

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
const CallOperand = S.Union(
  S.Tuple(
    Hex.pipe(
      S.annotations({
        title: "sender",
      }),
    ),
    S.Boolean.pipe(
      S.annotations({
        title: "eureka",
      }),
    ),
    Hex.pipe(
      S.annotations({
        title: "contractAddress",
      }),
    ),
    Hex.pipe(
      S.annotations({
        title: "contractCalldata",
      }),
    ),
  ),
)
/**
 * @category models
 * @since 2.0.0
 */
type CallOperand = typeof CallOperand.Type

/**
 * @category models
 * @since 2.0.0
 */
const TokenOrderOperandV1 = S.Union(
  S.Tuple(
    Hex.pipe(
      S.annotations({
        title: "sender",
        description: "source chain sender address",
      }),
    ),
    Hex.pipe(
      S.annotations({
        title: "receiver",
        description: "destination chain receiver address",
      }),
    ),
    Hex.pipe(
      S.annotations({
        title: "baseToken",
        description: "token being sent",
      }),
    ),
    S.BigIntFromSelf.pipe(
      S.annotations({
        title: "baseAmount",
        description: "amount being sent",
      }),
    ),
    S.String.pipe(
      S.annotations({
        title: "baseTokenSymbol",
        description: "token symbol for wrapped asset",
      }),
    ),
    S.String.pipe(
      S.annotations({
        title: "baseTokenName",
        description: "token name for wrapped asset",
      }),
    ),
    S.Uint8.pipe(
      S.annotations({
        title: "baseTokenDecimals",
        description: "token decimals for wrapped asset",
      }),
    ),
    S.BigIntFromSelf.pipe(
      S.annotations({
        title: "baseTokenPath",
        description: "origin path for unwrapping",
      }),
    ),
    HexChecksum.pipe(
      S.annotations({
        title: "quoteToken",
        description: "token requested in return",
      }),
    ),
    S.BigIntFromSelf.pipe(
      S.annotations({
        title: "quoteAmount",
        description: "minimum amount requested",
      }),
    ),
  ),
)
/**
 * @category models
 * @since 2.0.0
 */
type TokenOrderOperandV1 = typeof TokenOrderOperandV1.Type

/**
 * @category schemas
 * @since 2.0.0
 */
const SolverMetadata = S.Struct({
  solverAddress: Hex,
  metadata: Hex,
})
/**
 * @category models
 * @since 2.0.0
 */
export type SolverMetadata = typeof SolverMetadata.Type

/**
 * @category schemas
 * @since 2.0.0
 */
export const TokenMetadataKind = S.Union(
  S.Literal(0).pipe(
    S.annotations({
      title: "Initialize",
      description: "Uses a metadata image hash for existing token identification",
    }),
  ),
  S.Literal(1).pipe(
    S.annotations({
      title: "Escrow",
    }),
  ),
  S.Literal(2).pipe(
    S.annotations({
      title: "Unescrow",
    }),
  ),
  S.Literal(3).pipe(
    S.annotations({
      title: "Solve",
    }),
  ),
)
/**
 * @category models
 * @since 2.0.0
 */
export type TokenMetadataKind = typeof TokenMetadataKind.Type

/**
 * @category models
 * @since 2.0.0
 */
const TokenOrderOperandV2 = S.Union(
  S.Tuple(
    Hex.pipe(
      S.annotations({
        title: "sender",
        description: "source chain sender address",
      }),
    ),
    Hex.pipe(
      S.annotations({
        title: "receiver",
        description: "destination chain receiver address",
      }),
    ),
    Hex.pipe(
      S.annotations({
        title: "baseToken",
        description: "token being sent",
      }),
    ),
    S.BigIntFromSelf.pipe(
      S.annotations({
        title: "baseAmount",
        description: "amount being sent",
      }),
    ),
    HexChecksum.pipe(
      S.annotations({
        title: "quoteToken",
        description: "token requested in return",
      }),
    ),
    S.BigIntFromSelf.pipe(
      S.annotations({
        title: "quoteAmount",
        description: "minimum amount requested",
      }),
    ),
    TokenMetadataKind.pipe(
      S.annotations({
        title: "kind",
      }),
    ),
    S.Union(
      S.Literal("0x"),
      Hex.pipe(
        S.annotations({
          title: "metadata",
          description: "metadata based on kind. for 0 token init args, 1&2 empty.",
        }),
      ),
    ),
  ),
)
/**
 * @category models
 * @since 2.0.0
 */
type TokenOrderOperandV2 = typeof TokenOrderOperandV2.Type

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
  CallOperand,
  // [readonly { version: number; opcode: number; operand: `0x${string}`; }[]]
  S.Tuple(S.Array(S.Struct({ version: Version, opcode: OpCode, operand: Hex }))),
  TokenOrderOperandV1,
  TokenOrderOperandV2,
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
export class Forward extends S.TaggedClass<Forward>()("@unionlabs/sdk/Ucs03/Forward", {
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
    Uint256FromSelf.annotations({
      title: "path",
      description: "channel sequence as (prevDst,nextSrc) pairs",
    }),
    Uint64FromSelf.annotations({
      title: "timeout height",
      description: "block height timeout",
    }),
    Uint64FromSelf.annotations({
      title: "timeout timestamp",
      description: "Unix timestamp timeout",
    }),
    S.suspend((): S.Schema<Schema, SchemaEncoded> => Schema).annotations({
      title: "instruction",
      description: "instruction to forward",
    }),
  ),
}) {
  static fromOperand = (operand: typeof this.Type.operand) => this.make({ operand })
}

/**
 * @category models
 * @since 2.0.0
 */
export class Call extends S.TaggedClass<Call>()("@unionlabs/sdk/Ucs03/Call", {
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
  operand: CallOperand,
}) {
  static fromOperand = (operand: typeof this.Type.operand) => this.make({ operand })
}

/**
 * @category models
 * @since 2.0.0
 */
export class Batch extends S.TaggedClass<Batch>()("@unionlabs/sdk/Ucs03/Batch", {
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
  // XXX: can only be `Call` or `TokenOrder`
  operand: S.NonEmptyArray(S.suspend((): S.Schema<Schema, SchemaEncoded> => Schema)),
}) {
  static fromOperand = (operand: typeof this.Type.operand) => this.make({ operand })
}

/**
 * @category models
 * @since 2.0.0
 */
export class TokenOrderV1 extends S.TaggedClass<TokenOrderV1>()("@unionlabs/sdk/Ucs03/TokenOrder", {
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
  operand: TokenOrderOperandV1,
}) {
  static fromOperand = (operand: typeof this.Type.operand) => this.make({ operand })
}

/**
 * @category models
 * @since 2.0.0
 */
export class TokenOrderV2 extends S.TaggedClass<TokenOrderV2>()("@unionlabs/sdk/Ucs03/TokenOrder", {
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
  operand: TokenOrderOperandV2,
}) {
  static fromOperand = (operand: typeof this.Type.operand) => this.make({ operand })
}

/**
 * @category models
 * @since 2.0.0
 */
export class Instruction extends S.TaggedClass<Instruction>()("@unionlabs/sdk/Ucs03/Instruction", {
  opcode: S.Uint8,
  version: S.Uint8,
  operand: Hex,
}) {}

/**
 * @category schemas
 * @since 2.0.0
 */
export const TokenOrder = S.Union(TokenOrderV1, TokenOrderV2)
/**
 * @category models
 * @since 2.0.0
 */
export type TokenOrder = typeof TokenOrder.Type

/**
 * @category models
 * @since 2.0.0
 */
export type Schema = Instruction | Forward | Call | Batch | TokenOrder // XXX: call this `Root` and drop `Instruction`

/**
 * @category models
 * @since 2.0.0
 */
type SchemaEncoded =
  | typeof Instruction.Encoded
  | {
    readonly _tag: "@unionlabs/sdk/Ucs03/Forward"
    readonly opcode?: 0 | undefined
    readonly version?: 0 | undefined
    readonly operand: readonly [bigint, bigint, bigint, SchemaEncoded]
  }
  | typeof Call.Encoded
  | {
    readonly _tag: "@unionlabs/sdk/Ucs03/Batch"
    readonly opcode?: 2 | undefined
    readonly version?: 0 | undefined
    readonly operand: A.NonEmptyReadonlyArray<SchemaEncoded>
  }
  | typeof TokenOrder.Encoded

/**
 * @category schemas
 * @since 2.0.0
 */
export const Schema = S.Union(Instruction, Forward, Call, Batch, TokenOrder)

/**
 * @category models
 * @since 2.0.0
 */
export const Ucs03 = Data.taggedEnum<Ucs03>()
/**
 * @category models
 * @since 2.0.0
 */
export type Ucs03 = typeof Schema.Type

/**
 * @category transformations
 * @since 2.0.0
 */
export const ForwardFromHex = S.transformOrFail(
  Hex,
  Forward,
  {
    decode: (fromA, _, ast) =>
      pipe(
        Effect.try(() => decodeAbiParameters(ForwardAbi(), fromA)),
        Effect.mapError((e) => new ParseResult.Type(ast, fromA, String(e.error))),
        Effect.flatMap(([path, timeoutHeight, timeoutTimestamp, instruction]) =>
          pipe(
            S.decodeUnknown(S.suspend(() => Ucs03FromHex))(instruction),
            Effect.map((i) =>
              Forward.fromOperand([
                path,
                timeoutHeight,
                timeoutTimestamp,
                i,
              ])
            ),
          )
        ),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
      ),
    encode: (toI, _, ast, toA) =>
      pipe(
        S.encodeUnknown(S.suspend(() => Ucs03FromHex))(toA.operand[3]),
        Effect.map((operand) =>
          [
            toA.operand[0],
            toA.operand[1],
            toA.operand[2],
            {
              opcode: toA.operand[3].opcode,
              version: toA.operand[3].version,
              operand,
            },
          ] as const
        ),
        Effect.flatMap((x) => Effect.try(() => encodeAbiParameters(ForwardAbi(), x))),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
        Effect.catchTag(
          "UnknownException",
          (error) => ParseResult.fail(new ParseResult.Type(ast, toI, String(error.error))),
        ),
      ),
  },
)

/**
 * @category transformations
 * @since 2.0.0
 */
export const CallFromHex = S.transformOrFail(
  Hex,
  Call,
  {
    decode: (fromA, _, ast) =>
      pipe(
        Effect.try(() => decodeAbiParameters(CallAbi(), fromA)),
        Effect.catchTag(
          "UnknownException",
          (error) => ParseResult.fail(new ParseResult.Type(ast, fromA, String(error.error))),
        ),
        Effect.map(Call.fromOperand),
      ),
    encode: (toI, _, ast, toA) =>
      pipe(
        Effect.try(() => encodeAbiParameters(CallAbi(), toA.operand)),
        Effect.catchTag(
          "UnknownException",
          (error) => ParseResult.fail(new ParseResult.Type(ast, toI, String(error.error))),
        ),
      ),
  },
)

/**
 * @category transformations
 * @since 2.0.0
 */
export const BatchFromHex = S.transformOrFail(
  Hex,
  Batch,
  {
    decode: (fromA, _, ast) =>
      pipe(
        Effect.try(() => decodeAbiParameters(BatchAbi(), fromA)),
        Effect.mapError((e) => new ParseResult.Type(ast, fromA, String(e.error))),
        Effect.flatMap(flow(
          (x) => x[0],
          Effect.forEach(
            (instruction) => S.decode(Ucs03FromHex)(instruction.operand),
            { concurrency: "unbounded" },
          ),
          // Effect.flatMap(Effect.liftPredicate(Predicate.(Batch), (n) => new ParseResult.Type(ast, fromA, "Instruction cannot be nested inside a Batch"))),
        )),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
        // XXX: narrow to exclude instruction
        Effect.map((operand) => Batch.fromOperand(operand as unknown as any)),
      ),
    encode: (toI, _, ast, toA) =>
      pipe(
        toA.operand,
        A.map((instruction) =>
          pipe(
            S.encodeUnknown(Ucs03FromHex)(instruction),
            Effect.map((operand) =>
              ({
                version: instruction.version,
                opcode: instruction.opcode,
                operand,
              }) as const
            ),
          )
        ),
        Effect.allWith({ concurrency: "unbounded" }),
        Effect.flatMap(x => Effect.try(() => encodeAbiParameters(BatchAbi(), [x]))),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
        Effect.catchTag(
          "UnknownException",
          (error) => ParseResult.fail(new ParseResult.Type(ast, toI, String(error.error))),
        ),
      ),
  },
)

/**
 * @category transformations
 * @since 2.0.0
 */
export const TokenOrderFromHex = S.transformOrFail(
  Hex,
  TokenOrder,
  {
    decode: (fromA, _, ast) =>
      pipe(
        Effect.raceAll(
          [
            Effect.try(
              () => decodeAbiParameters(TokenOrderV1Abi(), fromA),
            ),
            Effect.try(
              () => decodeAbiParameters(TokenOrderV2Abi(), fromA),
            ),
          ],
        ),
        Effect.flatMap((operand) =>
          S.decodeUnknown(TokenOrder)({ _tag: "@unionlabs/sdk/Ucs03/TokenOrder", operand })
        ),
        Effect.catchTag(
          "UnknownException",
          (error) => ParseResult.fail(new ParseResult.Type(ast, fromA, String(error.error))),
        ),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
      ),
    encode: (toI, _, ast, toA) =>
      pipe(
        // TODO(ehegnes): improve narrowing
        Match.value(toA.version),
        Match.when(
          1,
          () =>
            Effect.try(() =>
              encodeAbiParameters(
                TokenOrderV1Abi(),
                toA.operand as TokenOrderV1["operand"],
              )
            ),
        ),
        Match.when(
          2,
          () =>
            Effect.try(() =>
              encodeAbiParameters(
                TokenOrderV2Abi(),
                toA.operand as TokenOrderV2["operand"],
              )
            ),
        ),
        Match.exhaustive,
        Effect.catchTag("UnknownException", (error) =>
          ParseResult.fail(new ParseResult.Type(ast, toI, String(error.error)))),
      ),
  },
)

/**
 * @category transformations
 * @since 2.0.0
 */
export const InstructionFromHex = S.transformOrFail(
  Hex,
  Instruction,
  {
    decode: (fromA, _, ast) =>
      pipe(
        Effect.try(() => decodeAbiParameters(InstructionAbi(), fromA)),
        Effect.flatMap(([version, opcode, operand]) =>
          S.decode(Instruction)({
            _tag: "@unionlabs/sdk/Ucs03/Instruction",
            version,
            opcode,
            operand,
          })
        ),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
        Effect.catchTag(
          "UnknownException",
          (error) => ParseResult.fail(new ParseResult.Type(ast, fromA, String(error.error))),
        ),
      ),
    encode: (toI, _, ast, toA) =>
      pipe(
        Effect.try(() =>
          encodeAbiParameters(
            InstructionAbi(),
            [toA.version, toA.opcode, toA.operand] as const,
          )
        ),
        Effect.catchTag(
          "UnknownException",
          (error) => ParseResult.fail(new ParseResult.Type(ast, toI, String(error.error))),
        ),
      ),
  },
)

/**
 * @category transformations
 * @since 2.0.0
 */
export const SolverMetadataFromHex = S.transformOrFail(
  Hex,
  SolverMetadata,
  {
    decode: (fromA, _, ast) =>
      pipe(
        Effect.try(() => decodeAbiParameters(SolverMetadataAbi(), fromA)),
        Effect.flatMap(([solverAddress, metadata]) =>
          S.decode(SolverMetadata)({
            solverAddress,
            metadata,
          })
        ),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
        Effect.catchTag(
          "UnknownException",
          (error) => ParseResult.fail(new ParseResult.Type(ast, fromA, String(error.error))),
        ),
      ),
    encode: (toI, _, ast) =>
      pipe(
        Effect.try(() =>
          encodeAbiParameters(SolverMetadataAbi(), [toI.solverAddress, toI.metadata])
        ),
        Effect.catchTag(
          "UnknownException",
          (error) => ParseResult.fail(new ParseResult.Type(ast, toI, String(error.error))),
        ),
      ),
  },
)

/**
 * @category transformations
 * @since 2.0.0
 */
export const Ucs03FromHex: S.Union<[
  S.transformOrFail<
    typeof Hex,
    typeof Batch
  >,
  S.transformOrFail<
    typeof Hex,
    typeof TokenOrder
  >,
  S.transformOrFail<
    typeof Hex,
    typeof Forward
  >,
  S.transformOrFail<
    typeof Hex,
    typeof Call
  >,
  S.transformOrFail<
    typeof Hex,
    typeof Instruction
  >,
]> = S.Union(
  BatchFromHex,
  TokenOrderFromHex,
  ForwardFromHex,
  CallFromHex,
  InstructionFromHex,
)

/**
 * @category transformations
 * @since 2.0.0
 */
export const Ucs03FromHexWithoutInstruction: S.Union<[
  S.transformOrFail<
    typeof Hex,
    typeof Batch
  >,
  S.transformOrFail<
    typeof Hex,
    typeof TokenOrder
  >,
  S.transformOrFail<
    typeof Hex,
    typeof Forward
  >,
  S.transformOrFail<
    typeof Hex,
    typeof Call
  >,
]> = S.Union(
  BatchFromHex,
  TokenOrderFromHex,
  ForwardFromHex,
  CallFromHex,
)

/**
 * @category transformations
 * @since 2.0.0
 */
export const Ucs03FromInstruction = S.transformOrFail(
  Instruction,
  Schema,
  {
    decode: (fromA, _, ast, fromI) =>
      pipe(
        fromA.operand,
        // XXX: choose decode given explicit opcode / version
        S.decode(Ucs03FromHex),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
      ),
    encode: (toI, _, ast, toA) =>
      pipe(
        toA,
        S.encode(Ucs03FromHex),
        Effect.flatMap((operand) =>
          S.decode(Instruction)({
            _tag: "@unionlabs/sdk/Ucs03/Instruction",
            version: toA.version,
            opcode: toA.opcode,
            operand,
          })
        ),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
      ),
  },
)

/**
 * * encode: wraps a UCS03 schema with an instruction
 * @category transformations
 * @since 2.0.0
 */
export const Ucs03WithInstructionFromHex = S.compose(
  InstructionFromHex,
  Ucs03FromInstruction,
)
/**
 * @category transformations
 * @since 2.0.0
 */
export type Ucs03WithInstructionFromHex = typeof Ucs03WithInstructionFromHex.Type

/**
 * @category models
 * @since 2.0.0
 */
export class Packet extends S.TaggedClass<Packet>()("@unionlabs/sdk/Ucs03/Packet", {
  salt: Hex,
  path: Uint256FromSelf,
  instruction: Instruction,
}) {}

/**
 * @category transformations
 * @since 2.0.0
 */
export const PacketFromHex = S.transformOrFail(
  Hex,
  Packet,
  {
    decode: (fromA, _, ast, fromI) =>
      pipe(
        Effect.try(() => decodeAbiParameters(ZkgmPacketAbi(), fromA)),
        Effect.flatMap(([salt, path, instruction]) =>
          pipe(
            S.decode(Instruction)({
              _tag: "@unionlabs/sdk/Ucs03/Instruction",
              ...instruction,
            }),
            Effect.flatMap((instruction) =>
              S.decode(Packet)({
                _tag: "@unionlabs/sdk/Ucs03/Packet",
                salt,
                path,
                instruction,
              })
            ),
          )
        ),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
        Effect.catchTag(
          "UnknownException",
          (error) => ParseResult.fail(new ParseResult.Type(ast, fromA, String(error.error))),
        ),
      ),
    encode: (toI, _, ast, toA) =>
      pipe(
        Effect.try(() =>
          encodeAbiParameters(ZkgmPacketAbi(), [
            toI.salt,
            toI.path,
            toI.instruction,
          ])
        ),
        Effect.catchTag(
          "UnknownException",
          (error) => ParseResult.fail(new ParseResult.Type(ast, toI, String(error.error))),
        ),
      ),
  },
)
