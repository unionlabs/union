import {
  type Aptos,
  type PublicKey,
  Ed25519PublicKey,
  type AnyRawTransaction,
  type SimpleTransaction,
  type AccountAddressInput,
  type AccountAuthenticator,
  type Account as AptosAccount,
  type UserTransactionResponse,
  type PendingTransactionResponse,
  type InputGenerateTransactionPayloadData
} from "@aptos-labs/ts-sdk"
import { err, ok, type Result, ResultAsync } from "neverthrow"
import { isValidBech32Address } from "../utilities/address.ts"
import type { AptosBrowserWallet, AuthAccess } from "#aptos/wallet.ts"
import { bech32AddressToHex, bech32ToBytes, hexToBytes } from "../convert.ts"

export type { AptosAccount, AptosTransferParams, AptosPublicAccountInfo }

type AptosPublicAccountInfo = { address: string; publicKey: string }

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

type AptosTransferParams = AptosTransferBaseParams & {
  sourceChannel: string
  relayContractAddress: string
}

export const waitForTransactionReceipt: (args: { aptos: Aptos; hash: string }) => ResultAsync<
  string,
  Error
> = ResultAsync.fromThrowable(
  async args => {
    const transactionResult = await args.aptos.waitForTransaction({
      transactionHash: args.hash,
      options: { checkSuccess: false }
    })
    if (!transactionResult?.success) {
      throw new Error(transactionResult.vm_status || "waiting for transaction failed")
    }
    return transactionResult.hash
  },
  error => new Error(`Waiting for transaction failed: ${error}`, { cause: error })
)

export const buildSimpleTransaction: (args: {
  aptos: Aptos
  accountAddress: AccountAddressInput
  data: InputGenerateTransactionPayloadData
}) => ResultAsync<SimpleTransaction, Error> = ResultAsync.fromThrowable(
  async args =>
    args.aptos.transaction.build.simple({
      data: args.data,
      sender: args.accountAddress
    }),
  error => new Error(`Build simple transaction failed`, { cause: error })
)

export const submitSimpleTransaction: (args: {
  aptos: Aptos
  transaction: AnyRawTransaction
  accountAuthenticator: AccountAuthenticator
}) => ResultAsync<PendingTransactionResponse, Error> = ResultAsync.fromThrowable(
  async args =>
    args.aptos.transaction.submit.simple({
      transaction: args.transaction,
      senderAuthenticator: args.accountAuthenticator
    }),
  error => new Error(`Submit simple transaction failed`, { cause: error })
)

export const simulateSimpleTransaction: (args: {
  aptos: Aptos
  signerPublicKey: PublicKey
  transaction: AnyRawTransaction
}) => ResultAsync<UserTransactionResponse, Error> = ResultAsync.fromThrowable(
  async args => {
    const [simulationResult] = await args.aptos.transaction.simulate.simple({
      transaction: args.transaction,
      signerPublicKey: args.signerPublicKey
    })
    if (!simulationResult?.success) throw new Error("simulation result not found")
    return simulationResult
  },
  error => new Error(`Simulate simple transaction failed`, { cause: error })
)

export const transferAssetFromAptos: (args: AptosTransferParams) => ResultAsync<string, Error> =
  ResultAsync.fromThrowable(
    async parameters => {
      const payload = {
        function: `${parameters.relayContractAddress}::ibc::send`,
        typeArguments: [],
        functionArguments: [
          parameters.sourceChannel,
          isValidBech32Address(parameters.receiver)
            ? bech32AddressToHex({ address: parameters.receiver })
            : parameters.receiver,
          [parameters.denomAddress],
          [parameters.amount],
          parameters.memo ?? "",
          9n,
          999_999_999n,
          0n
        ]
      } as const satisfies InputGenerateTransactionPayloadData

      if (parameters.authAccess === "wallet") {
        const signer = parameters.signer as AptosBrowserWallet
        const transaction = await signer.signAndSubmitTransaction({
          payload: {
            function: payload.function,
            type_arguments: payload.typeArguments,
            arguments: [
              parameters.sourceChannel,
              isValidBech32Address(parameters.receiver)
                ? bech32AddressToHex({ address: parameters.receiver })
                : parameters.receiver,
              [parameters.denomAddress],
              [parameters.amount.toString()],
              parameters.memo ?? "",
              BigInt(9n).toString(),
              BigInt(999_999_999).toString(),
              BigInt(0n).toString()
            ]
          }
        })

        if (!transaction?.success) {
          throw new Error(
            `Transaction failed: ${transaction?.vm_status} - ${JSON.stringify(transaction, undefined, 2)}`
          )
        }
        return transaction.hash
      }

      const signer = parameters.signer as AptosAccount

      const transaction = await buildSimpleTransaction({
        data: payload,
        aptos: parameters.aptos,
        accountAddress: signer.accountAddress
      })

      if (!transaction.isOk()) throw transaction.error

      if (parameters.simulate) {
        const simulationResult = await simulateSimpleTransaction({
          aptos: parameters.aptos,
          transaction: transaction.value,
          signerPublicKey: signer.publicKey
        })
        if (!simulationResult.isOk()) throw simulationResult.error

        console.info(`aptosTransferSimulate simulation succeeded: ${simulationResult.value.hash}`)
      }

      const pendingTransaction = await submitSimpleTransaction({
        aptos: parameters.aptos,
        transaction: transaction.value,
        accountAuthenticator: parameters.aptos.transaction.sign({
          signer,
          transaction: transaction.value
        })
      })

      if (!pendingTransaction.isOk()) throw pendingTransaction.error
      if (!pendingTransaction.value.hash.startsWith("0x")) throw new Error("hash not found")

      return pendingTransaction.value.hash
    },
    error => new Error(`Transfer failed: ${error}`, { cause: error })
  )

