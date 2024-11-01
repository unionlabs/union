import {
  type Aptos,
  Ed25519PublicKey,
  type SimpleTransaction,
  type AccountAddressInput,
  type Account as AptosAccount,
  type InputGenerateTransactionPayloadData
} from "@aptos-labs/ts-sdk"
import { err, ok, type Result, ResultAsync } from "neverthrow"
import { isValidBech32Address } from "../utilities/address.ts"
import type { AptosBrowserWallet, AuthAccess } from "#aptos/wallet.ts"
import { bech32AddressToHex, bech32ToBytes, hexStringToUint8Array } from "../convert.ts"

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

export const waitForTransactionReceipt: (args_0: { aptos: Aptos; hash: string }) => ResultAsync<
  string,
  Error
> = ResultAsync.fromThrowable(
  async ({ aptos, hash }: { aptos: Aptos; hash: string }) => {
    const transactionResult = await aptos.waitForTransaction({
      transactionHash: hash,
      options: { checkSuccess: false }
    })
    if (!transactionResult.success) {
      throw new Error(transactionResult.vm_status || "waiting for transaction failed")
    }
    return transactionResult.hash
  },
  error => new Error(`Waiting for transaction failed: ${error}`, { cause: error })
)

const buildSimpleTransaction: (args: {
  aptos: Aptos
  accountAddress: AccountAddressInput
  data: InputGenerateTransactionPayloadData
}) => ResultAsync<SimpleTransaction, Error> = ResultAsync.fromThrowable(
  async (args: {
    aptos: Aptos
    accountAddress: AccountAddressInput
    data: InputGenerateTransactionPayloadData
  }) =>
    args.aptos.transaction.build.simple({
      data: args.data,
      sender: args.accountAddress
    }),
  error => new Error(`Build simple transaction failed: ${error}`, { cause: error })
)

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
  try {
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
      const hash = await signer.signAndSubmitTransaction({
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
      return ok(hash)
    }

    const signer = parameters.signer as AptosAccount

    const transaction = await buildSimpleTransaction({
      data: payload,
      aptos: parameters.aptos,
      accountAddress: signer.accountAddress
    })

    if (!transaction.isOk()) return err(transaction.error)

    if (parameters.simulate) {
      const simulationResult = await parameters.aptos.transaction.simulate.simple({
        transaction: transaction.value,
        signerPublicKey: signer.publicKey
      })

      const resultItem = simulationResult.at(0)
      if (!resultItem?.success) return err(new Error(`Simulation failed: ${simulationResult}`))
      console.info(`aptosTransferSimulate simulation succeeded: ${simulationResult.at(0)?.hash}`)
    }

    const pendingTransaction = await parameters.aptos.transaction.submit.simple({
      transaction: transaction.value,
      senderAuthenticator: parameters.aptos.transaction.sign({
        signer,
        transaction: transaction.value
      })
    })

    return ok(pendingTransaction.hash)
  } catch (error) {
    return err(new Error(`Transfer failed: ${error}`))
  }
}

export async function aptosSameChainTransfer(
  parameters: AptosTransferBaseParams
): Promise<Result<string, Error>> {
  try {
    if (!parameters.signer) return err(new Error("no `signer` passed"))

    if (parameters.authAccess === "wallet") {
      const signer = parameters.signer as AptosBrowserWallet
      const hash = await signer.signAndSubmitTransaction({
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

      return ok(hash)
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

    if (!transaction.isOk()) return err(transaction.error)

    if (parameters.simulate) {
      const simulationResult = await parameters.aptos.transaction.simulate.simple({
        transaction: transaction.value,
        signerPublicKey: signer.publicKey
      })

      const resultItem = simulationResult.at(0)
      if (!resultItem?.success) return err(new Error(`Simulation failed: ${simulationResult}`))
      console.info(`aptosSameChainTransfer simulation succeeded: ${simulationResult.at(0)?.hash}`)
    }

    const senderAuthenticator = parameters.aptos.transaction.sign({
      signer,
      transaction: transaction.value
    })

    const simpleTransactionResult = await parameters.aptos.transaction.submit.simple({
      senderAuthenticator,
      transaction: transaction.value
    })

    if (!simpleTransactionResult.hash.startsWith("0x")) return err(new Error("hash not found"))
    return ok(simpleTransactionResult.hash)
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

    const transaction = await parameters.aptos.transaction.build.simple({
      sender: signerAddress,
      data: transactionPayload
    })

    const simulationResult = await parameters.aptos.transaction.simulate.simple({
      transaction,
      signerPublicKey
    })

    const resultItem = simulationResult.at(0)
    if (resultItem?.success) return ok(resultItem.hash)

    return err(new Error(resultItem?.vm_status || "Simulation failed."))
  } catch (error) {
    return err(new Error(`Simulation failed ${error instanceof Error ? error.message : error}`))
  }
}
