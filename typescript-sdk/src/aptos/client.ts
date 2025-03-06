import {
  type AptosAccount,
  waitForTransactionReceipt,
  type AptosPublicAccountInfo
} from "./transfer.ts"
import { ok, err, type Result } from "neverthrow"
import { Aptos, Network, AptosConfig, AccountAddress, MoveVector } from "@aptos-labs/ts-sdk"
import { createClient, fallback, type HttpTransport } from "viem"
import type { AptosBrowserWallet, AuthAccess } from "./wallet.ts"

// Define a unified signer type that always includes an accountAddress.
export type AptosSigner = AptosAccount | (AptosBrowserWallet & { accountAddress: string })

export type { AptosAccount, AptosBrowserWallet }

export const aptosChainId = [
  "2",   // aptos testnet
  "177", // movement porto
  "250"  // movement bardock
] as const
export type AptosChainId = `${(typeof aptosChainId)[number]}`

/**
 * This client supports two kinds of transports:
 * - A key-based transport (using an AptosAccount)
 * - A wallet-based transport (using an AptosBrowserWallet)
 */
type AptosWindowTransport = AptosBrowserWallet

export type AptosClientParameters = {
  chainId: AptosChainId
} & (
  | { account: AptosAccount; transport: HttpTransport }
  | { account?: AptosPublicAccountInfo; transport: AptosWindowTransport }
)
export type WalletSigner = AptosBrowserWallet & { accountAddress: string };

/**
 * Overloads for retrieving an Aptos client.
 */
async function getAptosClient(
  parameters: AptosClientParameters & { authAccess: "key" }
): Promise<{ authAccess: "key"; aptos: Aptos; signer: AptosSigner; transport: HttpTransport }>;

async function getAptosClient(
  parameters: AptosClientParameters & { authAccess: "wallet" }
): Promise<{ authAccess: "wallet"; aptos: Aptos; signer: AptosSigner; transport: AptosWindowTransport }>;


async function getAptosClient(
  parameters: AptosClientParameters & { authAccess: AuthAccess }
): Promise<
  | { authAccess: "key"; aptos: Aptos; signer: AptosSigner; transport: HttpTransport }
  | { authAccess: "wallet"; aptos: Aptos; signer: AptosSigner; transport: AptosWindowTransport }
> {
  if (parameters.authAccess === "key") {
    if (typeof parameters.transport !== "function") {
      throw new Error("Invalid Aptos transport")
    }
    const rpcUrl = parameters.transport({}).value?.url
    if (!rpcUrl) throw new Error("No Aptos RPC URL found")
    const config = new AptosConfig({
      fullnode: rpcUrl,
      network: Network.CUSTOM
    })
    return {
      authAccess: "key",
      aptos: new Aptos(config),
      signer: parameters.account as AptosAccount, // AptosAccount is assumed to have accountAddress
      transport: parameters.transport
    }
  }

  if (parameters.authAccess === "wallet") {
    if (typeof parameters.transport !== "object") {
      throw new Error("Invalid Aptos transport")
    }
    const networkInfo = await parameters.transport.getNetwork()
    const network =
      networkInfo.name.toLowerCase() === "mainnet" ? Network.MAINNET : Network.TESTNET
    const config = new AptosConfig({ fullnode: networkInfo.url, network })
  
    // Get the connected account
    const account = await parameters.transport.getAccount?.() ||
      { address: "" }
    if (!account.address) {
      throw new Error("No account address found from the wallet")
    }
  
    // Create a signer by merging the wallet’s methods with the account address.
    const signer = Object.assign({}, parameters.transport, {
      accountAddress: account.address
    }) as unknown as AptosAccount  // <== Force-cast to AptosAccount
  
    return {
      authAccess: "wallet",
      aptos: new Aptos(config),
      signer: signer,
      transport: parameters.transport
    }
  }
  

  throw new Error("Invalid Aptos transport")
}

/**
 * New unified transfer parameters for Aptos, matching the Cosmos & EVM clients.
 */
export interface TransferAssetParameters<AptosChainId> {
  baseAmount: bigint
  baseToken: string
  quoteAmount: bigint
  quoteToken: string
  receiver: string
  sourceChannelId: number
  ucs03address: string
}

