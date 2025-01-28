import type { Readable } from "svelte/store"
import { derived } from "svelte/store"
import type { IntentsStore } from "./intents.ts"
import type { Chain } from "$lib/types"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context"
import { isHex } from "viem"
import {
  evmChainId,
  aptosChainId,
  cosmosChainId,
  isValidEvmAddress,
  isValidBech32Address
} from "@unionlabs/client"
import type { FormFields, RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents"
import { userAddrOnChain } from "$lib/utilities/address.ts"

export type FieldErrors = Partial<Record<keyof FormFields, string>>

export interface ValidTransfer {
  sourceChain: Chain
  destinationChain: Chain
  asset: string
  receiver: string
  amount: string
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
    if ($rawIntents.asset) {
      if (!$rawIntents.amount) errors.amount = "Amount is required"
      if (!$rawIntents.receiver) errors.receiver = "Receiver is required"

      // Amount validation
      // if ($rawIntents.amount) {
      //   try {
      //     const parsedAmount = parseUnits(
      //       $rawIntents.amount,
      //       $intents.selectedAsset.metadata.decimals ?? 0
      //     )
      //     if (parsedAmount <= 0n) {
      //       errors.amount = "Amount must be greater than 0"
      //     }
      //     if (parsedAmount > BigInt($intents.selectedAsset.balance)) {
      //       errors.amount = "Amount exceeds balance"
      //     }
      //   } catch {
      //     errors.amount = "Invalid amount"
      //   }
      // }

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

    return errors
  })

  const transfer = derived(
    [rawIntents, intents, context, errors],
    ([$rawIntents, $intents, $context, $errors]) => {
      if (Object.keys($errors).length > 0) return undefined

      if (!($intents.sourceChain && $intents.destinationChain)) {
        return undefined
      }

      const sender = userAddrOnChain($context.userAddress, $intents.sourceChain)
      if (!sender) return undefined

      return {
        sourceChain: $intents.sourceChain,
        destinationChain: $intents.destinationChain,
        asset: $rawIntents.asset,
        receiver: $rawIntents.receiver,
        amount: $rawIntents.amount,
        sender
      } as ValidTransfer
    }
  )

  return derived([transfer, errors], ([$transfer, $errors]): ValidationStore => {
    const isValid = $transfer !== undefined

    return isValid
      ? { transfer: $transfer as ValidTransfer, errors: $errors, isValid: true }
      : { transfer: undefined, errors: $errors, isValid: false }
  })
}
