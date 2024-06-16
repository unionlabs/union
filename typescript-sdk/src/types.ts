export type ErrorType<name extends string = "Error"> = Error & { name: name }

export type RpcRequest = {
  jsonrpc?: "2.0" | undefined
  method: string
  params?: any | undefined
  id?: number | undefined
}

export type HexAddress = `0x${string}`
export type Bech32Address<T extends string = string> = `${T}${string}`

export type OfflineSigner = OfflineAminoSigner | OfflineDirectSigner

export interface OfflineDirectSigner {
  readonly getAccounts: () => Promise<ReadonlyArray<AccountData>>
  readonly signDirect: (signerAddress: string, signDoc: SignDoc) => Promise<DirectSignResponse>
}

export interface OfflineAminoSigner {
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
  readonly signAmino: (signerAddress: string, signDoc: StdSignDoc) => Promise<AminoSignResponse>
}

export interface DirectSignResponse {
  /**
   * The sign doc that was signed.
   * This may be different from the input signDoc when the signer modifies it as part of the signing process.
   */
  readonly signed: SignDoc
  readonly signature: StdSignature
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

export type Coin = { denom: string; amount: string }

export interface AminoMsg {
  readonly type: string
  readonly value: any
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

export interface MessageTransferWithOptionals
  extends Optional<MessageTransfer, "timeoutTimestamp" | "sender"> {}

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
  readonly pub_key: Pubkey
  readonly signature: string
}

export interface AminoSignResponse {
  /**
   * The sign doc that was signed.
   * This may be different from the input signDoc when the signer modifies it as part of the signing process.
   */
  readonly signed: StdSignDoc
  readonly signature: StdSignature
}

export interface Pubkey {
  readonly type: string
  readonly value: any
}

export type Pretty<T> = {
  [K in keyof T]: T[K]
} & {}

export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>

export type ExtractParameters<T> = T extends new (..._args: infer P) => any ? P[0] : never

export type NoRepetition<U extends string, ResultT extends Array<any> = []> =
  | ResultT
  | {
      [k in U]: NoRepetition<Exclude<U, k>, [k, ...ResultT]>
    }[U]