/**
 * The Aptos client now exposes both a `transferAsset` and a `simulateTransaction`
 * function that accept the same parameters as in your other chains.
 */
export const createAptosClient = (clientParameters: AptosClientParameters) => {
  return createClient({ transport: fallback([]) })
    .extend(() => ({
      getAptosClient: async () => {
        // Use the transport type to determine which client to create.
        if (typeof clientParameters.transport === "function") {
          console.info("returning key-based client")
          return await getAptosClient({ ...clientParameters, authAccess: "key" })
        } else {
          console.info("returning wallet-based client")
          return await getAptosClient({ ...clientParameters, authAccess: "wallet" })
        }
      }
    }))
    .extend(client => ({
      waitForTransactionReceipt: async ({ hash }: { hash: string }) => {
        const { aptos } = await client.getAptosClient()
        return await waitForTransactionReceipt({ aptos, hash })
      },

      /**
       * Executes a transfer on Aptos by calling the UCS03 contract’s `transfer` function.
       * The parameters mirror those used on Cosmos and EVM.
       */
      transferAsset: async ({
        baseAmount,
        baseToken,
        quoteAmount,
        quoteToken,
        receiver,
        sourceChannelId,
        ucs03address
      }: TransferAssetParameters<AptosChainId>): Promise<Result<string, Error>> => {
        const { aptos, signer, authAccess, transport } = await client.getAptosClient();

        const quoteTokenVec = MoveVector.U8(quoteToken)
        const receiverVec = MoveVector.U8(receiver)

        const rawSalt = new Uint8Array(14) 
        crypto.getRandomValues(rawSalt)
        const salt = MoveVector.U8(rawSalt)

        try {
          if (authAccess === "key") {
            console.info("key-based flow")
            // Key-based flow using the full AptosAccount
            const payload = await aptos.transaction.build.simple({
              sender: signer.accountAddress,
              data: {
                function: `${ucs03address}::ibc_app::transfer`,
                typeArguments: [],
                functionArguments: [
                  sourceChannelId,
                  receiverVec,
                  AccountAddress.fromString(baseToken),
                  baseAmount,
                  quoteTokenVec,
                  quoteAmount,
                  18446744073709551615n,
                  18446744073709551615n,
                  salt
                ]
              }
            })

            const txn = await aptos.signAndSubmitTransaction({
              signer: signer as AptosAccount,
              transaction: payload
            });
            const receipt = await waitForTransactionReceipt({ aptos, hash: txn.hash });
            return receipt;
          }   
          const saltHex = toHex(new Uint8Array(14) ); 
          // 14 bytes + 0x 2 bytes and that walletPayload encodes it in it
          // so it becomes 32 byte.
          const walletPayload = {
            function: `${ucs03address}::ibc_app::transfer`,
            type_arguments: [],
            arguments: [
              sourceChannelId.toString(),
              hexToAscii(receiver), // It is hexing again in it.
              baseToken,
              baseAmount.toString(),
              hexToAscii(quoteToken), // It is hexing again in it.
              quoteAmount.toString(),
              18446744073709551615n.toString(),
              18446744073709551615n.toString(),
              saltHex
            ]
          };
          try {
            const signedTxn = await transport.signAndSubmitTransaction({ payload: walletPayload });
            return ok(signedTxn.hash); // Wrap the string in a successful Result
          } catch (error) {
            return err(new Error("Transaction signing failed"));
          }
          
        } catch (error) {
          console.info("error is:", error)
          return err(new Error("failed to execute aptos call", { cause: error as Error }))
        }
      }
    }))
}
function toHex(uint8array: Uint8Array): string {
  return `0x${Array.from(uint8array)
    .map(b => b.toString(16).padStart(2, "0"))
    .join("")}`;  
}

function hexToAscii(hexString: string): string {
  // Remove the "0x" prefix if present.
  if (hexString.startsWith("0x") || hexString.startsWith("0X")) {
    hexString = hexString.slice(2);
  }
  let ascii = "";
  for (let i = 0; i < hexString.length; i += 2) {
    ascii += String.fromCharCode(parseInt(hexString.substr(i, 2), 16));
  }
  return ascii;
}
