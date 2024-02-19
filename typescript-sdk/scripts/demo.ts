#!/usr/bin/env bun
import "#/patch.ts";
import { raise } from "#/utilities";
import { unionActions } from "#/actions.ts";
import { http, publicActions, createWalletClient, fallback } from "viem";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { privateKeyToAccount, mnemonicToAccount } from "viem/accounts";
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
  // const demoEthereumAccount = privateKeyToAccount(demoPrivateKey)
  const demoEthereumAccount = mnemonicToAccount(demoMnemonic);
  // 0x3a7c1964ea700Ee19887c747C72e68F84Cb9C5DD
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
  console.log(demoUnionAddress, demoEthereumAddress);

  const SEPOLIA_RPC_URL = process.env.SEPOLIA_RPC_URL;
  const { sepolia } = chain.ethereum;
  const { testnet: unionTestnet } = chain.union;

  const client = createWalletClient({
    chain: sepolia,
    account: demoEthereumAccount,
    transport: fallback([
      http(SEPOLIA_RPC_URL),
      http("https://ethereum-sepolia.publicnode.com"),
    ]),
  })
    .extend(publicActions)
    .extend(unionActions);

  const denomAddress = await client.getDenomAddress();
  console.log({ denomAddress });

  // await client
  //   .getBalance({
  //     chainId: '6',
  //     address: demoUnionAddress,
  //     assetId: unionTestnet.token.denom,
  //   })
  //   .then(_ => console.log({ balanceOnUnion: _ }))

  // await client
  //   .sendAsset({
  //     chainId: '6',
  //     signer: demoUnionAccount,
  //     assetId: unionTestnet.token.address,
  //     amount: '1234',
  //     denom: 'muno',
  //     receiver: demoEthereumAddress,
  //     gasPrice: '0.001muno',
  //   })
  //   .then(_ => console.log(JSON.stringify(_, undefined, 2)))

  // await client
  //   .getBalance({
  //     chainId: '11155111',
  //     address: demoEthereumAddress,
  //   })
  //   .then(_ => console.log({ balanceOnSepolia: _ }))

  // await client
  //   .approveAsset({
  //     chainId: '11155111',
  //     signer: demoEthereumAccount,
  //     amount: 135920n,
  //     spender: UCS01_EVM_ADDRESS,
  //     assetId: denomAddress,
  //   })
  //   .then(_ => console.log({ approvalTransactionHash: _ }))

  // await client
  //   .sendAsset({
  //     chainId: '11155111',
  //     portId: sepolia.portId,
  //     signer: demoEthereumAccount,
  //     assetId: denomAddress,
  //     amount: 100n,
  //     receiver: demoUnionAddress,
  //     simulate: true,
  //   })
  //   .then(_ => console.log({ sendAssetFromEthereumToUnion: _ }))
}
