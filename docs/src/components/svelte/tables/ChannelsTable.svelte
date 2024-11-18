<script lang="ts">
import Table from "#/components/svelte/Table.svelte"

let promise = $state(fetchChannels())

async function fetchChannels() {
  const response = await fetch("https://development.graphql.union.build/v1/graphql", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      query: /* GraphQL */ `
          query v1UcsPathsQuery {
            data: v1_ucs1_paths {
              channel_id
              source_chain {
                chain_id
                rpc_type
                display_name
              }
              destination_chain {
                chain_id
                rpc_type
                display_name
              }
            }
          }
        `
    })
  })
  const json = await response.json()
  // @ts-expect-error
  const dataArray = json.data.data
  return {
    data: {
      headers: ["channel", "source label", "source chain", "dest. label", "dest. chain"],
      // @ts-expect-error
      rows: dataArray.map(item => [
        item.channel_id.split("-").at(-1),
        item.source_chain.display_name,
        `${item.source_chain.rpc_type}/${item.source_chain.chain_id}`,
        item.destination_chain.display_name,
        `${item.destination_chain.rpc_type}/${item.destination_chain.chain_id}`
      ])
    }
  }
}
</script>

{#await promise}
  <div>Loading...</div>
{:then data}
  {@const { headers, rows } = data.data}
  <Table {rows} {headers} />
{:catch error}
  <div>Error: {error.message}</div>
{/await}

<style lang="postcss"></style>
