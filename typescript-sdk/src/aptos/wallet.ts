import type {
  RawTransaction,
  AccountAuthenticator,
  InputGenerateTransactionOptions
} from "@aptos-labs/ts-sdk"
import type { Prettify, KeysToSnakeCase } from "../types.ts"

const aptosNetworks = ["mainnet", "testnet", "devnet", "local", "custom"] as const
type AptosNetwork = (typeof aptosNetworks)[number]

type AptosPublicAccountInfo = { address: string; publicKey: string }
export type AuthAccess = "key" | "wallet"

type AptosNetworkInfo = { chainId: string; name: Capitalize<AptosNetwork>; url: string }

export interface AptosWalletTransactionPayload {
  function: string
  type_info?: string
  arguments: Array<string>
  type_arguments: Array<string>
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
   * @warm don't go this route, just `signAndSubmitTransaction` to preserve your well-being
   */
  signTransaction: (args: {
    payload: AptosWalletTransactionPayload
    options?: Prettify<KeysToSnakeCase<InputGenerateTransactionOptions>>
  }) => Promise<{ accountAuthenticator: AccountAuthenticator; rawTxn: RawTransaction }>

  signAndSubmitTransaction: (args: {
    payload: AptosWalletTransactionPayload
  }) => Promise<`0x${string}`>
}
