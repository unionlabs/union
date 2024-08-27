import "viem/window";
import * as React from "react";
import { arbitrumSepolia } from "viem/chains";
import { custom, formatEther, toHex } from "viem";
import { Addreth, AddrethConfig } from "addreth/no-wagmi";
import {
  createCosmosSdkClient as createUnionClient,
  type TransferAssetsParameters,
} from "@union/client";

export default function TypeScriptSdkDemo() {
  const [connected, setConnected] = React.useState(false);
  const [balance, setBalance] = React.useState<bigint | undefined>(undefined);
  const [account, setAccount] = React.useState<`0x${string}` | undefined>(
    undefined,
  );
  const [client, setClient] =
    React.useState<ReturnType<typeof createUnionClient>>();

  const [hash, setHash] = React.useState<string | undefined>(undefined);

  React.useEffect(() => {
    if (!window.ethereum) return;
    const client = createUnionClient({
      evm: {
        account,
        chain: arbitrumSepolia,
        transport: custom(window.ethereum),
      },
    });
    setClient(client);
  }, [account]);

  const onConnectClick = async (event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    if (!client) return;
    if (connected)
      return [
        setAccount(undefined),
        setConnected(false),
        setBalance(undefined),
      ];

    const [address] = await client.request({ method: "eth_requestAccounts" });
    await client.request({
      method: "wallet_switchEthereumChain",
      params: [{ chainId: toHex(arbitrumSepolia.id) }],
    });
    const balance = await client.getBalance({ address });
    return [setAccount(address), setBalance(balance), setConnected(true)];
  };

  const transferPayload = {
    amount: 1n,
    approve: true,
    network: "evm",
    sourceChannel: "channel-0",
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
    denomAddress: "0xb1d4538b4571d411f07960ef2838ce337fe1e80e",
    relayContractAddress: "0xBd346331b31f8C43CC378286Bfe49f2f7F128c39",
    path: ["421614", "union-testnet-8"],
  } satisfies TransferAssetsParameters;

  const onTransferClick = async (
    event: React.MouseEvent<HTMLButtonElement>,
  ) => {
    event.preventDefault();
    if (!client) return;

    const transfer = await client.transferAsset(transferPayload);

    console.info(transfer);

    if (transfer.success) setHash(transfer.data);
  };

  return (
    <main className="flex flex-col items-center justify-center">
      <section
        className="mx-auto w-full items-center justify-center"
        aria-label="Wallet Address"
      >
        <div className="text-center">
          <AddrethConfig ens={true}>
            {client?.account?.address && (
              <Addreth
                ens={true}
                icon="ens"
                theme="dark"
                actions="none"
                address={client?.account?.address}
              />
            )}
          </AddrethConfig>
        </div>
        {typeof balance === "bigint" && (
          <h6 className="border-none text-center">
            ETH {formatEther(balance)}
          </h6>
        )}
      </section>

      <button
        type="button"
        onClick={onConnectClick}
        className="rounded-sm bg-accent-800 px-2 py-1 font-bold text-sm text-white uppercase hover:cursor-pointer hover:bg-accent-700"
      >
        {connected ? "Disconnect" : "Connect"}
      </button>

      <section aria-label="transfer">
        <button type="button" onClick={onTransferClick}>
          transfer
        </button>
      </section>

      <a
        target="_blank"
        rel="noreferrer noopener"
        href={`https://app.union.build/explorer/transfers/${hash}`}
      >
        {hash}
      </a>
    </main>
  );
}
