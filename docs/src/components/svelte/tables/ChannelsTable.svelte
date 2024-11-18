<script lang="ts">
  let promise = $state(fetchChannels())

  async function fetchChannels() {
    const response = await fetch('https://development.graphql.union.build/v1/graphql', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
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
        `,
      }),
    })
    const json = await response.json()
    // @ts-expect-error
    const dataArray = json.data.data
    return {
      data: {
        headers: ['channel', 'source label', 'source chain', 'dest. label', 'dest. chain'],
        // @ts-expect-error
        rows: dataArray.map(item => [
          item.channel_id.split('-').at(-1),
          item.source_chain.display_name,
          `${item.source_chain.rpc_type}/${item.source_chain.chain_id}`,
          item.destination_chain.display_name,
          `${item.destination_chain.rpc_type}/${item.destination_chain.chain_id}`,
        ]),
      },
    }
  }
</script>

<div class="px-4 sm:px-6 lg:px-8 flex justify-center w-full">
  <div class="mt-8 flow-root">
    <div class="overflow-x-auto">
      <div class="inline-block min-w-full align-middle border rounded-sm border-neutral-400">
        {#await promise}
          <p>Loading...</p>
        {:then data}
          {@const { headers, rows } = data.data}
          <table class="divide-y divide-gray-300 tabular-nums">
            <thead class="w-full min-w-full">
              <tr class="divide-x divide-gray-200 w-full">
                {#each headers as header, index}
                  <th
                    scope="col"
                    class:list={[
                      `py-3.5 text-left text-md font-semibold`,
                      index === 0
                        ? 'pl-4'
                        : index === headers.length - 1
                          ? 'pl-4 text-center'
                          : 'px-4',
                    ]}
                  >
                    {header}
                  </th>
                {/each}
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-200 w-full min-w-full">
              {#each rows as row, index}
                <tr class="w-full">
                  {#each row as cell, index}
                    <td class="whitespace nowrap p-4 text-sm font-medium">{cell}</td>
                  {/each}
                </tr>
              {/each}
            </tbody>
          </table>
        {:catch error}
          <p>{error.message}</p>
        {/await}
      </div>
    </div>
  </div>
</div>

<style lang="postcss"></style>
