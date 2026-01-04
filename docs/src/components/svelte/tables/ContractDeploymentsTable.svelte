<script lang="ts">
import * as Table from "#/components/svelte/ui/table/index.ts"
import deployments from "~root/deployments/deployments.json" with { type: "json" }

let addr_link = (chain_id: string, addr: string) => {
  switch (chain_id) {
    case ("arbitrum.42161"):
    case ("arbitrum.421614"):
    case ("base.8453"):
    case ("base.84532"):
    case ("bob.60808"):
    case ("bob.808813"):
    case ("bsc.56"):
    case ("bsc.97"):
    case ("ethereum.1"):
    case ("ethereum.11155111"):
    case ("sei.1328"):
    case ("sei.1329"): {
      return `https://dashboard.tenderly.co/contract/${chain_id.split(".")[1]}/${addr}`
    }
    case ("babylon.bbn-1"): {
      return `https://www.mintscan.io/osmosis/wasm/contract/${addr}/`
    }
    case ("corn.21000000"): {
      return `https://cornscan.io/address/${addr}`
    }
    case ("corn.21000001"): {
      return `https://testnet.cornscan.io/address/${addr}`
    }
    case ("intento.intento-dev-1"): {
      return `https://explorer.intento.zone/intento-devnet/cosmwasm/into18s3t0mx4ja5mclffsjheductwc72p786cled76/transactions?contract=${addr}`
    }
    case ("osmosis.osmo-test-5"): {
      return `https://www.mintscan.io/osmosis-testnet/wasm/contract/${addr}/`
    }
    case ("osmosis.osmosis-1"): {
      return `https://www.mintscan.io/osmosis/wasm/contract/${addr}/`
    }
    case ("sui.4c78adac"): {
      return `https://suiscan.xyz/testnet/object/${addr.split("::")[0]}/tx-blocks`
    }
    case ("union.union-1"): {
      return `https://explorer.union.build/union/cosmwasm/0/transactions?contract=${addr}`
    }
    case ("union.union-testnet-10"): {
      return `https://testnet.explorer.union.build/union/cosmwasm/0/transactions?contract=${addr}`
    }
    case ("xion.xion-testnet-2"): {
      return `https://explorer.burnt.com/xiontestnet2/cosmwasm/0/transactions?contract=${addr}`
    }
    default:
      console.error("???", chain_id, addr)
  }
}
</script>

{#each Object.entries(deployments) as [network, values]}
  <h3 class="font-mono">
    {network}
  </h3>
  <Table.Root class="w-full border border-neutral-500 rounded-sm">
    <Table.Row class="w-full">
      <Table.Cell>
        Address
      </Table.Cell>
      <Table.Cell>
        Name
      </Table.Cell>
    </Table.Row>
    {#each Object.entries(values.contracts).toSorted((a, b) => a[1].name < b[1].name ? -1 : 1) as
      [address, data]
    }
      <Table.Row>
        <Table.Cell class="font-mono">
          <a
            href={addr_link(network, address)}
            target="_blank"
          >{address}</a>
        </Table.Cell>
        <Table.Cell class="font-mono text-nowrap">
          {data.name}
        </Table.Cell>
      </Table.Row>
    {/each}
  </Table.Root>
{/each}
