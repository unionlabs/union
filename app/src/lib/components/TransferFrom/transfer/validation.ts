import type { Readable } from "svelte/store"
import { derived } from "svelte/store"
import type { IntentsStore } from "./intents.ts"
import type { Chain } from "$lib/types"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context"
import { isHex, parseUnits } from "viem"
import {
  evmChainId,
  aptosChainId,
  cosmosChainId,
  isValidEvmAddress,
  isValidBech32Address,
  type getChannelInfo
} from "@unionlabs/client"
import type { FormFields, RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents"
import { userAddrOnChain } from "$lib/utilities/address.ts"

export type FieldErrors = Partial<Record<keyof FormFields, string>>

export interface ValidTransfer {
  sourceChain: Chain
  destinationChain: Chain
  baseTokens: Array<{ denom: string; balance: string }>
  baseToken: { denom: string; balance: string }
  channel: NonNullable<ReturnType<typeof getChannelInfo>>
  receiver: string
  ucs03address: string
  amount: string
  parsedAmount: bigint
  sender: string
}

export interface InvalidValidationStore {
  transfer: undefined
  errors: FieldErrors
  isValid: false
}

export interface ValidValidationStore {
  transfer: ValidTransfer
  errors: FieldErrors
  isValid: true
}

export type ValidationStore = InvalidValidationStore | ValidValidationStore

export function createValidationStore(
  rawIntents: RawIntentsStore,
  intents: Readable<IntentsStore>,
  context: Readable<ContextStore>
): Readable<ValidationStore> {
  const errors = derived([rawIntents, intents, context], ([$rawIntents, $intents, $context]) => {
    const errors: FieldErrors = {}
    let parsedAmount: bigint | undefined

    if ($rawIntents.source && !$intents.sourceChain) {
      errors.source = "Chain not supported"
    }

    if ($rawIntents.destination && !$intents.destinationChain) {
      errors.destination = "Chain not supported"
    }

    // Source chain wallet validation
    if ($intents.sourceChain) {
      if ($intents.sourceChain?.rpc_type === "evm" && !$context.userAddress.evm) {
        errors.source = "EVM wallet not connected"
      }
      if ($intents.sourceChain?.rpc_type === "cosmos" && !$context.userAddress.cosmos) {
        errors.source = "Cosmos wallet not connected"
      }
      if ($intents.sourceChain?.rpc_type === "aptos" && !$context.userAddress.aptos) {
        errors.source = "Aptos wallet not connected"
      }
    }

    if ($rawIntents.source === $rawIntents.destination) {
      errors.destination = "Source and destination chains must be different"
    }

    // Required fields when asset is selected
    if ($rawIntents.asset && $intents.baseToken) {
      if (!$rawIntents.amount) errors.amount = "Amount is required"
      if (!$rawIntents.receiver) errors.receiver = "Receiver is required"

      if ($rawIntents.amount) {
        try {
          parsedAmount = parseUnits(
            $rawIntents.amount,
            $intents.baseTokenInfo?.combined.decimals ?? 0
          )
          if (parsedAmount <= 0n) {
            errors.amount = "Amount must be greater than 0"
          }
          if (parsedAmount > BigInt($intents.baseToken.balance)) {
            errors.amount = "Amount exceeds balance"
          }
        } catch {
          errors.amount = "Invalid amount"
        }
      }

      if ($rawIntents.receiver && $rawIntents.destination) {
        if (aptosChainId.includes($rawIntents.destination) && !isHex($rawIntents.receiver)) {
          errors.receiver = "Invalid Aptos address"
        }
        if (
          evmChainId.includes($rawIntents.destination) &&
          !isValidEvmAddress($rawIntents.receiver)
        ) {
          errors.receiver = "Invalid EVM address"
        }
        if (
          cosmosChainId.includes($rawIntents.destination) &&
          !isValidBech32Address($rawIntents.receiver)
        ) {
          errors.receiver = "Invalid Cosmos address"
        }
      }
    }

    return { errors, parsedAmount }
  })

  const transfer = derived(
    [rawIntents, intents, context, errors],
    ([$rawIntents, $intents, $context, $errors]) => {
      if (Object.keys($errors.errors).length > 0) return undefined

      if (
        !(
          $intents.sourceChain &&
          $intents.destinationChain &&
          $intents.baseToken &&
          $intents.channel &&
          $intents.ucs03address &&
          $intents.receiver &&
          $errors.parsedAmount
        )
      )
        return undefined

      const sender = userAddrOnChain($context.userAddress, $intents.sourceChain)
      if (!sender) return undefined

      return {
        sourceChain: $intents.sourceChain,
        destinationChain: $intents.destinationChain,
        baseTokens: $intents.baseTokens,
        baseToken: $intents.baseToken,
        channel: $intents.channel,
        receiver: $intents.receiver,
        ucs03address: $intents.ucs03address,
        amount: $rawIntents.amount,
        parsedAmount: $errors.parsedAmount,
        sender
      } as ValidTransfer
    }
  )

  return derived([transfer, errors], ([$transfer, $errors]): ValidationStore => {
    if ($transfer !== undefined) {
      return {
        transfer: $transfer,
        errors: $errors.errors,
        isValid: true as const
      }
    }

    return {
      transfer: undefined,
      errors: $errors.errors,
      isValid: false as const
    }
  })
}
