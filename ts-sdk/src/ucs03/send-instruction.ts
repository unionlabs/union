import { Effect } from "effect"
import { ucs03abi } from "../evm/abi/ucs03.js"
import { ViemWalletClient } from "../evm/client.js"
import { writeContract } from "../evm/contract.js"
import { type Instruction, encodeAbi } from "./instruction.js"
import { generateSalt } from "../utils/index.js"
import { SourceConfig } from "../evm/quote-token.js"
import type { Address } from "viem"

export const sendInstructionEvm = (instruction: Instruction, sender: Address) =>
  Effect.gen(function* () {
    const walletClient = yield* ViemWalletClient
    const sourceConfig = yield* SourceConfig

    return yield* writeContract(walletClient.client, {
      account: sender,
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
