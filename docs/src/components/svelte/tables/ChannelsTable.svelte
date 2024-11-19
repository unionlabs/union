<script lang="ts">
import { cn } from "#/lib/shadcn"
import { dedent } from "ts-dedent"
import { stringIsJSON } from "#/lib/utilities.ts"
import jsonSvg from "#/assets/icons/json.svg?raw"
import { Button } from "#/components/svelte/ui/button"
import { highlightCode } from "#/lib/highlight-code.ts"
import * as Table from "#/components/svelte/ui/table/index.ts"

let promise = $state(fetchChannels())

async function fetchChannels() {
  const response = await fetch("https://development.graphql.union.build/v1/graphql", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      query: /* GraphQL */ `
          query ChannelsForDocs {
            v1_channels {
              source_chain {
                display_name
                chain_id
              }
              source_channel_id
              source_connection_id
              source_port_id
              status
              version
            }
          }
        `
    })
  })
  const json = await response.json()
  // @ts-expect-error
  const dataArray = json.data.v1_channels

  let trim = (item: string | null, length: number) => {
    if (item != null) {
      return `${item.substring(0, length)}...`
    }
    item
  }

  return {
    data: {
      headers: ["chain", "connection", "channel", "status", "version"],
      // @ts-expect-error
      rows: dataArray.map(item => {
        return [
          item.source_chain.display_name,
          item.source_connection_id,
          item.source_channel_id,
          item.status,
          item.version
        ]
      })
    }
  }
}
</script>

{#await promise}
  <div>Loading...</div>
{:then data}
  {@const { headers, rows } = data.data}
  <Table.Root class="w-full">
    <Table.Header class="w-full">
      <Table.Row class="w-full">
        {#each headers as header, index}
          <Table.Head
            class={cn(index === 0 && 'w-[100px]', index === headers.length - 1 && 'text-right')}
          >
            {header}
          </Table.Head>
        {/each}
      </Table.Row>
    </Table.Header>
    <Table.Body class="w-full">
      {#each rows as row, rowIndex}
        <Table.Row class="w-full">
          {#each row as cell, cellIndex}
            {@const lastColumn = cellIndex === row.length - 1}
            {#if lastColumn}
              <Table.Cell class={cn('text-right w-min')}>
                {@const isJSON = stringIsJSON(cell)}
                {#if isJSON}
                  {@const version = JSON.parse(cell)}
                  <Button
                    size="icon"
                    variant="ghost"
                    class="size-8 bg-transparent hover:bg-background/30 hover:cursor-pointer"
                    onclick={async event => {
                      event.preventDefault()

                      const jsonSnippetElement = document.querySelector(
                        `td[data-row-index="${rowIndex}"]`,
                      )
                      if (!jsonSnippetElement) return

                      const jsonSnippet = `\`\`\`json\n${JSON.stringify(version, undefined, 2)}`
                      const highlightedCode = await highlightCode(dedent(jsonSnippet))

                      jsonSnippetElement.innerHTML = highlightedCode
                      // jsonSnippetElement.scrollIntoView({ behavior: 'smooth' })
                      const state = jsonSnippetElement.dataset?.state || 'collapsed'
                      if (state === 'collapsed') {
                        jsonSnippetElement.dataset.state = 'expanded'
                      } else {
                        jsonSnippetElement.innerHTML = ''
                        jsonSnippetElement.dataset.state = 'collapsed'
                      }
                    }}
                  >
                    {@html jsonSvg}
                  </Button>
                {:else}
                  {cell}
                {/if}
              </Table.Cell>
            {:else}
              <Table.Cell class={cn(cellIndex === 0 && 'font-medium w-[130px]')}>
                {cell}
              </Table.Cell>
            {/if}
          {/each}
        </Table.Row>
        <Table.Row>
          <Table.Cell colspan={5} class="p-0" data-json-snippet data-row-index={rowIndex}
          ></Table.Cell>
        </Table.Row>
      {/each}
    </Table.Body>
  </Table.Root>
{:catch error}
  <div>Error: {error.message}</div>
{/await}

<style lang="postcss">
  :global(table) {
    width: 100%;
    min-width: 100%;
    display: table;
  }
</style>
