import {
  type Aptos,
  Ed25519PublicKey,
  SimpleTransaction,
  type RawTransaction,
  type AccountAuthenticator,
  type Account as AptosAccount,
  type InputGenerateTransactionOptions,
  type InputGenerateTransactionPayloadData
} from "@aptos-labs/ts-sdk"
import { err, ok, type Result } from "neverthrow"
import type { Prettify, KeysToSnakeCase } from "../types.ts"
import { isValidBech32Address } from "../utilities/address.ts"
import { bech32ToBytes, hexStringToUint8Array } from "../convert.ts"

export type {
  AuthAccess,
  AptosAccount,
  AptosBrowserWallet,
  AptosTransferParams,
  AptosPublicAccountInfo,
  AuthAccess as AptosAuthAccess
}

const aptosNetworks = ["mainnet", "testnet", "devnet", "local", "custom"] as const
type AptosNetwork = (typeof aptosNetworks)[number]

type AptosPublicAccountInfo = { address: string; publicKey: string }
type AptosNetworkInfo = { chainId: string; name: Capitalize<AptosNetwork>; url: string }

interface AptosBrowserWallet {
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
   */
  signTransaction: (transactionParameters: {
    payload: Prettify<KeysToSnakeCase<InputGenerateTransactionPayloadData>>
    options?: Prettify<KeysToSnakeCase<InputGenerateTransactionOptions>>
  }) => Promise<{ accountAuthenticator: AccountAuthenticator; rawTxn: RawTransaction }>
}

type AuthAccess = "key" | "wallet"

type AptosTransferBaseParams = {
  aptos: Aptos
  memo?: string
  amount: bigint
  receiver: string
  simulate?: boolean
  denomAddress: string
  destinationChainId?: string
} & (
  | {
      authAccess: "key"
      account?: AptosAccount
    }
  | {
      authAccess: "wallet"
      account?: AptosPublicAccountInfo
      sign: AptosBrowserWallet["signTransaction"]
    }
)

type AptosTransferParams = AptosTransferBaseParams & {
  sourceChannel: string
  relayContractAddress: string
}

export async function waitForTransactionReceipt({
  aptos,
  hash
}: { aptos: Aptos; hash: string }): Promise<Result<string, Error>> {
  const transactionResult = await aptos.waitForTransaction({
    transactionHash: hash,
    options: { checkSuccess: false }
  })
  if (!transactionResult.success) {
    return err(new Error(transactionResult.vm_status || "waiting for transaction failed"))
  }

  return ok(transactionResult.hash)
}

/**
 * Transfer an asset from the Aptos blockchain (e.g., Aptos) using the IBC `send` function, similar to EVM implementation.
 *
 * @example
 * ```ts
 * const transfer = await transferAssetFromAptos({
 *   memo: "test",
 *   amount: BigInt(1),
 *   account: "0xSenderAccountAddress",
 *   receiver: "HEX_PR_BECH32_ADDRESS",
 *   denomAddress: "0x1::aptos_coin::AptosCoin",
 *   sourceChannel: "channel-1",
 *   relayContractAddress: "0x2222222222222222222222222222222222222222",
 *   baseUrl: "https://fullnode.devnet.aptoslabs.com",
 *   simulate: false,
 * });
 * ```
 */
export async function transferAssetFromAptos(
  parameters: AptosTransferParams
): Promise<Result<string, Error>> {
  let transaction: SimpleTransaction
  let accountAuthenticator: AccountAuthenticator

  try {
    const transactionPayload = {
      typeArguments: [],
      function: `${parameters.relayContractAddress}::ibc::send`,
      functionArguments: [
        parameters.sourceChannel,
        isValidBech32Address(parameters.receiver)
          ? bech32ToBytes(parameters.receiver)
          : hexStringToUint8Array(parameters.receiver),
        [parameters.denomAddress],
        [parameters.amount],
        parameters.memo,
        9n,
        BigInt(999_999_999) + 100n,
        0n
      ]
    } satisfies InputGenerateTransactionPayloadData

    if (!parameters.account) return err(new Error("no `account` passed"))

    const { signerPublicKey, signerAddress } =
      parameters.authAccess === "key"
        ? {
            signerPublicKey: parameters.account.publicKey,
            signerAddress: parameters.account.accountAddress
          }
        : {
            signerAddress: parameters.account.address,
            signerPublicKey: new Ed25519PublicKey(parameters.account.publicKey)
          }

    if (parameters.simulate) {
      transaction = await parameters.aptos.transaction.build.simple({
        sender: signerAddress,
        data: transactionPayload
      })

      const simulationResult = await parameters.aptos.transaction.simulate.simple({
        transaction,
        signerPublicKey
      })

      const resultItem = simulationResult.at(0)
      if (!resultItem?.success) return err(new Error(`Simulation failed: ${simulationResult}`))
    }

    // transfer submitted with direct private key
    if (parameters.authAccess === "key") {
      transaction ??= await parameters.aptos.transaction.build.simple({
        sender: signerAddress,
        data: transactionPayload
      })

      accountAuthenticator = parameters.aptos.transaction.sign({
        transaction,
        signer: parameters.account
      })

      // transfer submitted with browser extension wallet or mobile app wallet
    } else if (parameters.authAccess === "wallet") {
      const signTransactionResponse = await parameters.sign({
        payload: {
          function: transactionPayload.function,
          type_arguments: transactionPayload.typeArguments,
          function_arguments: transactionPayload.functionArguments
        }
      })

      accountAuthenticator = signTransactionResponse.accountAuthenticator
      transaction = new SimpleTransaction(signTransactionResponse.rawTxn)

      // opsy
    } else return err(new Error("opsy"))

    const pendingTransaction = await parameters.aptos.transaction.submit.simple({
      transaction,
      senderAuthenticator: accountAuthenticator
    })

    return ok(pendingTransaction.hash)
  } catch (error) {
    return err(new Error(`Transfer failed: ${error}`))
  }
}

