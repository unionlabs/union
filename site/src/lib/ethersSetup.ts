import { ethers } from "ethers";
import {
  ethereumAddress,
  ethersProvider,
  ethersSigner,
  connectedToSepolia,
  snapInstalled,
  connectedToUnion,
} from "./stores/wallets";
import { get } from "svelte/store";

const SEPOLIA_CHAIN_ID = "0xaa36a7";

export const updateConnectedToSepolia = async () => {
  if (window.ethereum === undefined) {
    console.error(
      "trying to update connected to sepolia with no metamask installed"
    );
  }
  const currentChainId = await window.ethereum.request({
    method: "eth_chainId",
  });
  connectedToSepolia.set(currentChainId === SEPOLIA_CHAIN_ID);
};

export const updateSnapInstalled = async () => {
  //@ts-ignore
  window.process = { env: {} };
  const { getSnap } = await import("@leapwallet/cosmos-snap-provider");
  const snap = await getSnap();
  snapInstalled.set(snap !== undefined);
};

export const connectToSepolia = async () => {
  if (window.ethereum === undefined) {
    console.error("trying to set up ethers while metamask is not installed");
  }

  const eProvider = new ethers.BrowserProvider(window.ethereum);

  eProvider.on("network", (newNetwork, oldNetwork) => {
    // When a Provider makes its initial connection, it emits a "network"
    // event with a null oldNetwork along with the newNetwork. So, if the
    // oldNetwork exists, it represents a changing network
    if (oldNetwork) {
      window.location.reload();
    }
  });
  ethersProvider.set(eProvider);

  /// Requests the end user to switch to sepolia.
  await window.ethereum.request({
    method: "wallet_switchEthereumChain",
    params: [{ chainId: SEPOLIA_CHAIN_ID }],
  });

  updateConnectedToSepolia();
  if (get(connectedToSepolia) == false) {
    console.error("did not properly connect to sepolia");
  }
};

export const ethersSetup = async () => {
  const eProvider = new ethers.BrowserProvider(window.ethereum);
  const allAccounts = await eProvider.listAccounts();
  if (allAccounts.length === 0) {
    console.log("0 accounts avail");
  }
  const eSigner = await eProvider.getSigner(0);
  const eAddress = await eSigner.getAddress();
  ethersSigner.set(eSigner);
  ethereumAddress.set(eAddress);
  ethersProvider.set(eProvider);
};

export const connectLeapSnap = async () => {
  const { getSnap, connectSnap, getKey } = await import(
    "@leapwallet/cosmos-snap-provider"
  );
  //@ts-ignore
  window.process = { env: {} };
  const snap = await getSnap();
  if (snap === undefined) {
    await connectSnap(); // Initiates installation if not already present
  }
  await updateSnapInstalled();
};

const UNION_CHAIN_ID = "union-testnet-3";

export const connectToUnion = async () => {
  const { suggestChain, getKey } = await import(
    "@leapwallet/cosmos-snap-provider"
  );

  await suggestChain(
    {
      chainId: UNION_CHAIN_ID,
      chainName: "union-testnet",
      bip44: { coinType: 118 },
      bech32Config: {
        bech32PrefixAccAddr: "union",
      },
    },
    { force: false }
  );
  connectedToUnion.set(true);
};

export const updateConnectedToUnion = async () => {
  const { suggestChain, getKey } = await import(
    "@leapwallet/cosmos-snap-provider"
  );
  try {
    const key = await getKey(UNION_CHAIN_ID);
    connectedToUnion.set(true);
  } catch {
    // not connected to union yet
    connectedToUnion.set(false);
  }
};
