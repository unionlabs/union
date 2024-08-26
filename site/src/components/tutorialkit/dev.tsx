import "viem/window"
import { custom } from "viem"
import * as React from "react"
import { sepolia } from "viem/chains"
import { Addreth, AddrethConfig } from "addreth/no-wagmi"
import { createCosmosSdkClient as createUnionClient } from "@union/client"

export default () => {
  const [connected, setConnected] = React.useState(false)
  const [account, setAccount] = React.useState<`0x${string}` | undefined>(undefined)
  const [client, setClient] = React.useState<ReturnType<typeof createUnionClient>>()

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
    if (!(window.ethereum && client)) return
    if (connected) return [setAccount(undefined), setConnected(false)]

    const [account] = await client.request({ method: "eth_requestAccounts" })
    return [setAccount(account), setConnected(true)]
  }

  return (
    <main className="flex flex-col items-center justify-center gap-4 p-4">
      <section className="h-12" aria-label="Wallet Address">
        <AddrethConfig ens={true}>
          {client?.account?.address && (
            <Addreth
              address={client?.account?.address}
              ens={true}
              theme="dark"
              icon="ens"
              actions="none"
            />
          )}
        </AddrethConfig>
      </section>

      <button
        type="button"
        onClick={onConnectClick}
        className="rounded-sm hover:cursor-pointer hover:bg-gray-500 hover:text-white"
      >
        {connected ? "Disconnect" : "Connect"}
      </button>
    </main>
  )
}
