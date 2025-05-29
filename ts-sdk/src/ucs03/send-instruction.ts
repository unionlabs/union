import { Transaction } from "@mysten/sui/transactions"
import { Effect } from "effect"
import { encodeAbiParameters } from "viem"
import { AptosChannelSource } from "../aptos/channel.js"
import { AptosWalletClient } from "../aptos/client.js"
import { writeContract as writeContractAptos } from "../aptos/contract.js"
import { CosmosChannelSource } from "../cosmos/channel.js"
import { SigningCosmWasmClientContext } from "../cosmos/client.js"
import { executeContract } from "../cosmos/contract.js"
import { instructionAbi } from "../evm/abi/index.js"
import { ucs03abi } from "../evm/abi/ucs03.js"
import { EvmChannelSource } from "../evm/channel.js"
import { ViemWalletClient } from "../evm/client.js"
import { writeContract as writeContractEvm } from "../evm/contract.js"
import { SuiChannelSource } from "../sui/channel.js"
import { SuiWalletClient } from "../sui/client.js"
import { writeContract as writeContractSui } from "../sui/contract.js"
import { SuiFungibleAssetOrderDetails } from "../sui/fungible_asset_order_details.js"
import { generateSalt } from "../utils/index.js"
import { getTimeoutInNanoseconds24HoursFromNow } from "../utils/timeout.js"
import { encodeAbi, FungibleAssetOrder, type Instruction } from "./instruction.js"

export const sendInstructionEvm = (instruction: Instruction) =>
  Effect.gen(function*() {
    const walletClient = yield* ViemWalletClient
    const sourceConfig = yield* EvmChannelSource

    const timeoutTimestamp = getTimeoutInNanoseconds24HoursFromNow()
    const salt = yield* generateSalt("evm")

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
        salt,
        {
          opcode: instruction.opcode,
          version: instruction.version,
          operand: encodeAbi(instruction),
        },
      ],
    })
  })

export const sendInstructionCosmos = (
  instruction: Instruction,
  funds?: ReadonlyArray<{ denom: string; amount: string }>,
) =>
  Effect.gen(function*() {
    const signingClient = yield* SigningCosmWasmClientContext
    const sourceConfig = yield* CosmosChannelSource

    const timeout_timestamp = getTimeoutInNanoseconds24HoursFromNow().toString()
    const salt = yield* generateSalt("cosmos")

    return yield* executeContract(
      signingClient.client,
      signingClient.address,
      sourceConfig.ucs03address,
      {
        send: {
          channel_id: sourceConfig.channelId,
          timeout_height: "0",
          timeout_timestamp,
          salt,
          instruction: encodeAbiParameters(instructionAbi, [
            instruction.version,
            instruction.opcode,
            encodeAbi(instruction),
          ]),
        },
      },
      funds,
    )
  })

export const sendInstructionAptos = (instruction: Instruction) =>
  Effect.gen(function*() {
    const walletClient = yield* AptosWalletClient
    const sourceConfig = yield* AptosChannelSource
    const timeoutTimestamp = getTimeoutInNanoseconds24HoursFromNow()
    const salt = yield* generateSalt("aptos")

    const module_name = "ibc_app"
    const function_name = "send"
    const function_arguments = [
      sourceConfig.channelId,
      0,
      timeoutTimestamp,
      salt,
      instruction.version,
      instruction.opcode,
      encodeAbi(instruction),
    ]

    return yield* writeContractAptos(
      walletClient.client,
      walletClient.account,
      sourceConfig.ucs03address,
      module_name,
      function_name,
      [], // type arguments
      function_arguments,
    )
  })

// turn a hex string like "0xdeadbeef" into a number[] of bytes
function hexToBytes(hex: string): number[] {
  const h = hex.startsWith("0x") ? hex.slice(2) : hex
  return h.match(/.{1,2}/g)!.map(b => parseInt(b, 16))
}

export const sendInstructionSui = (fungibleAssetOrder: FungibleAssetOrder) =>
  Effect.gen(function*() {
    const walletClient = yield* SuiWalletClient
    const sourceConfig = yield* SuiChannelSource
    const details = yield* SuiFungibleAssetOrderDetails
    const timeoutTimestamp = getTimeoutInNanoseconds24HoursFromNow()
    const salt = yield* generateSalt("evm")

    const module_name = "zkgm_relay"
    const function_name = "send"
    const tx = new Transaction()

    const function_arguments = [
      tx.object(details.ibc_store),
      tx.object(details.relay_store),
      tx.object(details.coin),
      tx.object(details.metadata),
      tx.pure.u32(sourceConfig.channelId),
      tx.pure.u64(0),
      tx.pure.u64(timeoutTimestamp),
      tx.pure.vector("u8", hexToBytes(salt)),
      tx.pure.u8(1),
      tx.pure.u8(3),
      tx.pure.vector("u8", hexToBytes(encodeAbi(fungibleAssetOrder))),
    ]

    return yield* writeContractSui(
      walletClient.client,
      walletClient.signer,
      sourceConfig.ucs03address,
      module_name,
      function_name,
      [details.typename_t], // type arguments
      function_arguments,
      tx,
    )
  })
