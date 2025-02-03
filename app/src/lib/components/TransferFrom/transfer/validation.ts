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

// Modified PartialTransfer to use empty values instead of undefined
export interface PartialTransfer {
  sourceChain: Chain | null
  destinationChain: Chain | null
  baseTokens: Array<{ denom: string; balance: string }>
  baseToken: { denom: string; balance: string } | null
  channel: NonNullable<ReturnType<typeof getChannelInfo>> | null
  receiver: string
  ucs03address: string
  amount: string
  parsedAmount: bigint
  sender: string
}

export interface InvalidValidationStore {
  transfer: PartialTransfer
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
    let parsedAmount: bigint = 0n

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
    ([$rawIntents, $intents, $context, $errors]): PartialTransfer => {
      const partialTransfer: PartialTransfer = {
        sourceChain: null,
        destinationChain: null,
        baseTokens: [],
        baseToken: null,
        channel: null,
        receiver: "",
        ucs03address: "",
        amount: "",
        parsedAmount: 0n,
        sender: ""
      }

      if ($intents.sourceChain) {
        partialTransfer.sourceChain = $intents.sourceChain
      }
      if ($intents.destinationChain) {
        partialTransfer.destinationChain = $intents.destinationChain
      }
      if ($intents.baseTokens) {
        partialTransfer.baseTokens = $intents.baseTokens
      }
      if ($intents.baseToken) {
        partialTransfer.baseToken = $intents.baseToken
      }
      if ($intents.channel) {
        partialTransfer.channel = $intents.channel
      }
      if ($intents.receiver) {
        partialTransfer.receiver = $intents.receiver
      }
      if ($intents.ucs03address) {
        partialTransfer.ucs03address = $intents.ucs03address
      }
      if ($rawIntents.amount) {
        partialTransfer.amount = $rawIntents.amount
      }
      if ($errors.parsedAmount) {
        partialTransfer.parsedAmount = $errors.parsedAmount
      }

      const sender = userAddrOnChain($context.userAddress, $intents.sourceChain)
      if (sender) {
        partialTransfer.sender = sender
      }

      // Check if we have all required fields for a valid transfer
      if (
        partialTransfer.sourceChain &&
        partialTransfer.destinationChain &&
        partialTransfer.baseToken &&
        partialTransfer.channel &&
        partialTransfer.ucs03address &&
        partialTransfer.receiver &&
        partialTransfer.parsedAmount &&
        partialTransfer.sender &&
        Object.keys($errors.errors).length === 0
      ) {
        return partialTransfer as ValidTransfer
      }

      return partialTransfer
    }
  )

  return derived([transfer, errors], ([$transfer, $errors]): ValidationStore => {
    if (
      $transfer.sourceChain &&
      $transfer.destinationChain &&
      $transfer.baseToken &&
      $transfer.channel &&
      $transfer.ucs03address &&
      $transfer.receiver &&
      $transfer.parsedAmount &&
      $transfer.sender &&
      Object.keys($errors.errors).length === 0
    ) {
      return {
        transfer: $transfer as ValidTransfer,
        errors: $errors.errors,
        isValid: true as const
      }
    }

    return {
      transfer: $transfer,
      errors: $errors.errors,
      isValid: false as const
    }
  })
}