export async function aptosSameChainTransfer(
  parameters: AptosTransferBaseParams
): Promise<Result<string, Error>> {
  let transaction: SimpleTransaction
  let accountAuthenticator: AccountAuthenticator

  try {
    const transactionPayload = {
      function: "0x1::primary_fungible_store::transfer",
      typeArguments: ["0x1::fungible_asset::Metadata"],
      functionArguments: [parameters.denomAddress, parameters.receiver, parameters.amount]
    } satisfies InputGenerateTransactionPayloadData

    if (!parameters.account) return err(new Error("no `account` passed"))

    const { signerPublicKey, signerAddress } =
      parameters.authAccess === "key"
        ? {
            signerPublicKey: parameters.account.publicKey,
            signerAddress: parameters.account.accountAddress
          }
        : {
            signerAddress: parameters.account.address,
            signerPublicKey: new Ed25519PublicKey(parameters.account.publicKey)
          }

    if (parameters.simulate) {
      transaction = await parameters.aptos.transaction.build.simple({
        sender: signerAddress,
        data: transactionPayload
      })

      const simulationResult = await parameters.aptos.transaction.simulate.simple({
        transaction,
        signerPublicKey
      })

      const resultItem = simulationResult.at(0)
      if (!resultItem?.success) return err(new Error(`Simulation failed: ${simulationResult}`))
    }

    // transfer submitted with direct private key
    if (parameters.authAccess === "key") {
      transaction ??= await parameters.aptos.transaction.build.simple({
        sender: signerAddress,
        data: transactionPayload
      })

      accountAuthenticator = parameters.aptos.transaction.sign({
        transaction,
        signer: parameters.account
      })

      // transfer submitted with browser extension wallet or mobile app wallet
    } else if (parameters.authAccess === "wallet") {
      const signTransactionResponse = await parameters.sign({
        payload: {
          function: transactionPayload.function,
          type_arguments: transactionPayload.typeArguments,
          function_arguments: transactionPayload.functionArguments
        }
      })

      accountAuthenticator = signTransactionResponse.accountAuthenticator
      transaction = new SimpleTransaction(signTransactionResponse.rawTxn)

      // opsy TODO: actually
    } else return err(new Error("opsy"))

    const pendingTransaction = await parameters.aptos.transaction.submit.simple({
      transaction,
      senderAuthenticator: accountAuthenticator
    })

    return ok(pendingTransaction.hash)
  } catch (error) {
    return err(new Error(`Transfer failed: ${error}`))
  }
}

type AptosTransferSimulateParams =
  | (AptosTransferBaseParams & {
      path: "SAME_CHAIN"
    })
  | (AptosTransferParams & {
      path: "CROSS_CHAIN"
    })

export async function aptosTransferSimulate(
  parameters: AptosTransferSimulateParams
): Promise<Result<string, Error>> {
  try {
    let transactionPayload: InputGenerateTransactionPayloadData

    if (parameters.path === "SAME_CHAIN") {
      transactionPayload = {
        typeArguments: ["0x1::fungible_asset::Metadata"],
        function: "0x1::primary_fungible_store::transfer",
        functionArguments: [parameters.denomAddress, parameters.receiver, parameters.amount]
      }
    } else {
      transactionPayload = {
        function: `${parameters.relayContractAddress}::ibc::send`,
        functionArguments: [
          parameters.sourceChannel,
          isValidBech32Address(parameters.receiver)
            ? bech32ToBytes(parameters.receiver)
            : hexStringToUint8Array(parameters.receiver),
          [parameters.denomAddress],
          [parameters.amount],
          parameters.memo,
          9n,
          BigInt(999_999_999) + 100n,
          0n
        ]
      }
    }

    if (!parameters.account) return err(new Error("no `account` passed"))

    const { signerPublicKey, signerAddress } =
      parameters.authAccess === "wallet"
        ? {
            signerAddress: parameters.account.address,
            signerPublicKey: new Ed25519PublicKey(parameters.account.publicKey)
          }
        : {
            signerPublicKey: parameters.account.publicKey,
            signerAddress: parameters.account.accountAddress
          }

    const transaction = await parameters.aptos.transaction.build.simple({
      sender: signerAddress,
      data: transactionPayload
    })

    const simulationResult = await parameters.aptos.transaction.simulate.simple({
      transaction,
      signerPublicKey: signerPublicKey
    })
    const resultItem = simulationResult.at(0)

    if (resultItem?.success) return ok(resultItem.vm_status || "Simulation succeeded.")

    return err(new Error(resultItem?.vm_status || "Simulation failed."))
  } catch (error) {
    return err(new Error(`Simulation failed ${error instanceof Error ? error.message : error}`))
  }
}
