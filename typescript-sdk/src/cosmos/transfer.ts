import type {
  Coin,
  Prettify,
  MessageTransferWithOptionals,
  OfflineSigner as CosmosOfflineSigner
} from "../types.ts"
import { timestamp } from "../utilities/index.ts"
import { ok, err, type Result, ResultAsync } from "neverthrow"
import type { ExecuteInstruction } from "@cosmjs/cosmwasm-stargate"
import { connectCosmwasmWithSigner, connectStargateWithSigner } from "./wallet.ts"
import { assertIsDeliverTxSuccess, type MsgTransferEncodeObject } from "@cosmjs/stargate"

type CosmosTransferBaseParams = {
  rpcUrl: string
  account: CosmosOfflineSigner
  gasPrice: { amount: string; denom: string }
}

type CosmosIbcTransferParams = Prettify<
  CosmosTransferBaseParams & {
    messageTransfers: Array<MessageTransferWithOptionals>
  }
>

type CosmosCosmwasmTransferParams = Prettify<
  CosmosTransferBaseParams & {
    instructions: Array<ExecuteInstruction>
  }
>

type CosmosLocalTransferParams = Prettify<
  CosmosTransferBaseParams & {
    asset: Coin
    receiver: string
  }
>

/**
 * Make ICS-20 transfer:
 * - https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md
 * - transfer tokens from ibc-enabled chain to another ibc-enabled chain
 *
 * @example
 * ```ts
 * const transfer = await ibcTransfer(client, {
 *   gasPrice: { amount: "0.0025", denom: "muno" },
 *   account: cosmosAccount,
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 *   messageTransfers: [
 *     {
 *       sourcePort: "transfer",
 *       sourceChannel: "channel-1",
 *       sender: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
 *       token: { denom: "muno", amount: "1" },
 *       timeoutHeight: { revisionHeight: 888n, revisionNumber: 8n },
 *       receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
 *       memo: "test",
 *     },
 *   ],
 * })
 * ```
 */
export async function ibcTransfer({
  gasPrice,
  account,
  rpcUrl,
  messageTransfers
}: CosmosIbcTransferParams): Promise<Result<string, Error>> {
  const accountResult = await ResultAsync.fromPromise(account.getAccounts(), error => {
    console.error("@unionlabs/client-[ibcTransfer]_accountResult", error)
    return new Error("Failed to get accounts", { cause: error })
  }).andThen(([_account]) => {
    if (_account) return ok(_account)
    console.error("@unionlabs/client-[ibcTransfer]_accountResult", accountResult)
    return err(new Error("No account found"))
  })

  if (accountResult.isErr()) return err(accountResult.error)
  const _account = accountResult.value

  const signingClient = await connectStargateWithSigner({
    rpcUrl,
    account,
    gasPrice
  })

  if (signingClient.isErr()) return err(signingClient.error)
  const _signingClient = signingClient.value

  const response = await _signingClient.signAndBroadcast(
    _account.address,
    messageTransfers.map(
      ({ sender = _account.address, timeoutTimestamp = 0n, ...messageTransfer }) => ({
        typeUrl: "/ibc.applications.transfer.v1.MsgTransfer",
        value: { sender, timeoutTimestamp, ...messageTransfer }
      })
    ) satisfies Array<MsgTransferEncodeObject>,
    "auto"
  )

  _signingClient.disconnect()
  return ok(response.transactionHash)
}

/**
 * simulate an ibc transfer
 * @example
 * ```ts
 * const transfer = await ibcTransferSimulate(client, {
 *   gasPrice: { amount: "0.0025", denom: "muno" },
 *   account: cosmosAccount,
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 *   messageTransfers: [
 *     {
 *       sourcePort: "transfer",
 *       sourceChannel: "channel-1",
 *       sender: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
 *       token: { denom: "muno", amount: "1" },
 *       timeoutHeight: { revisionHeight: 888n, revisionNumber: 8n },
 *       receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
 *       memo: "test",
 *     },
 *   ],
 * })
 * ```
 */
