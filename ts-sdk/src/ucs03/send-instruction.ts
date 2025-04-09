import { Effect } from "effect"
import { ucs03abi } from "../evm/abi/ucs03.js"
import { ViemWalletClient } from "../evm/client.js"
import { writeContract as writeContractEvm } from "../evm/contract.js"
import { writeContract as writeContractAptos } from "../aptos/contract.js"
import { AptosWalletClient } from "../aptos/client.js"
import { type Instruction, encodeAbi } from "./instruction.js"
import { generateSalt } from "../utils/index.js"
import { EvmChannelSource } from "../evm/channel.js"
import { AptosChannelSource } from "../aptos/channel.js"
import { executeContract } from "../cosmos/contract.js"
import { SigningCosmWasmClientContext } from "../cosmos/client.js"
import { CosmosChannelSource } from "../cosmos/channel.js"
import { encodeAbiParameters } from "viem"
import { instructionAbi } from "../evm/abi/index.js"
import { getTimeoutInNanoseconds24HoursFromNow } from "../utils/timeout.js"

export const sendInstructionEvm = (instruction: Instruction) =>
  Effect.gen(function* () {
    const walletClient = yield* ViemWalletClient
    const sourceConfig = yield* EvmChannelSource

    const timeoutTimestamp = getTimeoutInNanoseconds24HoursFromNow()

    return yield* writeContractEvm(walletClient.client, {
      account: walletClient.account,
      abi: ucs03abi,
      chain: walletClient.chain,
      functionName: "send",
      address: sourceConfig.ucs03address,
      args: [
        sourceConfig.channelId,
        0n,
        timeoutTimestamp,
        generateSalt("evm"),
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
    const sourceConfig = yield* CosmosChannelSource

    const timeout_timestamp = getTimeoutInNanoseconds24HoursFromNow()

    return yield* executeContract(
      signingClient.client,
      signingClient.address,
      sourceConfig.ucs03address,
      {
        send: {
          channel_id: sourceConfig.channelId,
          timeout_height: 0,
          timeout_timestamp,
          salt: generateSalt("cosmos"),
          instruction: encodeAbiParameters(instructionAbi, [
            instruction.version,
            instruction.opcode,
            encodeAbi(instruction)
          ])
        }
      }
    )
  })

export const sendInstructionAptos = (instruction: Instruction) =>
  Effect.gen(function* () {
    const walletClient = yield* AptosWalletClient
    const sourceConfig = yield* AptosChannelSource
    const timeoutTimestamp = getTimeoutInNanoseconds24HoursFromNow()

    const module_name = "ibc_app"
    const function_name = "send"
    const function_arguments = [
      sourceConfig.channelId,
      0,
      timeoutTimestamp,
      generateSalt("aptos"),
      instruction.version,
      instruction.opcode,
      encodeAbi(instruction)
    ]

    return yield* writeContractAptos(
      walletClient.client,
      walletClient.account,
      sourceConfig.ucs03address,
      module_name,
      function_name,
      [], // type arguments
      function_arguments
    )
  })
