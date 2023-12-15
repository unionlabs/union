#!/usr/bin/env bun
import "#/patch.ts";
import { raise } from "#/utilities";
import { unionActions } from "#/actions.ts";
import { privateKeyToAccount } from "viem/accounts";
import { http, publicActions, createWalletClient } from "viem";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import {
  UCS01_EVM_ADDRESS,
  demoPrivateKey,
  demoMnemonic,
  chain,
} from "#/constants";

main().catch((_) => {
  console.error(_);
  process.exit(1);
});

async function main() {
  const demoEthereumAccount = privateKeyToAccount(demoPrivateKey);
  const demoEthereumAddress = demoEthereumAccount.address;
  const demoUnionAccount = await DirectSecp256k1HdWallet.fromMnemonic(
    demoMnemonic,
    {
      prefix: "union",
    }
  );
  const [demoUnionAccountData] = await demoUnionAccount.getAccounts();
  const demoUnionAddress =
    demoUnionAccountData?.address ?? raise("demoUnionAddress is undefined");

  const { sepolia } = chain.ethereum;
  const { testnet: unionTestnet } = chain.union;

  const client = createWalletClient({
    chain: sepolia,
    account: demoEthereumAccount,
    transport: http(process.env.SEPOLIA_RPC_URL),
  })
    .extend(publicActions)
    .extend(unionActions);

  await client
    .approveAsset({
      chainId: "11155111",
      signer: demoEthereumAccount,
      amount: 500n,
      spender: UCS01_EVM_ADDRESS,
      assetId: sepolia.token.address,
    })
    .then((_) => console.log({ approvalTransactionHash: _ }));

  await client
    .sendAsset({
      chainId: "11155111",
      signer: demoEthereumAccount,
      assetId: sepolia.token.address,
      amount: 1n,
      receiver: demoUnionAddress,
    })
    .then((_) => console.log({ sendAssetFromEthereumToUnion: _ }));

  const balanceOnEthereum = await client.getBalance({
    chainId: "11155111",
    address: demoEthereumAddress,
    assetId: sepolia.token.address,
  });

  const balanceOnUnion = await client.getBalance({
    chainId: "32382",
    address: demoUnionAddress,
    assetId: unionTestnet.token.denom,
  });

  await client
    .sendAsset({
      chainId: "32382",
      signer: demoUnionAccount,
      assetId: unionTestnet.token.address,
      amount: "100",
      denom: "muno",
      receiver: demoEthereumAddress,
      gasPrice: "0.001muno",
    })
    .then((_) => console.log(JSON.stringify({ _ }, undefined, 2)));
}
