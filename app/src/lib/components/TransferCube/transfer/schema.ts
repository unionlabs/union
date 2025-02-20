//CURRENTLY NOT USED

import {
  evmChainId,
  aptosChainId,
  cosmosChainId,
  isValidEvmAddress,
  isValidBech32Address
} from "@unionlabs/client"
import * as v from "valibot"
import { isHex, parseUnits } from "viem"

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
      v.description("Amount must be a valid number greater than 0 and not exceed balance")
    ),
    balance: v.pipe(v.bigint(), v.title("Balance"), v.description("Current balance for the asset")),
    decimals: v.fallback(v.number(), 0)
  }),
  v.forward(
    v.partialCheck(
      [["amount"], ["balance"], ["decimals"]],
      input => {
        try {
          const parseAmount = (amount: string, dec: number): bigint => {
            try {
              if (dec === 0) {
                const wholeNumber = Math.floor(Number.parseFloat(amount)).toString()
                return parseUnits(wholeNumber, 0)
              }
              return parseUnits(amount, dec)
            } catch {
              return 0n
            }
          }
          const amountBigInt = parseAmount(input.amount, input.decimals)
          return amountBigInt > 0n && amountBigInt <= input.balance
        } catch (error) {
          console.error("Validation error:", error)
          return false
        }
      },
      "Amount must be greater than 0 and not exceed available balance"
    ),
    ["amount"]
  ),
  v.forward(
    v.partialCheck(
      [["destination"], ["receiver"]],
      input => {
        if (aptosChainId.includes(input.destination)) {
          return isHex(input.receiver)
        }
        if (evmChainId.includes(input.destination)) {
          return isValidEvmAddress(input.receiver)
        }
        if (cosmosChainId.includes(input.destination)) {
          return isValidBech32Address(input.receiver)
        }
        return false
      },
      "`receiver` must be a valid address for the selected destination chain"
    ),
    ["receiver"]
  )
)

export type TransferSchema = v.InferOutput<typeof transferSchema>
