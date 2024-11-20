<script lang="ts">
import { Debounced } from "runed"
import { dedent } from "ts-dedent"
import { cn } from "#/lib/shadcn.ts"
import jsonSvg from "#/assets/icons/json.svg?raw"
import { highlightCode } from "#/lib/highlight-code.ts"
import { splitArray, stringIsJSON } from "#/lib/utilities.ts"
import * as Table from "#/components/svelte/ui/table/index.ts"
import * as Pagination from "#/components/svelte/ui/pagination/index.ts"

let pageNumber = $state(0)
let toggleRowIcon = $state(jsonSvg)
const promise = $state(fetchChannels())

let search = $state("")
const debounedSearch = new Debounced(() => search, 1_000)

function filterRows(rows: Array<Array<string>>, inputSearch: string) {
  return rows.filter(row =>
    row.some(cell => cell?.toLowerCase()?.includes(inputSearch.toLowerCase()))
  )
}

/**
 * set this as desired
 */
const rowsPerPage = 10

async function fetchChannels() {
  const response = await fetch("https://development.graphql.union.build/v1/graphql", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      query: /* GraphQL */ `
          query ChannelsForDocs {
            data: v1_channels {
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
  const dataArray = json.data.data
  // @ts-expect-error
  const rows = dataArray.map(item => [
    item.source_chain.display_name,
    item.source_connection_id?.split("-")?.at(-1),
    item.source_channel_id?.split("-")?.at(-1),
    item.status,
    item.version
  ]) as Array<Array<string>>

  return {
    data: {
      allRows: rows as Array<Array<string>>,
      total: rows.length,
      rowsChunks: splitArray({ array: rows, n: rowsPerPage }),
      headers: ["chain", "conn. #", "channel #", "status", "version"]
    }
  }
}

async function attachContent(event: MouseEvent, rowIndex: number, version: unknown) {
  let eventTarget = event.target?.closest("button") as HTMLElement
  if (!eventTarget.innerHTML) return

  const jsonSnippetElement = document.querySelector(`td[data-row-index="${rowIndex}"]`)
  if (!jsonSnippetElement) return
  const previousUncleElement = jsonSnippetElement.parentElement
    ?.previousElementSibling as HTMLTableRowElement

  const jsonSnippet = `\`\`\`json\n${JSON.stringify(version, undefined, 2)}`
  const highlightedCode = await highlightCode(dedent(jsonSnippet))

  jsonSnippetElement.innerHTML = highlightedCode
  // jsonSnippetElement.scrollIntoView({ behavior: 'smooth' })

  const state = jsonSnippetElement.dataset?.state || "collapsed"
  if (state === "collapsed") {
    previousUncleElement.style.setProperty(
      "background-color",
      "hsl(var(--muted) / 0.1)",
      "important"
    )
    eventTarget.innerHTML = "▼"
    jsonSnippetElement.dataset.state = "expanded"
  } else {
    previousUncleElement.style.removeProperty("background-color")
    eventTarget.innerHTML = jsonSvg
    jsonSnippetElement.innerHTML = ""
    jsonSnippetElement.dataset.state = "collapsed"
  }
}
</script>

{#await promise}
  <div>Loading...</div>
{:then data}
  {@const { headers, rowsChunks, total, allRows } = data.data}
  {@const [perPage, count] = [Number(rowsChunks.at(0)?.length), total]}
  {@const rows = (
    debounedSearch.current
      ? filterRows(allRows, debounedSearch.current)
      : rowsChunks.at(pageNumber - 1 < 0 ? 0 : pageNumber - 1)
  ) as Array<Array<string>>}

  <div class={cn('mt-4 outline outline-accent-200/50 rounded-sm outline-[0.75px] w-1/2 ml-auto')}>
    <input
      type="search"
      autocorrect="off"
      spellcheck="false"
      autocomplete="off"
      autocapitalize="off"
      placeholder="Search"
      bind:value={search}
      class={cn(
        'py-1 px-2 border border-neutral-500 rounded-sm focus:outline-none focus-visible:ring-0 w-full',
      )}
    />
  </div>

  <Table.Root class="w-full border border-neutral-500 rounded-sm">
    <Table.Header class="w-full">
      <Table.Row class="w-full">
        {#each headers as header, index}
          <Table.Head
            class={cn(
              'text-nowrap uppercase',
              index === 0 && 'w-[100px]',
              index === headers.length - 1 && 'text-right',
            )}
          >
            {header}
          </Table.Head>
        {/each}
      </Table.Row>
    </Table.Header>
    <Table.Body class="w-full">
      {#each rows as row, rowIndex}
        <Table.Row class={cn('w-full border-neutral-500')}>
          {#each row as cell, cellIndex}
            {@const lastColumn = cellIndex === row.length - 1}
            {#if lastColumn}
              <Table.Cell class={cn('text-right text-nowrap border-neutral-500')}>
                {@const isJSON = stringIsJSON(cell)}
                {#if isJSON}
                  {@const version = JSON.parse(cell)}
                  <button
                    onclick={event => attachContent(event, rowIndex, version)}
                    class="bg-transparent hover:bg-background/30 hover:cursor-pointer size-8"
                  >
                    {@html toggleRowIcon}
                  </button>
                {:else}
                  {cell}
                {/if}
              </Table.Cell>
            {:else}
              <Table.Cell
                class={cn(
                  'border-neutral-500',
                  cellIndex === 0 && 'font-medium w-[135px] text-nowrap',
                )}
              >
                {cell}
              </Table.Cell>
            {/if}
          {/each}
        </Table.Row>
        <Table.Row class="border-none">
          <Table.Cell
            colspan={5}
            class="p-0 border-transparent"
            data-json-snippet
            data-row-index={rowIndex}
          ></Table.Cell>
        </Table.Row>
      {/each}
    </Table.Body>
  </Table.Root>

  <Pagination.Root {count} {perPage}>
    {#snippet children({ pages, currentPage })}
      <Pagination.Content>
        <Pagination.Item>
          <Pagination.PrevButton class="mr-2 -mb-1" onclick={_ => (pageNumber = currentPage)} />
        </Pagination.Item>
        {#each pages as page (page.key)}
          {#if page.type === 'ellipsis'}
            <Pagination.Item>
              <Pagination.Ellipsis />
            </Pagination.Item>
          {:else}
            <Pagination.Item isVisible={currentPage === page.value}>
              <Pagination.Link
                {page}
                isActive={currentPage === page.value}
                onclick={_ => (pageNumber = page.value)}
              >
                {page.value}
              </Pagination.Link>
            </Pagination.Item>
          {/if}
        {/each}
        <Pagination.Item>
          <Pagination.NextButton class="ml-2" onclick={_ => (pageNumber = currentPage)} />
        </Pagination.Item>
      </Pagination.Content>
    {/snippet}
  </Pagination.Root>
{:catch error}
  <div>Error: {error.message}</div>
{/await}

<style lang="postcss">
  :global(.sl-markdown-content table:not(:where(.not-content *))) {
    display: table;
  }

  :global(.sl-markdown-content th:not(:where(.not-content *))) {
    border-bottom: 0.8px solid #a1a1ab;
  }

  :global(li::marker) {
    color: transparent;
  }

  :global(.rehype-pretty-copy) {
    background-color: transparent;
  }

  :global(pre, figure) {
    border-top: 0px solid transparent !important;
  }

  :global(table) {
    width: 100%;
    min-width: 100%;
    display: table;
  }
</style>
