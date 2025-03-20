import { Effect } from "effect"
import { ucs03abi } from "../evm/abi/ucs03.js"
import { ViemWalletClient } from "../evm/client.js"
import { writeContract } from "../evm/contract.js"
import { type Instruction, encodeAbi } from "./instruction.js"
import { generateSalt } from "../utils/index.js"
import { SourceConfig } from "../evm/quote-token.js"
import { executeContract } from "../cosmos/contract.js"
import { SigningCosmWasmClientContext } from "../cosmos/client.js"

export const sendInstructionEvm = (instruction: Instruction) =>
  Effect.gen(function* () {
    const walletClient = yield* ViemWalletClient
    const sourceConfig = yield* SourceConfig

    return yield* writeContract(walletClient.client, {
      account: walletClient.account,
      abi: ucs03abi,
      chain: walletClient.chain,
      functionName: "send",
      address: sourceConfig.ucs03address,
      args: [
        sourceConfig.channelId,
        0n,
        1000000000000n,
        generateSalt(),
        {
          opcode: instruction.opcode,
          version: instruction.version,
          operand: encodeAbi(instruction)
        }
      ]
    })
  })

export const sendInstructionCosmos = (instruction: Instruction) =>
  Effect.gen(function* () {
    const signingClient = yield* SigningCosmWasmClientContext
    const sourceConfig = yield* SourceConfig

    return yield* executeContract(
      signingClient.client,
      "union1d95n4r6dnrfrps59szhl8mk7yqewsuzyw0zh5q",
      sourceConfig.ucs03address,
      {
        send: {
          channel_id: sourceConfig.channelId,
          timeout_height: 0,
          timeout_timestamp: 100000n,
          salt: generateSalt(),
          instruction: encodeAbi(instruction)
        }
      }
    )
  })

// Send {
//     channel_id: ChannelId,
//     timeout_height: u64,
//     timeout_timestamp: u64,
//     salt: H256,
//     instruction: Bytes,
// },
