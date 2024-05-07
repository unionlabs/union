import {
  type ExecuteResult,
  SigningCosmWasmClient,
  type ExecuteInstruction
} from "@cosmjs/cosmwasm-stargate"
import { raise } from "./utilities.ts"
import { ucs01relayAbi } from "./abi/ucs01-relay.ts"
import { Comet38Client } from "@cosmjs/tendermint-rpc"
import type { Optional, Coin, ExtractParameters } from "./types.ts"
import { hexStringToUint8Array, unionToEvmAddress } from "./convert.ts"
import type { Account, PublicActions, WalletClient, Address, Hash } from "viem"
import type { MsgTransfer } from "cosmjs-types/ibc/applications/transfer/v1/tx"
import type { AccountData, OfflineSigner as CosmosOfflineSigner } from "@cosmjs/proto-signing"
import { GasPrice, type DeliverTxResponse, type MsgTransferEncodeObject } from "@cosmjs/stargate"

type MessageTransfer = Optional<MsgTransfer, "timeoutTimestamp" | "sender">
type EvmClient = WalletClient & PublicActions

export interface IUnionClient {
  rpcClient(): Promise<Comet38Client>
  getAccount(): Promise<AccountData>
  simulateIbcMessageTransfers(messageTransfers: Array<MessageTransfer>): Promise<number>
  ibcMessageTransfers(messageTransfers: Array<MessageTransfer>): Promise<DeliverTxResponse>
  cosmwasmMessageExecuteContract(instructions: Array<ExecuteInstruction>): Promise<ExecuteResult>
  transferEvmAsset(parameters: {
    receiver: `union${string}`
    denomAddress: Address
    sourcePort: string
    sourceChannel: string
    amount: bigint
    account: Account | Address
    contractAddress?: Address
    simulate?: true
  }): Promise<Hash>
  transferAssets<Kind extends "ibc" | "cosmwasm">({
    kind
  }: { kind: Kind } & (Kind extends "ibc"
    ? { messageTransfers: Array<MessageTransfer> }
    : { instructions: Array<ExecuteInstruction> })): Promise<DeliverTxResponse | ExecuteResult>
}

/**
 * A client for interacting with Cosmos SDK-based chains,
 * sending IBC messages and executing CosmWasm contracts.
 *
 * @example
 * ```ts
 * // Passing a private key or mnemonic directly
 * const unionClient = await UnionClient.connectWithSecret({
 *   bech32Prefix: "osmo",
 *   chainId: "osmo-test-5",
 *   gas: { denom: "uosmo", amount: "0.0025" },
 *   rpcUrl: "https://rpc.testnet.osmosis.zone:443",
 *   secretType: "key", // can be "mnemonic" or "key"
 *   privateKeyOrMnemonic: "<PASS-PRIVATE-KEY>"
 * })
 *
 * const ibcMessageTransfer = await unionClient.ibcMessageTransfers([
 *   {
 *     receiver: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
 *     sourcePort: "transfer",
 *     sourceChannel: "channel-7775",
 *     token: { denom: "uosmo", amount: "2" },
 *     memo: "sending wrapped OSMO from Osmosis to Union",
 *     timeoutHeight: { revisionHeight: 88888888n, revisionNumber: 8n }
 *   }
 * ])
 *
 * console.info(ibcMessageTransfer.transactionHash)
 * ```
 * @example

 * ```ts
 * // Using browser wallet signer
 * // Leap Wallet works too `window.leap?.`
 * const offlineSigner = window.keplr?.getOfflineSigner('osmo-test-5', {})

 * const unionClient = new UnionClient({
 *   offlineSigner,
 *   bech32Prefix: "osmo",
 *   chainId: "osmo-test-5",
 *   gas: { denom: "uosmo", amount: "0.0025" },
 *   rpcUrl: "https://rpc.testnet.osmosis.zone:443",
 * })

 * const ibcMessageTransfer = await unionClient.ibcMessageTransfers([
 *   // ...
 * ])

 * console.info(ibcMessageTransfer.transactionHash)
 * ```
 */
export class UnionClient implements IUnionClient {
  /** Cosmos */
  #rpcUrl: string
  public chainId: string
  public bech32Prefix: string
  #cosmosOfflineSigner: CosmosOfflineSigner
  #gas?: Coin
  /** EVM */
  #evmSigner?: EvmClient
  #UCS01_ADDRESS: Address = "0x3d0EB16AD2619666dbde1921282cd885b58eEefE" satisfies Address
  #UCS02_ADDRESS: Address = "0xB455b205106c9b72E967399E15EFd8A025FD4A90" satisfies Address
  #COMETBLS_ADDRESS: Address = "0xf906A05a25bf5b61a5e4Ff24bE9122E2Cea5F1E3" satisfies Address
  #IBC_HANDLER_ADDRESS: Address = "0x6B6b60a68b8DCbB170F25045974d10098917F816" satisfies Address
  #UNION_UCS01_ADDRESS = "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7"
  constructor(arguments_: {
    rpcUrl: string
    chainId: string
    bech32Prefix: string
    cosmosOfflineSigner: CosmosOfflineSigner
    privateKeyOrMnemonic?: string
    gas?: Coin
    evmSigner?: EvmClient
  }) {
    this.#rpcUrl = arguments_.rpcUrl
    this.chainId = arguments_.chainId
    this.bech32Prefix = arguments_.bech32Prefix
    this.#cosmosOfflineSigner = arguments_.cosmosOfflineSigner
    this.#gas = arguments_.gas
    this.#evmSigner = arguments_.evmSigner
  }

