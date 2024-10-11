import { err, ok, type Result } from "neverthrow"
import {
  type Account,
  Aptos,
  AptosConfig,
  Network,
  parseTypeTag,
  APTOS_COIN
} from "@aptos-labs/ts-sdk"
import consola from "consola"
import { Hex } from "node_modules/@aptos-labs/ts-sdk/dist/common"

const APTOS_COIN_TYPE = parseTypeTag(APTOS_COIN)

export type TransferAssetFromMoveParams = {
  memo?: string
  amount: bigint
  receiver: string // Receiver's address
  account: Account // Sender's account address
  denomAddress: string // The Move coin type resource address
  sourceChannel: string // Source IBC channel
  relayContractAddress: string // Contract address to call send function
  // timeoutHeight: { revision_number: bigint; revision_height: bigint } // Timeout height
  // timeoutTimestamp: bigint // Timeout timestamp
  baseUrl: string // Base URL of the Aptos full node
  simulate?: boolean // Flag for simulation
}

export type SameChainTransferParams = {
  amount: bigint
  account: Account // Sender's account
  receiver: string // Receiver's address
  denomAddress: string // The Move coin type resource address
  baseUrl: string // Base URL of the Aptos full node
}

/**
 * Transfer an asset from the Move blockchain (e.g., Aptos) using the IBC `send` function, similar to EVM implementation.
 *
 * @example
 * ```ts
 * const transfer = await transferAssetFromMove({
 *   memo: "test",
 *   amount: BigInt(1),
 *   account: "0xSenderAccountAddress",
 *   receiver: "0xReceiverAddress",
 *   denomAddress: "0x1::aptos_coin::AptosCoin",
 *   sourceChannel: "channel-1",
 *   relayContractAddress: "0x2222222222222222222222222222222222222222",
 *   timeoutHeight: { revision_number: 9n, revision_height: BigInt(999_999_999) },
 *   timeoutTimestamp: BigInt(0),
 *   baseUrl: "https://fullnode.devnet.aptoslabs.com",
 *   simulate: false,
 * });
 * ```
 */