export async function ibcTransferSimulate({
  gasPrice,
  account,
  rpcUrl,
  messageTransfers
}: CosmosIbcTransferParams): Promise<Result<string, Error>> {
  const accountResult = await ResultAsync.fromPromise(account.getAccounts(), error => {
    console.error("@unionlabs/client-[ibcTransferSimulate]_accountResult", error)
    return new Error("Failed to get accounts", { cause: error })
  }).andThen(([_account]) => (_account ? ok(_account) : err(new Error("No account found"))))

  if (accountResult.isErr()) return err(accountResult.error)
  const _account = accountResult.value

  const signingClient = await connectStargateWithSigner({
    rpcUrl,
    account,
    gasPrice
  })
  if (signingClient.isErr()) return err(signingClient.error)
  const _signingClient = signingClient.value

  const gas = await _signingClient.simulate(
    _account.address,
    messageTransfers.map(
      ({ sender = _account.address, timeoutTimestamp = 0n, ...messageTransfer }) => ({
        typeUrl: "/ibc.applications.transfer.v1.MsgTransfer",
        value: { sender, timeoutTimestamp, ...messageTransfer }
      })
    ) satisfies Array<MsgTransferEncodeObject>,
    "auto"
  )

  _signingClient.disconnect()
  return ok(gas.toString())
}

/**
 * transfer a wasm contract
 * @example
 * ```ts
 * const transfer = await cosmwasmTransfer(client, {
 *   gasPrice: { amount: "0.0025", denom: "muno" },
 *   account: cosmosAccount,
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 *   instructions: [
 *     {
 *       contractAddress: "0x2222222222222222222222222222222222222222",
 *       msg: {
 *         transfer: {
 *           channel: "channel-1",
 *           receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
 *           memo: "test",
 *         },
 *       },
 *       funds: [{ denom: "muno", amount: "1" }],
 *     },
 *   ],
 * })
 * ```
 */
export async function cosmwasmTransfer({
  gasPrice,
  instructions,
  account,
  rpcUrl
}: CosmosCosmwasmTransferParams): Promise<Result<string, Error>> {
  const accountResult = await ResultAsync.fromPromise(account.getAccounts(), error => {
    console.error("@unionlabs/client-[cosmwasmTransfer]_accountResult", error)
    return new Error("Failed to get accounts", { cause: error })
  }).andThen(([_account]) => (_account ? ok(_account) : err(new Error("No account found"))))

  if (accountResult.isErr()) return err(accountResult.error)
  const _account = accountResult.value

  const signingClient = await connectCosmwasmWithSigner({
    rpcUrl,
    account,
    gasPrice
  })
  if (signingClient.isErr()) return err(signingClient.error)
  const _signingClient = signingClient.value

  const response = ResultAsync.fromPromise(
    _signingClient.executeMultiple(_account.address, instructions, "auto"),
    error => {
      return new Error("failed to execute cosmwasm call", { cause: error })
    }
  ).map(res => res.transactionHash)

  _signingClient.disconnect()
  return response
}

/**
 * simulate a wasm contract
 * @example
 * ```ts
 * const transfer = await cosmwasmTransferSimulate(client, {
 *   gasPrice: { amount: "0.0025", denom: "muno" },
 *   account: cosmosAccount,
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 *   instructions: [
 *     {
 *       contractAddress: "0x2222222222222222222222222222222222222222",
 *       msg: {
 *         transfer: {
 *           channel: "channel-1",
 *           receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
 *           memo: "test",
 *         },
 *       },
 *       funds: [{ denom: "muno", amount: "1" }],
 *     },
 *   ],
 * })
 * ```
 */
