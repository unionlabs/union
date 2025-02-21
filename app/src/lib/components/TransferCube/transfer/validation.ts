import type { FormFields } from "$lib/components/TransferCube/transfer/raw-intents.ts"
import { isValidBech32Address } from "@unionlabs/client"
import { isHex, parseUnits } from "viem"
import { evmChainId, aptosChainId, cosmosChainId, isValidEvmAddress } from "@unionlabs/client"
import type { Chain } from "$lib/types"
import type { Balances } from "$lib/stores/balances.ts"
import type { UserAddresses } from "$lib/types"
import type { getChannelInfo } from "@unionlabs/client"
import type { Intents } from "$lib/components/TransferCube/transfer/types.ts"

export type FieldErrors = Partial<Record<keyof FormFields, string>>

export interface BaseTransferArgs {
  baseToken: string
  baseAmount: bigint
  quoteToken: string
  quoteAmount: bigint
  receiver: string
  sourceChannelId: number
  ucs03address: string
}

export interface EvmTransferArgs extends BaseTransferArgs {
  chainType: "evm"
  wethQuoteToken: string
}

export interface CosmosTransferArgs extends BaseTransferArgs {
  chainType: "cosmos"
}

export interface AptosTransferArgs extends BaseTransferArgs {
  chainType: "aptos"
}

export type TransferArgs = EvmTransferArgs | CosmosTransferArgs | AptosTransferArgs

export interface TransferContext {
  channel: NonNullable<ReturnType<typeof getChannelInfo>>
  sourceChain: Chain
  destinationChain: Chain
}

export interface InvalidValidationResult {
  errors: FieldErrors
  isValid: false
  context: TransferContext | null
  args: null
}

export interface ValidValidationResult {
  errors: FieldErrors
  isValid: true
  context: TransferContext
  args: TransferArgs
}

export type ValidationResult = InvalidValidationResult | ValidValidationResult

export const checkValidation = (
  rawIntents: FormFields,
  intents: Intents,
  _balances: Balances,
  userAddress: UserAddresses
): ValidationResult => {
  const errors: FieldErrors = {}
  let parsedAmount = 0n

  if (rawIntents.source && !intents.sourceChain) {
    errors.source = "Chain not supported"
  }

  if (rawIntents.destination && !intents.destinationChain) {
    errors.destination = "Chain not supported"
  }

  if (intents.sourceChain) {
    if (intents.sourceChain?.rpc_type === "evm" && !userAddress.evm) {
      errors.source = "EVM wallet not connected"
    }
    if (intents.sourceChain?.rpc_type === "cosmos" && !userAddress.cosmos) {
      errors.source = "Cosmos wallet not connected"
    }
    if (intents.sourceChain?.rpc_type === "aptos" && !userAddress.aptos) {
      errors.source = "Aptos wallet not connected"
    }
  }

  if (rawIntents.source === rawIntents.destination) {
    errors.destination = "Chains must be different"
  }

  if (rawIntents.asset && intents.baseToken) {
    if (!rawIntents.amount) errors.amount = "Amount is required"
    if (!rawIntents.receiver) errors.receiver = "Receiver is required"

    if (rawIntents.amount) {
      const validNumberFormat = /^\d*\.?\d*$/.test(rawIntents.amount)
      if (validNumberFormat) {
        try {
          parsedAmount = parseUnits(
            rawIntents.amount,
            intents.baseTokenInfo?.combined.decimals ?? 0
          )
          if (parsedAmount < 0n) {
            errors.amount = "Amount must be greater than 0"
          }
          if (
            intents.baseToken?.balance?.kind === "balance" &&
            parsedAmount > BigInt(intents.baseToken.balance.amount || 0)
          ) {
            errors.amount = "Amount exceeds balance"
          }
        } catch {
          errors.amount = "Invalid amount"
        }
      } else {
        errors.amount = "Invalid amount format"
      }
    }

    if (rawIntents.receiver && rawIntents.destination) {
      if (aptosChainId.includes(rawIntents.destination) && !isHex(rawIntents.receiver)) {
        errors.receiver = "Invalid Aptos address"
      }
      if (evmChainId.includes(rawIntents.destination) && !isValidEvmAddress(rawIntents.receiver)) {
        errors.receiver = "Invalid EVM address"
      }
      if (
        cosmosChainId.includes(rawIntents.destination) &&
        !isValidBech32Address(rawIntents.receiver)
      ) {
        errors.receiver = "Invalid Cosmos address"
      }
    }
  }

  // Create context first if we have the required chain info
  if (!(intents.sourceChain && intents.destinationChain && intents.channel)) {
    return {
      errors,
      isValid: false,
      context: null,
      args: null
    }
  }

  const context: TransferContext = {
    channel: intents.channel,
    sourceChain: intents.sourceChain,
    destinationChain: intents.destinationChain
  }

  // Check for errors before proceeding
  if (Object.keys(errors).length > 0) {
    return {
      errors,
      isValid: false,
      context,
      args: null
    }
  }

  // Then check all required fields including quote token
  if (
    !(
      intents.baseToken &&
      intents.ucs03address &&
      intents.receiver &&
      parsedAmount &&
      intents.quoteToken
    )
  ) {
    return {
      errors,
      isValid: false,
      context,
      args: null
    }
  }

  const baseArgs: BaseTransferArgs = {
    baseToken: intents.baseToken.denom,
    baseAmount: parsedAmount,
    quoteToken: intents.quoteToken,
    quoteAmount: parsedAmount,
    receiver: intents.receiver,
    sourceChannelId: intents.channel.source_channel_id,
    ucs03address: intents.ucs03address,
  }

  // Create chain-specific args
  let args: TransferArgs
  switch (intents.sourceChain.rpc_type) {
    case "evm":
      if (!intents.wethQuoteToken) {
        return {
          errors: { ...errors, asset: "WETH token required for EVM chains" },
          isValid: false,
          context,
          args: null
        }
      }
      args = {
        ...baseArgs,
        chainType: "evm",
        wethQuoteToken: intents.wethQuoteToken
      }
      break

    case "cosmos":
      args = {
        ...baseArgs,
        chainType: "cosmos"
      }
      break

    case "aptos":
      args = {
        ...baseArgs,
        chainType: "aptos"
      }
      break

    default:
      return {
        errors: { ...errors, source: "Unsupported chain type" },
        isValid: false,
        context,
        args: null
      }
  }

  return {
    errors,
    isValid: true,
    context,
    args
  }
}