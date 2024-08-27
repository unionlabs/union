import "viem/window"
import { custom, formatEther } from "viem"
import * as React from "react"
import { sepolia } from "viem/chains"
import { codeToHtml } from "shiki"
import { Addreth, AddrethConfig } from "addreth/no-wagmi"
import {
  createCosmosSdkClient as createUnionClient,
  type TransferAssetsParameters
} from "@union/client"

export default function TypeScriptSdkDemo() {
  const [connected, setConnected] = React.useState(false)
  const [balance, setBalance] = React.useState<bigint | undefined>(undefined)
  const [account, setAccount] = React.useState<`0x${string}` | undefined>(undefined)
  const [client, setClient] = React.useState<ReturnType<typeof createUnionClient>>()

  const [hash, setHash] = React.useState<string | undefined>(undefined)

  React.useEffect(() => {
    if (!window.ethereum) return
    const client = createUnionClient({
      evm: {
        account,
        chain: sepolia,
        transport: custom(window.ethereum)
      }
    })
    setClient(client)
  }, [account])

  const onConnectClick = async (event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault()
    if (!client) return
    if (connected) return [setAccount(undefined), setConnected(false), setBalance(undefined)]

    const [address] = await client.request({ method: "eth_requestAccounts" })
    const balance = await client.getBalance({ address })
    return [setAccount(address), setBalance(balance), setConnected(true)]
  }

  const transferPayload = {
    amount: 1n,
    approve: true,
    network: "evm",
    sourceChannel: "channel-90",
    path: ["11155111", "union-testnet-8"],
    recipient: "0x0000000000000000000000000000000000000000",
    denomAddress: "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238",
    relayContractAddress: "0xd0081080ae8493cf7340458eaf4412030df5feeb"
  } satisfies TransferAssetsParameters

  const onTransferClick = async (event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault()
    if (!client) return

    const transfer = await client.transferAsset(transferPayload)

    if (transfer.success) setHash(transfer.data)
  }

  const [highlightedPayload, setHighlightedPayload] = React.useState()

  React.useEffect(() => {
    // highlightCode(JSON.stringify(transferPayload)).then(_ => console.info(_))
    // console.info(transferPayload)
    codeToHtml(JSON.stringify(transferPayload, null, 2), {
      lang: "json",
      theme: "catppuccin-mocha"
    }).then(html => setHighlightedPayload(html))
  }, [transferPayload])
  return (
    <main className="flex flex-col items-center justify-center">
      <section className="mx-auto w-full items-center justify-center" aria-label="Wallet Address">
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
          <h6 className="border-none text-center">ETH {formatEther(balance)}</h6>
        )}
      </section>

      <button
        type="button"
        onClick={onConnectClick}
        className="rounded-sm bg-accent-800 px-2 py-1 font-bold text-sm text-white uppercase hover:cursor-pointer hover:bg-accent-700"
      >
        {connected ? "Disconnect" : "Connect"}
      </button>

      {connected && <div dangerouslySetInnerHTML={{ __html: highlightedPayload }}></div>}

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
  )
}
