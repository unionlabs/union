import { err, ok, type Result } from "neverthrow"
import { Hex } from "node_modules/@aptos-labs/ts-sdk/dist/common"
import { type Account, Aptos, AptosConfig, Network } from "@aptos-labs/ts-sdk"

export type TransferAssetFromAptosParams = {
  memo?: string
  amount: bigint
  receiver: string
  account: Account
  denomAddress: string
  sourceChannel: string
  relayContractAddress: string
  baseUrl: string
  simulate?: boolean
}

export type SameChainTransferParams = {
  amount: bigint
  account: Account
  receiver: string
  denomAddress: string
  baseUrl: string
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
 *   receiver: "0xReceiverAddress",
 *   denomAddress: "0x1::aptos_coin::AptosCoin",
 *   sourceChannel: "channel-1",
 *   relayContractAddress: "0x2222222222222222222222222222222222222222",
 *   baseUrl: "https://fullnode.devnet.aptoslabs.com",
 *   simulate: false,
 * });
 * ```
 */
export async function transferAssetFromAptos({
  memo = "transfer",
  amount,
  account,
  receiver,
  denomAddress,
  sourceChannel,
  relayContractAddress,
  baseUrl,
  simulate = false
}: TransferAssetFromAptosParams): Promise<Result<string, Error>> {
  try {
    if (!baseUrl) return err(new Error("Base URL for Aptos node not provided"))

    const config = new AptosConfig({ fullnode: baseUrl, network: Network.TESTNET })
    const aptos = new Aptos(config)

    const transaction = await aptos.transaction.build.simple({
      sender: account.accountAddress,
      data: {
        function: `${relayContractAddress}::ibc::send`,
        functionArguments: [
          sourceChannel,
          Hex.fromHexString(receiver).toUint8Array(),
          [denomAddress],
          [amount],
          memo,
          9n,
          BigInt(999_999_999) + 100n,
          0n
        ]
      }
    })

    if (simulate) {
      const simulationResult = await aptos.transaction.simulate.simple({
        transaction,
        signerPublicKey: account.publicKey
      })

      const resultItem = simulationResult.at(0)

      if (!resultItem?.success) return err(new Error(`Simulation failed: ${simulationResult}`))
    }

    const senderAuthenticator = aptos.transaction.sign({ signer: account, transaction })
    const pendingTransaction = await aptos.transaction.submit.simple({
      transaction,
      senderAuthenticator
    })

    return ok(pendingTransaction.hash)
  } catch (error) {
    return err(new Error(`Transfer failed: ${error}`))
  }
}

export async function aptosSameChainTransfer({
  amount,
  account,
  receiver,
  denomAddress,
  baseUrl
}: SameChainTransferParams): Promise<Result<string, Error>> {
  try {
    if (!baseUrl) return err(new Error("Base URL for Aptos node not provided"))

    const config = new AptosConfig({ fullnode: baseUrl, network: Network.TESTNET })
    const aptos = new Aptos(config)

    const transaction = await aptos.transaction.build.simple({
      sender: account.accountAddress,
      data: {
        function: "0x1::primary_fungible_store::transfer",
        typeArguments: ["0x1::fungible_asset::Metadata"],
        functionArguments: [denomAddress, receiver, amount]
      }
    })

    const senderAuthenticator = aptos.transaction.sign({ signer: account, transaction })

    const pendingTransaction = await aptos.transaction.submit.simple({
      transaction,
      senderAuthenticator
    })

    return ok(pendingTransaction.hash)
  } catch (error) {
    return err(new Error(`Transfer failed ${error instanceof Error ? error.message : error}`))
  }
}

async function getBalance(
  aptos: Aptos,
  accountAddress: string,
  denomAddress: string
): Promise<Result<number, Error>> {
  try {
    const [balanceString] = await aptos.view<[string]>({
      payload: {
        function: "0x1::primary_fungible_store::balance",
        typeArguments: ["0x1::object::ObjectCore"],
        functionArguments: [accountAddress, denomAddress]
      }
    })

    const balance = Number.parseInt(balanceString, 10)

    return ok(balance)
  } catch (error) {
    return err(new Error(`Failed to fetch balance for account ${accountAddress}`))
  }
}

export async function transferAssetFromAptosSimulate({
  memo = "transfer",
  amount,
  account,
  receiver,
  denomAddress,
  sourceChannel,
  relayContractAddress,
  baseUrl
}: TransferAssetFromAptosParams): Promise<Result<string, Error>> {
  try {
    if (!baseUrl) return err(new Error("Base URL for Aptos node not provided"))

    const config = new AptosConfig({ fullnode: baseUrl, network: Network.TESTNET })
    const aptos = new Aptos(config)

    const transaction = await aptos.transaction.build.simple({
      sender: account.accountAddress,
      data: {
        function: `${relayContractAddress}::ibc::send`,
        functionArguments: [
          sourceChannel,
          Hex.fromHexString(receiver).toUint8Array(),
          [denomAddress],
          [amount],
          memo,
          9n,
          BigInt(999_999_999) + 100n,
          0n
        ]
      }
    })

    const simulationResult = await aptos.transaction.simulate.simple({
      transaction,
      signerPublicKey: account.publicKey
    })
    const resultItem = simulationResult.at(0)

    if (resultItem?.success) return ok(resultItem.vm_status || "Simulation succeeded.")

    return err(new Error(resultItem?.vm_status || "Simulation failed."))
  } catch (error) {
    return err(new Error(`Simulation failed ${error instanceof Error ? error.message : error}`))
  }
}
