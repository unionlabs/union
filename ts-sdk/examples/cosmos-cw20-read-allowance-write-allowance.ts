import { Effect } from "effect"
import { CosmWasmClientContext, createCosmWasmClient, createSigningCosmWasmClient, SigningCosmWasmClientContext } from "../src/cosmos/client.ts"
import { readCw20TokenInfo, readCw20Balance, readCw20Allowance, writeCw20IncreaseAllowance } from "../src/cosmos/cw20.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { SigningCosmWasmClientOptions } from "@cosmjs/cosmwasm-stargate" 
import { bech32, hex, bytes } from "@scure/base"
import { Decimal } from "@cosmjs/math";
import { executeContract } from "../src/cosmos/contract.ts"
import { GasPrice } from "@cosmjs/stargate"

const PRIVATE_KEY =
  process.env.PRIVATE_KEY || "0x0000000000000000000000000000000000000000000000000000000000000000"

export function hexToBytes(hexString: string): Uint8Array {
  return bytes("hex", hexString.indexOf("0x") === 0 ? hexString.slice(2) : hexString)
}
Effect.runPromiseExit(
  Effect.gen(function* () {
    const RPC_URL = "https://rpc.union-testnet-9.union.chain.cooking"

    const wallet = yield* Effect.tryPromise(() =>
      DirectSecp256k1Wallet.fromKey(Uint8Array.from(hexToBytes(PRIVATE_KEY)), "union")
    )
    const [firstAccount] = yield* Effect.tryPromise(() => wallet.getAccounts())

    console.info("Account address: ", firstAccount.address)

    const options: SigningCosmWasmClientOptions = {
      gasPrice: GasPrice.fromString("0.025muno")
    }
    
    const client = yield* createSigningCosmWasmClient(
      RPC_URL,
      wallet,
      options
    )
    const cosmwasm_client = yield* createCosmWasmClient(RPC_URL)

    const contractAddress = "union13pxktu2hk8pseksaaka54ngxyfmpjljrleh3cc8sxvq4dxalvttqdmdgv5";
    const spender = "union1x2jzeup7uwfxjxxrtfna2ktcugltntgu6kvc0eeayk0d82l247cqz669ee"

    const allowance = yield* readCw20Allowance(contractAddress, firstAccount.address, spender).pipe(
      Effect.provideService(CosmWasmClientContext, { client: cosmwasm_client }),
    )
    console.info("Current allowance:", allowance.toString())

    yield* writeCw20IncreaseAllowance(contractAddress, firstAccount.address, spender, "1").pipe(
      Effect.provideService(SigningCosmWasmClientContext, { client: client, address: firstAccount.address }),
    )

    const allowance_after = yield* readCw20Allowance(contractAddress, firstAccount.address, spender).pipe(
      Effect.provideService(CosmWasmClientContext, { client: cosmwasm_client }),
    )

    console.info("allowance after increasing:", allowance_after.toString())
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
