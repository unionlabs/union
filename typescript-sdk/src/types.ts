import type { Account as ViemAccount, Address } from "viem"
import type { evmChainId, EvmChainId } from "./evm/client.ts"
import type { cosmosChainId, CosmosChainId } from "./cosmos/client.ts"
import type { aptosChainId, AptosChainId, AptosAccount } from "./aptos/client.ts"

export type LooseAutocomplete<T extends string> = T | Omit<string, T>

export type SelectFields<T, K extends keyof T> = T extends any ? Pick<T, K> : never

export type MergeUnion<T> = { [K in keyof T]: T[K] }

export type CamelToSnakeCase<S extends string> = S extends `${infer T}${infer U}`
  ? `${T extends Capitalize<T> ? "_" : ""}${Lowercase<T>}${CamelToSnakeCase<U>}`
  : S

export type KeysToSnakeCase<T extends object> = {
  [K in keyof T as CamelToSnakeCase<K & string>]: T[K]
}

export type Prettify<T> = { [K in keyof T]: T[K] } & {}

export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>

export type ChainId =
  | (typeof evmChainId)[number]
  | (typeof cosmosChainId)[number]
  | (typeof aptosChainId)[number]

export type TransferAssetsParametersLegacy<
  CHAIN_ID extends EvmChainId | CosmosChainId | AptosChainId
> = {
  memo?: string
  amount: bigint
  receiver: string
  autoApprove?: boolean
  destinationChainId: ChainId
} & (CHAIN_ID extends CosmosChainId
  ? {
      denomAddress: string
      account?: OfflineSigner
      relayContractAddress?: string
      gasPrice?: { amount: string; denom: string }
    }
  : CHAIN_ID extends EvmChainId
    ? {
        simulate?: boolean
        denomAddress: Address
        relayContractAddress?: Address
        account?: ViemAccount | undefined
      }
    : CHAIN_ID extends AptosChainId
      ? {
          simulate?: boolean
          denomAddress: string
          account?: AptosAccount
          authAccess: "key" | "wallet"
          relayContractAddress?: string
          gasPrice?: { amount: string; denom: string }
        }
      : undefined)

export type TransferAssetParameters<CHAIN_ID extends EvmChainId | CosmosChainId | AptosChainId> = {
  baseAmount: bigint
  baseToken: string
  quoteAmount: bigint
  quoteToken: string
  receiver: string
  sourceChannelId: number
  wethQuoteToken: Hex
} & (CHAIN_ID extends CosmosChainId
  ? {
      ucs03address: string
    }
  : CHAIN_ID extends EvmChainId
    ? {
        ucs03address: HexAddress
      }
    : undefined)

/** Currently supported networks. */
export type Network = "evm" | "cosmos" | "aptos"

export type Hex = `0x${string}`

/** Hex address of the form `0x${string}`. Used for EVM addresses. */
export type HexAddress = `0x${string}`

/** Bech32 address of the form `${string}1${string}`. Used for Cosmos addresses. */
export type Bech32Address<T extends string = string> = `${T}1${string}`

/** Cosmos chain coin type. */
export type Coin = { denom: string; amount: string }

/** Offline signer is the account for a Cosmos chain. */
export type OfflineSigner =
  | {
      /** getAccounts returns the list of accounts available on this signer. */
      readonly getAccounts: () => Promise<ReadonlyArray<AccountData>>
      readonly signDirect: (
        signerAddress: string,
        signDoc: SignDoc
      ) => Promise<{
        /**
         * The sign doc that was signed.
         * This may be different from the input signDoc when the signer modifies it as part of the signing process.
         */
        readonly signed: SignDoc
        readonly signature: StdSignature
      }>
    }
  | {
      readonly getAccounts: () => Promise<ReadonlyArray<AccountData>>
      /**
       * Request signature from whichever key corresponds to provided bech32-encoded address. Rejects if not enabled.
       *
       * The signer implementation may offer the user the ability to override parts of the signDoc. It must
       * return the doc that was signed in the response.
       *
       * @param signerAddress The address of the account that should sign the transaction
       * @param signDoc The content that should be signed
       */
      readonly signAmino: (
        signerAddress: string,
        signDoc: StdSignDoc
      ) => Promise<{
        /**
         * The sign doc that was signed.
         * This may be different from the input signDoc when the signer modifies it as part of the signing process.
         */
        readonly signed: StdSignDoc
        readonly signature: StdSignature
      }>
    }

export type Algo = "secp256k1" | "ed25519" | "sr25519"

/**
 * `address`: a printable address (typically bech32 encoded)
 */
export interface AccountData {
  readonly address: string
  readonly algo: Algo
  readonly pubkey: Uint8Array
}

/**
 * `revisionNumber`: the revision number that the client is currently on
 * `revisionHeight`: the height within the given revision
 */
export interface Height {
  revisionNumber: bigint
  revisionHeight: bigint
}

export interface AminoMsg {
  readonly type: string
  readonly value: any
}

/**
 * SignDoc is the type used for generating sign bytes for SIGN_MODE_DIRECT.
 */
export interface SignDoc {
  /**
   * body_bytes is protobuf serialization of a TxBody that matches the
   * representation in TxRaw.
   */
  bodyBytes: Uint8Array
  /**
   * auth_info_bytes is a protobuf serialization of an AuthInfo that matches the
   * representation in TxRaw.
   */
  authInfoBytes: Uint8Array
  /**
   * chain_id is the unique identifier of the chain this transaction targets.
   * It prevents signed transactions from being used on another chain by an
   * attacker
   */
  chainId: string
  /** account_number is the account number of the account in state */
  accountNumber: bigint
}

export interface StdSignature {
  readonly pub_key: {
    readonly type: string
    readonly value: any
  }
  readonly signature: string
}

export interface StdSignDoc {
  readonly chain_id: string
  readonly account_number: string
  readonly sequence: string
  readonly fee: StdFee
  readonly msgs: ReadonlyArray<AminoMsg>
  readonly memo: string
  readonly timeout_height?: string
}

export interface StdFee {
  readonly amount: ReadonlyArray<Coin>
  readonly gas: string
  /** The granter address that is used for paying with feegrants */
  readonly granter?: string
  /** The fee payer address. The payer must have signed the transaction. */
  readonly payer?: string
}

export interface MessageTransfer {
  token: Coin
  memo: string
  sender: string
  receiver: string
  sourcePort: string
  sourceChannel: string
  timeoutHeight: Height
  timeoutTimestamp: bigint
}

export type MessageTransferWithOptionals = Optional<MessageTransfer, "timeoutTimestamp" | "sender">
