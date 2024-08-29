import { ok, err, type Result, ResultAsync } from "neverthrow"
import {
  GasPrice,
  SigningStargateClient,
  assertIsDeliverTxSuccess,
  type MsgTransferEncodeObject
} from "@cosmjs/stargate"
import type {
  Coin,
  MessageTransferWithOptionals,
  OfflineSigner as CosmosOfflineSigner
} from "../types.ts"
import { timestamp } from "../utilities/index.ts"
import { SigningCosmWasmClient, type ExecuteInstruction } from "@cosmjs/cosmwasm-stargate"

function connectWithSigner(
  rpcUrl: string,
  account: CosmosOfflineSigner,
  gasPrice: { amount: string; denom: string }
) {
  
  return ResultAsync.fromPromise(
    SigningStargateClient.connectWithSigner(rpcUrl, account, {
      gasPrice: GasPrice.fromString(`${gasPrice.amount}${gasPrice.denom}`)
    }),
    error => new Error("Failed to connect with signer", { cause: error })
  )
}

/**
 * Make ICS-20 transfer:
 * - https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md
 * - transfer tokens from ibc-enabled chain to another ibc-enabled chain
 *
 * TODO: add JSDoc with examples
 */
export async function ibcTransfer({
  gasPrice,
  account,
  rpcUrl,
  messageTransfers
}: {
  gasPrice: { amount: string; denom: string }
  rpcUrl: string
  account: CosmosOfflineSigner
  messageTransfers: Array<MessageTransferWithOptionals>
}): Promise<Result<string, Error>> {
  const accountResult = await ResultAsync.fromPromise(
    account.getAccounts(),
    () => new Error("Failed to get accounts")
  ).andThen(([_account]) => (_account ? ok(_account) : err(new Error("No account found"))))

  if (accountResult.isErr()) return err(accountResult.error)
  const _account = accountResult.value

  return connectWithSigner(rpcUrl, account, gasPrice).andThen(signingClient =>
    ResultAsync.fromPromise(
      signingClient.signAndBroadcast(
        _account.address,
        messageTransfers.map(
          ({ sender = _account.address, timeoutTimestamp = 0n, ...messageTransfer }) => ({
            typeUrl: "/ibc.applications.transfer.v1.MsgTransfer",
            value: { sender, timeoutTimestamp, ...messageTransfer }
          })
        ) satisfies Array<MsgTransferEncodeObject>,
        "auto"
      ),
      error => new Error("Failed to sign and broadcast", { cause: error })
    ).map(response => {
      assertIsDeliverTxSuccess(response)
      signingClient.disconnect()
      return response.transactionHash
    })
  )
}

/**
 * TODO: add JSDoc with examples
 */
export async function ibcTransferSimulate({
  gasPrice,
  account,
  rpcUrl,
  messageTransfers
}: {
  gasPrice: { amount: string; denom: string }
  rpcUrl: string
  account: CosmosOfflineSigner
  messageTransfers: Array<MessageTransferWithOptionals>
}): Promise<Result<string, Error>> {
  const accountResult = await ResultAsync.fromPromise(
    account.getAccounts(),
    () => new Error("Failed to get accounts")
  ).andThen(([_account]) => (_account ? ok(_account) : err(new Error("No account found"))))

  if (accountResult.isErr()) return err(accountResult.error)
  const _account = accountResult.value

  const signingClient = await SigningStargateClient.connectWithSigner(rpcUrl, account, {
    gasPrice: GasPrice.fromString(`${gasPrice.amount}${gasPrice.denom}`)
  })

  const gas = await signingClient.simulate(
    _account.address,
    messageTransfers.map(
      ({ sender = _account.address, timeoutTimestamp = 0n, ...messageTransfer }) => ({
        typeUrl: "/ibc.applications.transfer.v1.MsgTransfer",
        value: { sender, timeoutTimestamp, ...messageTransfer }
      })
    ) satisfies Array<MsgTransferEncodeObject>,
    "auto"
  )

  signingClient.disconnect()

  return ok(gas.toString())
}

export async function cosmwasmTransfer({
  gasPrice,
  instructions,
  account,
  rpcUrl
}: {
  rpcUrl: string
  account: CosmosOfflineSigner
  instructions: Array<ExecuteInstruction>
  gasPrice: { amount: string; denom: string }
}): Promise<Result<string, Error>> {
  const accountResult = await ResultAsync.fromPromise(
    account.getAccounts(),
    () => new Error("Failed to get accounts")
  ).andThen(([_account]) => (_account ? ok(_account) : err(new Error("No account found"))))

  if (accountResult.isErr()) return err(accountResult.error)
  const _account = accountResult.value

  return ResultAsync.fromPromise(
    SigningCosmWasmClient.connectWithSigner(rpcUrl, account, {
      gasPrice: GasPrice.fromString(`${gasPrice.amount}${gasPrice.denom}`)
    }),
    error => new Error("Failed to connect with signer", { cause: error })
  ).andThen(signingClient =>
    ResultAsync.fromPromise(
      signingClient.executeMultiple(_account.address, instructions, "auto"),
      error => new Error("Failed to execute multiple", { cause: error })
    ).map(response => {
      signingClient.disconnect()
      return response.transactionHash
    })
  )
}

