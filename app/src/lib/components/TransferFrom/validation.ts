import {
  evmChainId,
  aptosChainId,
  cosmosChainId,
  isValidEvmAddress,
  isValidBech32Address
} from "@unionlabs/client"
import * as v from "valibot"
import { isHex } from "viem"

const chainId = [...evmChainId, ...cosmosChainId, ...aptosChainId]

export const transferSchema = v.pipe(
  v.object({
    source: v.pipe(
      v.string(),
      v.trim(),
      v.picklist(chainId, "Invalid source chain id"),
      v.title("Source")
    ),
    destination: v.pipe(
      v.string(),
      v.trim(),
      v.picklist(chainId, "Invalid destination chain id"),
      v.title("Destination")
    ),
    receiver: v.pipe(
      v.string(),
      v.trim(),
      v.title("Receiver"),
      v.description("Receiver must be a valid address")
    ),
    asset: v.pipe(
      v.string(),
      v.trim(),
      v.title("Asset"),
      v.description("Asset must be a valid asset contract address")
    ),
    amount: v.pipe(
      v.string(),
      v.trim(),
      v.title("Amount"),
      v.description("Amount must be a valid number greater than 0"),
      v.check(value => {
        const parsedValue = Number.parseFloat(value)
        return !Number.isNaN(parsedValue) && parsedValue > 0
      }, "Amount must be greater than 0")
    )
  }),
  v.forward(
    v.partialCheck(
      [["destination"], ["receiver"]], // Validate receiver against destination chain
      input => {
        if (aptosChainId.includes(input.destination)) {
          return isHex(input.receiver) // Aptos: Hexadecimal address
        }
        if (evmChainId.includes(input.destination)) {
          return isValidEvmAddress(input.receiver) // EVM: Valid Ethereum address
        }
        if (cosmosChainId.includes(input.destination)) {
          return isValidBech32Address(input.receiver) // Cosmos: Bech32 address
        }
        return false // If destination doesn't match any chain, fail validation
      },
      "`receiver` must be a valid address for the selected destination chain"
    ),
    ["receiver"]
  )
)

export type TransferSchema = v.InferOutput<typeof transferSchema>
