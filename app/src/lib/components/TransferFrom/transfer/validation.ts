import type {FormFields} from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import {
  getChannelInfo,
  isValidBech32Address,
} from "@unionlabs/client"
import {isHex, parseUnits} from "viem"
import {
  evmChainId,
  aptosChainId,
  cosmosChainId,
  isValidEvmAddress,
} from "@unionlabs/client"
import {userAddrOnChain} from "$lib/utilities/address.ts"
import type {
  FieldErrors,
  Intents, QuoteResponse, TransferArgs, TransferContext,
} from "$lib/components/TransferFrom/transfer/types.ts"
import type {UserAddresses} from "$lib/types"
import {Result} from "neverthrow";
import type {Balances} from "$lib/stores/balances.ts";

export interface ValidationResult {
  errors: FieldErrors
  isValid: boolean
  context: TransferContext | null
  args: TransferArgs | null
}

export const checkValidation = (
  rawIntents: FormFields,
  intents: Intents,
  balance: Balances,
  userAddress: UserAddresses,
  quoteToken: Result<QuoteResponse, Error> | null
): ValidationResult => {
  const errors: FieldErrors = {}
  let parsedAmount = 0n

  // All existing validations remain the same...
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
    errors.destination = "Source and destination chains must be different"
  }

  if (rawIntents.asset && intents.baseToken) {
    if (!rawIntents.amount) errors.amount = "Amount is required"
    if (!rawIntents.receiver) errors.receiver = "Receiver is required"

    if (rawIntents.amount) {
      try {
        parsedAmount = parseUnits(
          rawIntents.amount,
          baseTokenInfo?.combined.decimals ?? 0
        )
        if (parsedAmount <= 0n) {
          errors.amount = "Amount must be greater than 0"
        }
        if (parsedAmount > BigInt(intents.baseToken.balance)) {
          errors.amount = "Amount exceeds balance"
        }
      } catch {
        errors.amount = "Invalid amount"
      }
    }

    if (rawIntents.receiver && rawIntents.destination) {
      if (aptosChainId.includes(rawIntents.destination) && !isHex(rawIntents.receiver)) {
        errors.receiver = "Invalid Aptos address"
      }
      if (
        evmChainId.includes(rawIntents.destination) &&
        !isValidEvmAddress(rawIntents.receiver)
      ) {
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

  const sender = userAddrOnChain(userAddress, intents.sourceChain) || ""

  // Check if we have all required fields
  const hasAllFields =
    intents.sourceChain &&
    intents.destinationChain &&
    intents.baseToken &&
    intents.channel &&
    intents.ucs03address &&
    intents.receiver &&
    parsedAmount &&
    sender &&
    Object.keys(errors).length === 0

  if (!hasAllFields) {
    return {
      errors,
      isValid: false,
      context: null,
      args: null
    }
  }

  // We have valid data, create context
  const transferContext = {
    channel: intents.channel,
    sourceChain: intents.sourceChain,
    destinationChain: intents.destinationChain
  }

  // Handle quote token cases
  if (!quoteToken) {
    return {
      errors,
      isValid: false,
      context: transferContext,
      args: null
    }
  }

  if (quoteToken.isErr()) {
    return {
      errors,
      isValid: false,
      context: transferContext,
      args: null
    }
  }

  if (quoteToken.value.type === "NO_QUOTE_AVAILABLE") {
    return {
      errors,
      isValid: false,
      context: transferContext,
      args: "NO_QUOTE_AVAILABLE"
    }
  }

  // Success case with all args
  return {
    errors,
    isValid: true,
    context: transferContext,
    args: {
      baseToken: intents.baseToken!.denom,
      baseAmount: parsedAmount,
      quoteToken: quoteToken.value.quote_token,
      quoteAmount: parsedAmount,
      receiver: intents.receiver,
      sourceChannelId: intents.channel!.source_channel_id,
      ucs03address: intents.ucs03address!
    }
  }
}