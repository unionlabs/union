import "scripts/patch"
import "ox/window"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import type { Prettify } from "./types.ts"
import { raise } from "#utilities/index.ts"
import { Account, Ed25519PrivateKey, PrivateKey, PrivateKeyVariants } from "@aptos-labs/ts-sdk"
import { RpcSchema, RpcTransport, Provider, RpcRequest } from "ox"
import type { AptosBrowserWallet, AuthAccess } from "#aptos/wallet"
import { http, createUnionClient, type TransferAssetsParameters } from "#mod.ts"
import { Aptos, AptosConfig, Network, type ClientConfig } from "@aptos-labs/ts-sdk"
import { aptosTransferSimulate, buildSimpleTransaction, type AptosAccount } from "#aptos/transfer"
import type { Result } from "neverthrow"
/* node --import=tsx playground/aptos-to-union.ts --private-key $PRIVATE_KEY */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" } // User's private key
  }
})

const PRIVATE_KEY = values["private-key"]

if (!PRIVATE_KEY) raise("Private key not found")

const aptosAccount = Account.fromPrivateKey({
  privateKey: new Ed25519PrivateKey(
    PrivateKey.formatPrivateKey(PRIVATE_KEY, PrivateKeyVariants.Ed25519)
  )
})

// type Schema = RpcSchema.From<{
//   Request: { method: "eth_foobar"; params: [number] }
//   ReturnType: string
// }>

type AptosTransferBaseParams = {
  aptos: Aptos
  memo?: string
  amount: bigint
  receiver: string
  simulate?: boolean
  denomAddress: string
  authAccess?: AuthAccess
  destinationChainId?: string
  signer?: AptosAccount | AptosBrowserWallet
}

type AptosTransferParams = Prettify<
  AptosTransferBaseParams & {
    sourceChannel: string
    relayContractAddress: string
  }
>

type AptosTransferSimulateParams =
  | (AptosTransferBaseParams & {
      path: "SAME_CHAIN"
    })
  | (AptosTransferParams & {
      path: "CROSS_CHAIN"
    })

export type Method = "simulateTransaction" | "submitTransaction"
export type AptosMethod = `aptos_${Method}`

type AptosSchema = RpcSchema.From<{
  Request: {
    method: "aptos_simulateTransaction"
    params: [AptosTransferSimulateParams]
  }
  ReturnType: Result<string, Error>
}>

// const provider = Provider.from(transport, { schema: RpcSchema.from<Schema>() })
const store = RpcRequest.createStore()
const emitter = Provider.createEmitter()

const provider = Provider.from(
  {
    ...emitter,
    request: async _arguments => {
      const { method, params } = _arguments

      if (method === "aptos_simulateTransaction") {
        const [
          {
            memo,
            amount,
            simulate,
            receiver,
            denomAddress,
            aptos,
            destinationChainId,
            path,
            signer
          }
        ] = params as AptosSchema["Request"]["params"]

        const response = await aptosTransferSimulate({
          memo,
          aptos,
          signer,
          amount,
          receiver,
          denomAddress,
          path: "SAME_CHAIN"
        })
        return response
      }
    }
  },
  {
    includeEvents: true,
    schema: RpcSchema.from<AptosSchema>()
  }
)

const transferPayload = {
  memo: "",
  amount: 1n,
  simulate: false,
  authAccess: "key",
  account: aptosAccount,
  destinationChainId: "2",

  receiver: "0xe3579557fd55ed8fab0d1e211eb1c05d56d74650e7070b703925493c38fe2aed",
  denomAddress: "0x9935a6a334e070bcecf5b1abb1c842c123572e63e70f0539d79610c32954c06c"
}

const config = new AptosConfig({
  fullnode: "https://api.testnet.aptoslabs.com/v1",
  network: Network.TESTNET
})
const client = new Aptos(config)

provider
  .request({
    method: "aptos_simulateTransaction",
    params: [
      {
        memo: "",
        amount: 1n,
        simulate: false,
        authAccess: "key",
        aptos: client,
        signer: aptosAccount,
        destinationChainId: "2",
        path: "SAME_CHAIN",
        receiver: "0xe3579557fd55ed8fab0d1e211eb1c05d56d74650e7070b703925493c38fe2aed",
        denomAddress: "0x9935a6a334e070bcecf5b1abb1c842c123572e63e70f0539d79610c32954c06c"
      }
    ]
  })
  .then(_ => console.log(_))
  .catch(error => console.info("error thrown", error))

// const transport = RpcTransport.fromHttp(`https://1rpc.io/sepolia`, {
//   raw: true,
//   timeout: 10_000
// })

// const evmWindowProvider = Provider.from(window.ethereum)

// evmWindowProvider.request({ method: "s" })