export async function cosmwasmTransferSimulate({
  gasPrice,
  instructions,
  account,
  rpcUrl
}: {
  rpcUrl: string
  account: CosmosOfflineSigner
  instructions: Array<ExecuteInstruction>
  gasPrice: { amount: string; denom: string }
}): Promise<Result<string, Error>> {
  const accountResult = await ResultAsync.fromPromise(
    account.getAccounts(),
    () => new Error("Failed to get accounts")
  ).andThen(([_account]) => (_account ? ok(_account) : err(new Error("No account found"))))

  if (accountResult.isErr()) return err(accountResult.error)
  const _account = accountResult.value

  return ResultAsync.fromPromise(
    SigningCosmWasmClient.connectWithSigner(rpcUrl, account, {
      gasPrice: GasPrice.fromString(`${gasPrice.amount}${gasPrice.denom}`)
    }),
    error => new Error("Failed to connect with signer", { cause: error })
  ).andThen(signingClient =>
    ResultAsync.fromPromise(
      signingClient.simulate(
        _account.address,
        instructions.map(instruction => ({
          typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
          value: {
            sender: _account.address,
            contract: instruction.contractAddress,
            msg: new TextEncoder().encode(JSON.stringify(instruction.msg)),
            funds: instruction.funds?.map(fund => ({
              denom: fund.denom,
              amount: fund.amount.toString()
            }))
          }
        })),
        "auto"
      ),
      error => new Error("Failed to simulate", { cause: error })
    ).map(gas => {
      signingClient.disconnect()
      return gas.toString()
    })
  )
}

export async function cosmosSameChainTransfer({
  asset,
  gasPrice,
  recipient,
  account,
  rpcUrl
}: {
  asset: Coin
  recipient: string
  rpcUrl: string
  account: CosmosOfflineSigner
  gasPrice: { amount: string; denom: string }
}): Promise<Result<string, Error>> {
  const accountResult = await ResultAsync.fromPromise(
    account.getAccounts(),
    () => new Error("Failed to get accounts")
  ).andThen(([_account]) => (_account ? ok(_account) : err(new Error("No account found"))))

  if (accountResult.isErr()) return err(accountResult.error)
  const _account = accountResult.value

  return ResultAsync.fromPromise(
    SigningStargateClient.connectWithSigner(rpcUrl, account, {
      gasPrice: GasPrice.fromString(`${gasPrice.amount}${gasPrice.denom}`)
    }),
    error => new Error("Failed to connect with signer", { cause: error })
  ).andThen(signingClient =>
    ResultAsync.fromPromise(
      signingClient.signAndBroadcast(
        _account.address,
        [
          {
            typeUrl: "/cosmos.bank.v1beta1.MsgSend",
            value: {
              fromAddress: _account.address,
              toAddress: recipient,
              amount: [asset]
            }
          }
        ],
        "auto",
        `${timestamp()} Sending ${asset.amount} ${asset.denom} to ${recipient}`
      ),
      error => new Error("Failed to sign and broadcast", { cause: error })
    ).map(response => {
      assertIsDeliverTxSuccess(response)
      signingClient.disconnect()
      return response.transactionHash
    })
  )
}

export async function cosmosSameChainTransferSimulate({
  asset,
  gasPrice,
  recipient,
  account,
  rpcUrl
}: {
  asset: Coin
  recipient: string
  rpcUrl: string
  account: CosmosOfflineSigner
  gasPrice: { amount: string; denom: string }
}): Promise<Result<string, Error>> {
  const accountResult = await ResultAsync.fromPromise(
    account.getAccounts(),
    () => new Error("Failed to get accounts")
  ).andThen(([_account]) => (_account ? ok(_account) : err(new Error("No account found"))))

  if (accountResult.isErr()) return err(accountResult.error)
  const _account = accountResult.value

  return ResultAsync.fromPromise(
    SigningStargateClient.connectWithSigner(rpcUrl, account, {
      gasPrice: GasPrice.fromString(`${gasPrice.amount}${gasPrice.denom}`)
    }),
    error => new Error("Failed to connect with signer", { cause: error })
  ).andThen(signingClient =>
    ResultAsync.fromPromise(
      signingClient.simulate(
        _account.address,
        [
          {
            typeUrl: "/cosmos.bank.v1beta1.MsgSend",
            value: {
              amount: [asset],
              toAddress: recipient,
              fromAddress: _account.address
            }
          }
        ],
        "auto"
      ),
      error => new Error("Failed to simulate", { cause: error })
    ).map(gas => {
      signingClient.disconnect()
      return gas.toString()
    })
  )
}
