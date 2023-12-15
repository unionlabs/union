import {
  unionAccount,
  ethersProvider,
  ethersSigner,
  ethereumAddress,
} from "$lib/stores/wallets";

import { fromBech32 } from "@cosmjs/encoding";

import { ethers } from "ethers";
import { get } from "svelte/store";
import {
  MUNO_ERC20_ADDRESS,
  ERC20_CONTRACT_ABI,
  UCS01_EVM_ADDRESS,
  UCS01_RELAY_EVM_ABI,
  UCS01_SEPOLIA_SOURCE_CHANNEL,
  UCS01_SEPOLIA_PORT_ID,
  AMOUNT_TO_SEND_TO_UNION,
} from "./constants";
import { sendingUnoToUnion } from "../routes/blog/ics20-transfers-to-ethereum/demoStore";

export const approveUnoTransferToUnion = async () => {
  const _eProvider = get(ethersProvider);
  const eSigner = get(ethersSigner);
  const _eAddress = get(ethereumAddress);
  const contract = new ethers.Contract(
    MUNO_ERC20_ADDRESS,
    ERC20_CONTRACT_ABI,
    eSigner
  );

  const tx = await contract.approve(UCS01_EVM_ADDRESS, AMOUNT_TO_SEND_TO_UNION);
  await tx.wait();
};

export const sendUnoToUnion = async () => {
  const eProvider = get(ethersProvider);
  const eAddress = get(ethereumAddress);
  const eSigner = get(ethersSigner);
  const uAccount = get(unionAccount);

  if (
    eProvider === null ||
    eAddress === null ||
    eSigner === null ||
    uAccount === null
  ) {
    console.error("missing dependencies for transferFromSepolia");
    return;
  }

  await approveUnoTransferToUnion();

  const contract = new ethers.Contract(
    MUNO_ERC20_ADDRESS,
    ERC20_CONTRACT_ABI,
    eProvider
  );

  const _erc20balance = await contract.balanceOf(eAddress);

  const ibcContract = new ethers.Contract(
    UCS01_EVM_ADDRESS,
    UCS01_RELAY_EVM_ABI,
    eSigner
  );

  // string calldata portId,
  // string calldata channelId,
  // string calldata receiver,
  // LocalToken[] calldata tokens,
  // uint64 counterpartyTimeoutRevisionNumber,
  // uint64 counterpartyTimeoutRevisionHeight
  const tx = await ibcContract.send(
    UCS01_SEPOLIA_PORT_ID,
    UCS01_SEPOLIA_SOURCE_CHANNEL,
    fromBech32(uAccount.address).data,
    [[MUNO_ERC20_ADDRESS, AMOUNT_TO_SEND_TO_UNION]],
    4,
    800000000
  );

  sendingUnoToUnion.set("sending");

  await tx.wait();
};
