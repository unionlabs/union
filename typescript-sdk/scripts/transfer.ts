import "#/patch.ts";
import { getBalance } from "#/query.ts";
import { unionActions } from "#/actions";
import { chain, isValidChainId } from "#/constants";
import {
  createPublicClient,
  createWalletClient,
  fallback,
  formatUnits,
  http,
  publicActions,
  walletActions,
} from "viem";
import { raise } from "#/utilities";
import { mnemonicToAccount } from "viem/accounts";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import type { ExecuteResult } from "@cosmjs/cosmwasm-stargate";

/**
 * bun ./scripts/transfer.ts \
 *   --fromChainId <'11155111' | '6'> \
 *   --toChainId <'11155111' | '6'> \
 *   --fromPrivateKey <privateKey | mnemonic> \
 *   --toAddress <address>
 *   --amount <amount>
 *
 * e.g.
bun ./scripts/transfer.ts \
  --fromChainId 6 \
  --toChainId 11155111 \
  --fromPrivateKey 'enlist hip relief stomach skate base shallow young switch frequent cry park' \
  --toAddress 0xCa091fE8005596E64ba9Cf028a75755a2380021A \
  --amount 555
 */

main().catch((_) => {
  console.error(_);
  process.exit(1);
});

async function main() {
  const [
    fromChainFlag,
    fromChainId,
    toChainFlag,
    toChainId,
    fromPrivateKeyFlag,
    fromPrivateKey,
    toAddressFlag,
    toAddress,
    amountFlag,
    amount,
  ] = process.argv.slice(2);

  if (
    !fromChainFlag ||
    !fromChainId ||
    !toChainFlag ||
    !toChainId ||
    !fromPrivateKeyFlag ||
    !fromPrivateKey ||
    !toAddressFlag ||
    !toAddress ||
    !amountFlag ||
    !amount
  ) {
    raise(
      "Usage: bun ./scripts/transfer.ts --fromChainId <chain> --toChainId <chain> --fromPrivateKey <privateKey | mnemonic> --toAddress <address>"
    );
  }

  if (!isValidChainId(fromChainId) || !isValidChainId(toChainId)) {
    raise(`Invalid chain: ${fromChainId} or ${toChainId}`);
  }

  const ethereumAccount = mnemonicToAccount(fromPrivateKey);
  const ethereumAddress =
    fromChainId === "11155111" ? ethereumAccount.address : toAddress;

  const unionAccount = await DirectSecp256k1HdWallet.fromMnemonic(
    fromPrivateKey,
    {
      prefix: "union",
    }
  );
  const [unionAccountData] = await unionAccount.getAccounts();
  const unionAddress =
    fromChainId === "6"
      ? unionAccountData?.address ?? raise("unionAddress is undefined")
      : toAddress;

  const fromAddress = fromChainId === "6" ? unionAddress : ethereumAddress;

  console.info(
    `\nABOUT TO SEND ${amount} muno\nFROM CHAIN ID: ${fromChainId} - ADDRESS: ${fromAddress}\nTO CHAIN ID ${toChainId} - ADDRESS: ${toAddress}\n`
  );

  const client = createWalletClient({
    chain: chain.ethereum.sepolia,
    account: ethereumAccount,
    transport: fallback([
      http(process.env.SEPOLIA_RPC_URL),
      http("https://ethereum-sepolia.publicnode.com"),
    ]),
  })
    .extend(publicActions)
    .extend(unionActions);

  if (fromChainId === "6") {
    const result = await client.sendAsset({
      chainId: "6",
      signer: unionAccount,
      assetId: chain.union.testnet.token.address,
      amount,
      denom: "muno",
      receiver: ethereumAddress,
      gasPrice: "0.001muno",
    });

    // console.log(JSON.stringify(result, undefined, 2))

    console.log(
      "SUCCESS. Transaction hash:\n",
      fromChainId === "6" ? (result as ExecuteResult).transactionHash : result
    );
    return;
  }

  const denomAddress = await client.getDenomAddress();

  const result = await client.sendAsset({
    chainId: "11155111",
    signer: ethereumAccount,
    portId: chain.ethereum.sepolia.portId,
    assetId: denomAddress,
    amount: BigInt(amount),
    receiver: unionAddress,
  });

  console.log("SUCCESS. Transaction hash:\n", result);
}
