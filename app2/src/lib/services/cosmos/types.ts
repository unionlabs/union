//Move into schema?

export type CosmosWallet = Window["keplr"] | Window["leap"]

export interface StdSignature {
  readonly pub_key: {
    readonly type: string
    readonly value: any
  }
  readonly signature: string
}

export interface AminoMsg {
  readonly type: string
  readonly value: any
}

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

/** Cosmos chain coin type. */
export type Coin = { denom: string; amount: string }

export interface StdFee {
  readonly amount: ReadonlyArray<Coin>
  readonly gas: string
  /** The granter address that is used for paying with feegrants */
  readonly granter?: string
  /** The fee payer address. The payer must have signed the transaction. */
  readonly payer?: string
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

export type Algo = "secp256k1" | "ed25519" | "sr25519"
export interface AccountData {
  readonly address: string
  readonly algo: Algo
  readonly pubkey: Uint8Array
}

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
