import type {
  RawTransaction,
  AccountAuthenticator,
  UserTransactionResponse,
  InputGenerateTransactionOptions
} from "@aptos-labs/ts-sdk"

export type Prettify<T> = { [K in keyof T]: T[K] } & {}

export type CamelToSnakeCase<S extends string> = S extends `${infer T}${infer U}`
  ? `${T extends Capitalize<T> ? "_" : ""}${Lowercase<T>}${CamelToSnakeCase<U>}`
  : S
export type KeysToSnakeCase<T extends object> = {
  [K in keyof T as CamelToSnakeCase<K & string>]: T[K]
}
const aptosNetworks = ["mainnet", "testnet", "devnet", "local", "custom"] as const
type AptosNetwork = (typeof aptosNetworks)[number]

type AptosPublicAccountInfo = { address: string; publicKey: string }
export type AuthAccess = "key" | "wallet"

type AptosNetworkInfo = { chainId: string; name: Capitalize<AptosNetwork>; url: string }

export interface AptosWalletTransactionPayload {
  function: string
  type_info?: string
  type_arguments: Array<string>
  arguments: Array<string | [string]>
}

export interface AptosBrowserWallet {
  onDisconnect: () => void
  disconnect: () => Promise<void>
  isConnected: () => Promise<boolean>
  connect: () => Promise<AptosPublicAccountInfo>
  account: () => Promise<AptosPublicAccountInfo>
  getAccount: () => Promise<AptosPublicAccountInfo>
  network: () => Promise<Capitalize<AptosNetwork>>
  getNetwork: () => Promise<AptosNetworkInfo>
  onAccountChange: (
    callback: (account: AptosPublicAccountInfo & { type?: unknown }) => void
  ) => void
  onNetworkChange: (callback: (network: AptosNetworkInfo) => void) => void

  /**
   * @note
   * for some reason, aptos wallets use snake case for tx payload params
   * whereas aptos sdk uses camel case
   *
   * @warn don't go this route, just `signAndSubmitTransaction` to preserve your well-being
   */
  signTransaction: (args: {
    payload: AptosWalletTransactionPayload
    options?: Prettify<KeysToSnakeCase<InputGenerateTransactionOptions>>
  }) => Promise<{ accountAuthenticator: AccountAuthenticator; rawTxn: RawTransaction }>

  signAndSubmitTransaction: (args: {
    payload: AptosWalletTransactionPayload
  }) => Promise<UserTransactionResponse>
}
