import { err, ok, Result } from "neverthrow";
import { Account, Aptos, AptosConfig, Network } from "@aptos-labs/ts-sdk";
import consola from "consola";
import { raise } from "#utilities/index.ts";

export type TransferAssetFromMoveParams = {
  memo?: string;
  amount: bigint;
  receiver: string; // Receiver's address
  account: Account; // Sender's account address
  denomAddress: string; // The Move coin type resource address
  sourceChannel: string; // Source IBC channel
  relayContractAddress: string; // Contract address to call send function
  timeoutHeight: { revision_number: bigint; revision_height: bigint }; // Timeout height
  timeoutTimestamp: bigint; // Timeout timestamp
  baseUrl: string; // Base URL of the Aptos full node
  simulate?: boolean; // Flag for simulation
};

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
  receiver,
  denomAddress,
  sourceChannel,
  relayContractAddress,
  timeoutHeight,
  timeoutTimestamp,
  baseUrl,
  simulate = false,
}: TransferAssetFromMoveParams): Promise<Result<string, Error>> {
  try {
    // Ensure the baseUrl is provided and valid
    if (!baseUrl) {
      return err(new Error("Base URL for Aptos node not provided"));
    }

    // TODO: Handle simulation scenario
    if (simulate) {
      raise("Simulation not implemented");
    }

    // Setup the Aptos client with the correct network and base URL
    const config = new AptosConfig({ fullnode: baseUrl, network: Network.TESTNET });
    const aptos = new Aptos(config);

    consola.info(`Using Aptos fullnode at: ${baseUrl}`);

    // Build the transaction using the IBC `send` function (similar to EVM)
    const transaction = await aptos.transaction.build.simple({
      sender: account.accountAddress,
      data: {
        // Call the `send` function in the relay contract
        function: `${relayContractAddress}::ibc::send`,
        functionArguments: [
          sourceChannel,
          receiver.startsWith("0x") ? receiver : receiver,
          [{ denom: denomAddress, amount }], 
          memo, 
          timeoutHeight, 
          timeoutTimestamp, 
        ],
      },
    });

    consola.info("Transaction built successfully");

    // Sign and submit the transaction
    const senderAuth = await aptos.transaction.sign({
      signer: account,
      transaction,
    });

    const pendingTxn = await aptos.transaction.submit.simple({ transaction, senderAuth });

    consola.info(`Transaction executed! Hash: ${pendingTxn.hash}`);

    return ok(pendingTxn.hash); // Return the transaction hash
  } catch (error) {
    return err(new Error(`Transfer failed: ${error.message}`));
  }
}

// Helper function to convert Bech32 receiver address to hex (if needed)
function convertToHex(receiver: string): string {
  // You can implement this based on your project's needs for Bech32 conversion.
  return receiver;
}

// import aptosClient from "@aptos-labs/aptos-client";
// import { err, ok, Result } from "neverthrow";
// import { Account, Aptos, AptosConfig, Network } from "@aptos-labs/ts-sdk";
// import consola from "consola";

// const APTOS_COIN = "0x1::aptos_coin::AptosCoin";
// const COIN_STORE = `0x1::coin::CoinStore<${APTOS_COIN}>`;

// export type TransferAssetFromMoveParams = {
//   memo?: string;
//   amount: bigint;
//   receiver: string; // Receiver's address
//   account: string; // Sender's account address
//   denomAddress: string; // This will be the Move coin type resource address
//   baseUrl: string; // Base URL of the Aptos full node
// };

// /**
//  * Transfer an asset from the Move blockchain (e.g., Aptos).
//  *
//  * @example
//  * ```ts
//  * const transfer = await transferAssetFromMove({
//  *   memo: "test",
//  *   amount: BigInt(1),
//  *   account: "0xSenderAccountAddress",
//  *   receiver: "0xReceiverAddress",
//  *   denomAddress: "0x1::aptos_coin::AptosCoin",
//  *   baseUrl: "https://fullnode.devnet.aptoslabs.com",
//  * });
//  * ```
//  */
// export async function transferAssetFromMove({
//   memo = "transfer",
//   amount,
//   account,
//   receiver,
//   denomAddress,
//   baseUrl,
// }: TransferAssetFromMoveParams): Promise<Result<string, Error>> {
//   try {
//     // Ensure the baseUrl is provided and valid
//     if (!baseUrl) {
//       return err(new Error("Base URL for Aptos node not provided"));
//     }


//       // Setup the client
//     const config = new AptosConfig({ network: Network.TESTNET });
//     const aptos = new Aptos(config);

//     // Each account has a private key, a public key, and an address
//     const alice = Account.generate();
//     const bob = Account.generate();
    
//     console.log("=== Addresses ===\n");
//     console.log(`Alice's address is: ${alice.accountAddress}`);
//     console.log(`Bob's address is: ${bob.accountAddress}`);
//     const bobAccountBalance = await aptos.getAccountResource({
//         accountAddress: bob.accountAddress,
//         resourceType: COIN_STORE,
//       });
//       const bobBalance = Number(bobAccountBalance.coin.value);
//       console.log(`Bob's balance is: ${bobBalance}`);
      
//     const bobAccountBalance2 = await aptos.getAccountResource({
//         accountAddress: "0xe3579557fd55ed8fab0d1e211eb1c05d56d74650e7070b703925493c38fe2aed",
//         resourceType: COIN_STORE,
//     });
    
//     consola.info(`bobAccountBalance2: ${bobAccountBalance2}`);
//     const modules = await aptos.getAccountModules({ accountAddress: "0x123" });
//     consola.info(`modules: ${modules}`);
//     const tokens = await aptos.getAccountOwnedTokens({ accountAddress: "0xe3579557fd55ed8fab0d1e211eb1c05d56d74650e7070b703925493c38fe2aed" });
//     consola.info(`tokens: ${tokens}`);

//     // Building the transaction payload
//     const body = {
//       sender: account,
//       payload: {
//         function: `${denomAddress}::transfer`, // Coin module's transfer function
//         type_arguments: [],
//         arguments: [
//           receiver, // Receiver's address
//           amount.toString(), // Amount in string format
//         ],
//       },
//       memo,
//     };

//     // Full URL combining the base URL and the path
//     const requestUrl = `${baseUrl}/v1/transactions`;

//     // Make the request to the Aptos API
//     const response = await aptosClient({
//       url: requestUrl,
//       method: "POST",
//       body,
//     });

//     if (response.status === 200) {
//       const transactionHash = response.data.hash;
//       return ok(transactionHash); // Return transaction hash
//     } else {
//       return err(new Error(`Transaction failed: ${response.statusText}`));
//     }
//   } catch (error) {
//     return err(new Error(`Transfer failed: ${error.message}`));
//   }
// }