export async function cosmwasmTransferSimulate({
  gasPrice,
  instructions,
  account,
  rpcUrl
}: CosmosCosmwasmTransferParams): Promise<Result<string, Error>> {
  const accountResult = await ResultAsync.fromPromise(
    account.getAccounts(),
    error => new Error("Failed to get accounts", { cause: error })
  ).andThen(([_account]) => (_account ? ok(_account) : err(new Error("No account found"))))

  if (accountResult.isErr()) return err(accountResult.error)
  const _account = accountResult.value

  const signingClient = await connectCosmwasmWithSigner({
    rpcUrl,
    account,
    gasPrice
  })
  if (signingClient.isErr()) return err(signingClient.error)
  const _signingClient = signingClient.value

  const response = await _signingClient.simulate(
    _account.address,
    instructions.map(instruction => ({
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: {
        sender: _account.address,
        contract: instruction.contractAddress,
        msg: new TextEncoder().encode(JSON.stringify(instruction.msg)),
        funds: instruction.funds
      }
    })),
    "auto"
  )

  _signingClient.disconnect()
  return ok(response.toString())
}

/**
 * transfer an asset from cosmos
 * @example
 * ```ts
 * const transfer = await cosmosSameChainTransfer(client, {
 *   asset: { denom: "muno", amount: "1" },
 *   gasPrice: { amount: "0.0025", denom: "muno" },
 *   receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
 *   account: cosmosAccount,
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 * })
 * ```
 */
export async function cosmosSameChainTransfer({
  asset,
  gasPrice,
  receiver,
  account,
  rpcUrl
}: CosmosLocalTransferParams): Promise<Result<string, Error>> {
  const accountResult = await ResultAsync.fromPromise(
    account.getAccounts(),
    () => new Error("Failed to get accounts")
  ).andThen(([_account]) => (_account ? ok(_account) : err(new Error("No account found"))))

  if (accountResult.isErr()) return err(accountResult.error)
  const _account = accountResult.value

  const signingClient = await connectStargateWithSigner({
    rpcUrl,
    account,
    gasPrice
  })
  if (signingClient.isErr()) return err(signingClient.error)
  const _signingClient = signingClient.value

  const response = await _signingClient.signAndBroadcast(
    _account.address,
    [
      {
        typeUrl: "/cosmos.bank.v1beta1.MsgSend",
        value: {
          fromAddress: _account.address,
          toAddress: receiver,
          amount: [asset]
        }
      }
    ],
    "auto",
    `${timestamp()} Sending ${asset.amount} ${asset.denom} to ${receiver}`
  )
  assertIsDeliverTxSuccess(response)
  _signingClient.disconnect()
  return ok(response.transactionHash)
}

/**
 * simulate a transfer asset from cosmos
 * @example
 * ```ts
 * const transfer = await cosmosSameChainTransferSimulate(client, {
 *   asset: { denom: "muno", amount: "1" },
 *   gasPrice: { amount: "0.0025", denom: "muno" },
 *   receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
 *   account: cosmosAccount,
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 * })
 * ```
 */
export async function cosmosSameChainTransferSimulate({
  asset,
  gasPrice,
  receiver,
  account,
  rpcUrl
}: CosmosLocalTransferParams): Promise<Result<string, Error>> {
  const accountResult = await ResultAsync.fromPromise(
    account.getAccounts(),
    () => new Error("Failed to get accounts")
  ).andThen(([_account]) => (_account ? ok(_account) : err(new Error("No account found"))))

  if (accountResult.isErr()) return err(accountResult.error)
  const _account = accountResult.value

  const signingClient = await connectStargateWithSigner({
    rpcUrl,
    account,
    gasPrice
  })
  if (signingClient.isErr()) return err(signingClient.error)
  const _signingClient = signingClient.value

  const response = await _signingClient.simulate(
    _account.address,
    [
      {
        typeUrl: "/cosmos.bank.v1beta1.MsgSend",
        value: {
          amount: [asset],
          toAddress: receiver,
          fromAddress: _account.address
        }
      }
    ],
    "auto"
  )

  _signingClient.disconnect()
  return ok(response.toString())
}