export async function transferAssetFromMove({
  memo = "transfer",
  amount,
  account,
  simulate = true,
  receiver,
  denomAddress,
  sourceChannel,
  relayContractAddress,
  baseUrl
}: TransferAssetFromMoveParams): Promise<Result<string, Error>> {
  try {
    // Ensure the baseUrl is provided and valid
    if (!baseUrl) {
      return err(new Error("Base URL for Aptos node not provided"))
    }

    // Setup the Aptos client with the correct network and base URL
    const config = new AptosConfig({ fullnode: baseUrl, network: Network.TESTNET })
    const aptos = new Aptos(config)

    consola.info(`Using Aptos fullnode at: ${baseUrl}`)

    // Build the transaction using the IBC `send` function (similar to EVM)
    const transaction = await aptos.transaction.build.simple({
      sender: account.accountAddress,
      data: {
        // Call the `send` function in the relay contract
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

    consola.info("Transaction built successfully")

    if (simulate) {
      const simulationResult = await aptos.transaction.simulate.simple({
        transaction,
        signerPublicKey: account.publicKey
      })
      const success = simulationResult[0]?.success
      const vm_status = simulationResult[0]?.vm_status
      if (!success) {
        consola.error(`Simulation failed: ${vm_status}`)
        return err(new Error(`Simulation failed: ${simulationResult}`))
      }
      consola.info(`Simulation succeeded: ${vm_status}`)
    }

    // Sign and submit the transaction
    const senderAuthenticator = aptos.transaction.sign({
      signer: account,
      transaction
    })

    const pendingTxn = await aptos.transaction.submit.simple({ transaction, senderAuthenticator })
    consola.info(`Transaction executed! Hash: ${pendingTxn.hash}`)

    return ok(pendingTxn.hash) // Return the transaction hash
  } catch (error) {
    return err(new Error(`Transfer failed: ${error}`))
  }
}

export async function transferAssetFromMoveSimulate({
  memo = "transfer",
  amount,
  account,
  receiver,
  denomAddress,
  sourceChannel,
  relayContractAddress,
  baseUrl
}: TransferAssetFromMoveParams): Promise<Result<string, Error>> {
  try {
    // Setup the Aptos client with the correct network and base URL
    const config = new AptosConfig({ fullnode: baseUrl, network: Network.TESTNET })
    const aptos = new Aptos(config)

    consola.info(`Using Aptos fullnode at: ${baseUrl}`)

    // Build the transaction using the IBC `send` function (similar to EVM)
    const transaction = await aptos.transaction.build.simple({
      sender: account.accountAddress,
      data: {
        // Call the `send` function in the relay contract
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

    consola.info("Transaction built successfully")

    // Simulate the transaction to estimate gas or check validity
    const simulationResult = await aptos.transaction.simulate.simple({
      transaction,
      signerPublicKey: account.publicKey
    })

    // Check if the simulation was successful
    const success = simulationResult[0]?.success
    const vm_status = simulationResult[0]?.vm_status
    consola.info(`Simulation result: ${vm_status}`)

    // If successful, return the VM status as a success message
    if (success) {
      return ok(vm_status || "Simulation succeeded.")
    } else {
      // If simulation failed, return an error with the VM status as the error message
      return err(new Error(vm_status || "Simulation failed."))
    }
  } catch (error) {
    // In case of an exception, return the error
    consola.error(`Simulation failed: ${error}`)
    return err(new Error(`Simulation failed: ${error.message}`))
  }
}

export async function moveSameChainTransfer({
  amount,
  account,
  receiver,
  denomAddress,
  baseUrl
}: SameChainTransferParams): Promise<Result<string, Error>> {
  try {
    // Setup the Aptos client with the correct network and base URL
    const config = new AptosConfig({ fullnode: baseUrl, network: Network.TESTNET })
    const aptos = new Aptos(config)

    consola.info(`Using Aptos fullnode at: ${baseUrl}`)

    // Fetch and log the balance of the sender and receiver before the transfer
    //const senderBalanceBefore = await aptos.account.getAccountOwnedObjects(account.accountAddress);
    // //   const receiverBalanceBefore = await aptos.account.getAccountOwnedObjects(receiver);

    //   consola.info(`Sender balance before transfer: ${senderBalanceBefore}`);
    //   consola.info(`Receiver balance before transfer: ${receiverBalanceBefore}`);

    // Build the transaction for a direct transfer
    const sender_account_addr = account.accountAddress.toString()
    const balance = await getBalance(aptos, sender_account_addr, denomAddress)
    consola.info(`Balance of account ${sender_account_addr} is => ${balance.value}`)
    const balance_receiver = await getBalance(aptos, receiver, denomAddress)
    consola.info(`Balance of account ${receiver} is => ${balance_receiver.value}`)

    const transaction = await aptos.transaction.build.simple({
      sender: account.accountAddress,
      data: {
        function: "0x1::primary_fungible_store::transfer",
        typeArguments: ["0x1::fungible_asset::Metadata"],
        functionArguments: [denomAddress, receiver, amount]
      }
    })
    consola.info("Transaction built successfully")

    // Sign and submit the transaction
    const senderAuthenticator = aptos.transaction.sign({
      signer: account,
      transaction
    })

    const pendingTxn = await aptos.transaction.submit.simple({
      transaction,
      senderAuthenticator
    })

    consola.info(`Transaction executed! Hash: ${pendingTxn.hash}`)

    // Fetch and log the balance of the sender and receiver after the transfer
    //   const senderBalanceAfter = await aptos.account.getAccountOwnedObjects(account.accountAddress);

    //   consola.info(`Sender balance after transfer: ${senderBalanceAfter}`);

    return ok(pendingTxn.hash) // Return the transaction hash
  } catch (error) {
    consola.error(`Transfer failed: ${error}`)
    return err(new Error(`Transfer failed: ${error.message}`))
  }
}

// Helper function to get balance
async function getBalance(
  aptos: Aptos,
  accountAddress: string,
  denomAddress: string
): Promise<Result<number, Error>> {
  try {
    const [balanceStr] = await aptos.view<[string]>({
      payload: {
        function: "0x1::primary_fungible_store::balance",
        typeArguments: ["0x1::object::ObjectCore"],
        functionArguments: [accountAddress, denomAddress]
      }
    })

    const balance = Number.parseInt(balanceStr, 10)

    return ok(balance)
  } catch (error) {
    return err(new Error(`Failed to fetch balance for account ${accountAddress}`))
  }
}