export const aptosSameChainTransfer: (args: AptosTransferBaseParams) => ResultAsync<string, Error> =
  ResultAsync.fromThrowable(
    async parameters => {
      if (!parameters.signer) throw new Error("no `signer` passed")

      if (parameters.authAccess === "wallet") {
        const signer = parameters.signer as AptosBrowserWallet
        const transaction = await signer.signAndSubmitTransaction({
          payload: {
            function: "0x1::primary_fungible_store::transfer",
            type_arguments: ["0x1::fungible_asset::Metadata"],
            arguments: [
              //
              parameters.denomAddress,
              parameters.receiver,
              parameters.amount.toString()
            ]
          }
        })

        if (!transaction?.success) {
          throw new Error(
            `Transaction failed: ${transaction?.vm_status} - ${JSON.stringify(transaction, undefined, 2)}`
          )
        }
        return transaction.hash
      }

      const signer = parameters.signer as AptosAccount

      const transaction = await buildSimpleTransaction({
        data: {
          typeArguments: ["0x1::fungible_asset::Metadata"],
          function: "0x1::primary_fungible_store::transfer",
          functionArguments: [parameters.denomAddress, parameters.receiver, parameters.amount]
        },
        aptos: parameters.aptos,
        accountAddress: signer.accountAddress
      })

      if (!transaction.isOk()) throw transaction.error

      if (parameters.simulate) {
        const simulationResult = await simulateSimpleTransaction({
          aptos: parameters.aptos,
          transaction: transaction.value,
          signerPublicKey: signer.publicKey
        })
        if (!simulationResult.isOk()) throw simulationResult.error
        console.info(`aptosSameChainTransfer simulation succeeded: ${simulationResult.value.hash}`)
      }

      const senderAuthenticator = parameters.aptos.transaction.sign({
        signer,
        transaction: transaction.value
      })

      const pendingTransaction = await submitSimpleTransaction({
        aptos: parameters.aptos,
        transaction: transaction.value,
        accountAuthenticator: senderAuthenticator
      })

      if (!pendingTransaction.isOk()) throw pendingTransaction.error

      if (!pendingTransaction.value.hash.startsWith("0x")) throw new Error("hash not found")
      return pendingTransaction.value.hash
    },
    error => new Error(`Aptos to Aptos transfer failed: ${error}`, { cause: error })
  )

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
            : hexToBytes(parameters.receiver),
          [parameters.denomAddress],
          [parameters.amount],
          parameters.memo,
          9n,
          BigInt(999_999_999) + 100n,
          0n
        ]
      }
    }

    const { signerPublicKey, signerAddress } =
      parameters.authAccess === "wallet"
        ? {
            // @ts-expect-error TODO: fix
            signerAddress: parameters.signer.address,
            // @ts-expect-error TODO: fix
            signerPublicKey: new Ed25519PublicKey(parameters.signer.publicKey)
          }
        : {
            // @ts-expect-error TODO: fix
            signerPublicKey: parameters.signer.publicKey,
            // @ts-expect-error TODO: fix
            signerAddress: parameters.signer.accountAddress
          }

    const transaction = await buildSimpleTransaction({
      aptos: parameters.aptos,
      accountAddress: signerAddress,
      data: transactionPayload
    })
    if (!transaction.isOk()) return err(transaction.error)

    const simulationResult = await simulateSimpleTransaction({
      aptos: parameters.aptos,
      signerPublicKey,
      transaction: transaction.value
    })
    if (simulationResult.isOk()) return ok(simulationResult.value.hash)

    return err(new Error(simulationResult.error.message || "Simulation failed."))
  } catch (error) {
    return err(new Error(`Simulation failed ${error instanceof Error ? error.message : error}`))
  }
}
