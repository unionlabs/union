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
      v.description("Amount must be a valid number")
      // v.transform(input => {
      //   const gas = ''
      // })
    )
  }),
  v.forward(
    v.partialCheck(
      [["destination"], ["receiver"]],
      input => {
        if (aptosChainId.includes(input.destination)) return isHex(input.receiver)
        if (evmChainId.includes(input.destination)) return isValidEvmAddress(input.receiver)
        if (cosmosChainId.includes(input.destination)) return isValidBech32Address(input.receiver)
        return false
      },
      "`receiver` must be a valid address for the selected destination chain"
    ),
    ["receiver"]
  )
)

export type TransferSchema = v.InferOutput<typeof transferSchema>

// const test: TransferSchema = {
//   source: "11155111",
//   destination: "stride-internal-1",
//   asset: "0x0000000000000000000000000000000000000000",
//   receiver: "0x0000000000000000000000000000000000000000",
//   amount: 4
// }

// const parsed = await v.safeParseAsync(transferSchema, test)
// if (!parsed.success) {
//   console.log(parsed.issues.map(x => x.path?.[0].key))
// }