  #gasPrice = (gas = this.#gas) => GasPrice.fromString(`${gas?.amount}${gas?.denom}`)

  /**
   * Connect to the RPC client of the chain.
   */
  public rpcClient = async (): Promise<Comet38Client> => await Comet38Client.connect(this.#rpcUrl)

  static async connectWithSecret(
    params: Required<
      Omit<ExtractParameters<typeof UnionClient>, "cosmosOfflineSigner" | "evmSigner">
    > & {
      secretType: "mnemonic" | "key"
      evmSigner?: EvmClient
    }
  ): Promise<UnionClient> {
    if (!params.privateKeyOrMnemonic) throw new Error("privateKeyOrMnemonic is required")
    let cosmosOfflineSigner: CosmosOfflineSigner
    if (params.secretType === "key") {
      const { DirectSecp256k1Wallet } = await import("@cosmjs/proto-signing")
      cosmosOfflineSigner = await DirectSecp256k1Wallet.fromKey(
        Uint8Array.from(hexStringToUint8Array(params.privateKeyOrMnemonic)),
        params.bech32Prefix
      )
    } else {
      const { DirectSecp256k1HdWallet } = await import("@cosmjs/proto-signing")
      cosmosOfflineSigner = await DirectSecp256k1HdWallet.fromMnemonic(
        params.privateKeyOrMnemonic,
        {
          prefix: params.bech32Prefix
        }
      )
    }
    return new UnionClient({ ...params, cosmosOfflineSigner })
  }

  async getAccount(): Promise<AccountData> {
    const [account] = await this.#cosmosOfflineSigner.getAccounts()
    if (!account) throw new Error("Account not found")
    return account
  }

  protected getEvmAccount = (): Account =>
    this.#evmSigner?.account ?? raise("EVM account not found")

  #signingCosmWasmClient = async () =>
    await SigningCosmWasmClient.connectWithSigner(this.#rpcUrl, this.#cosmosOfflineSigner, {
      gasPrice: this.#gasPrice()
    })

  public async simulateIbcMessageTransfers(
    messageTransfers: Array<MessageTransfer>
  ): Promise<number> {
    const { address: signerAddress } = await this.getAccount()
    const cosmwasmClient = await this.#signingCosmWasmClient()
    const response = await cosmwasmClient.simulate(
      signerAddress,
      messageTransfers.map(
        ({ sender = signerAddress, timeoutTimestamp = 0n, ...messageTransfer }) => ({
          typeUrl: "/ibc.applications.transfer.v1.MsgTransfer",
          value: { sender, timeoutTimestamp, ...messageTransfer }
        })
      ) satisfies Array<MsgTransferEncodeObject>,
      "auto"
    )
    return response
  }

  /**
   * Executes `/ibc.applications.transfer.v1.MsgTransfer`, accepts an array of `MessageTransfer`.
   */
  public async ibcMessageTransfers(
    messageTransfers: Array<MessageTransfer>
  ): Promise<DeliverTxResponse> {
    const { address: signerAddress } = await this.getAccount()
    const cosmwasmClient = await this.#signingCosmWasmClient()
    const response = await cosmwasmClient.signAndBroadcast(
      signerAddress,
      messageTransfers.map(
        ({ sender = signerAddress, timeoutTimestamp = 0n, ...messageTransfer }) => ({
          typeUrl: "/ibc.applications.transfer.v1.MsgTransfer",
          value: { sender, timeoutTimestamp, ...messageTransfer }
        })
      ) satisfies Array<MsgTransferEncodeObject>,
      "auto"
    )
    return response
  }

  /**
   * Executes `/cosmwasm.wasm.v1.MsgExecuteContract`, accepts an array of `ExecuteInstruction`.
   */
  public async cosmwasmMessageExecuteContract(
    instructions: Array<ExecuteInstruction>
  ): Promise<ExecuteResult> {
    const { address: signerAddress } = await this.getAccount()
    const cosmwasmClient = await this.#signingCosmWasmClient()
    const response = await cosmwasmClient.executeMultiple(signerAddress, instructions, "auto")
    return response
  }

  public async transferAssets<Kind extends "ibc" | "cosmwasm">(
    params: { kind: Kind } & (Kind extends "ibc"
      ? { messageTransfers: Array<MessageTransfer> }
      : { instructions: Array<ExecuteInstruction> })
  ): Promise<DeliverTxResponse | ExecuteResult> {
    if (params.kind === "ibc") {
      return await this.ibcMessageTransfers(
        (params as { messageTransfers: Array<MessageTransfer> }).messageTransfers
      )
    }
    return await this.cosmwasmMessageExecuteContract(
      (params as { instructions: Array<ExecuteInstruction> }).instructions
    )
  }

  public async transferEvmAsset({
    account,
    receiver,
    denomAddress,
    sourceChannel,
    sourcePort,
    amount,
    contractAddress = this.#UCS01_ADDRESS,
    simulate = true
  }: Parameters<IUnionClient["transferEvmAsset"]>[0]): Promise<Hash> {
    const signer = this.#evmSigner ?? raise("EVM signer not found")
    const writeContractParameters = {
      account,
      abi: ucs01relayAbi,
      functionName: "send",
      address: contractAddress,
      chain: signer.chain,
      args: [
        sourcePort,
        sourceChannel,
        unionToEvmAddress(receiver),
        [{ denom: denomAddress, amount }],
        8n,
        10_000_000n
      ]
    } as const
    if (!simulate) return await signer.writeContract(writeContractParameters)
    const { request } = await signer.simulateContract(writeContractParameters)
    return await signer.writeContract(request)
  }
}
