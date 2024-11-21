<script lang="ts">
import { Debounced } from "runed"
import { dedent } from "ts-dedent"
import { cn } from "#/lib/shadcn.ts"
import curlSvg from "#/assets/icons/curl.svg?raw"
import * as Table from "#/components/svelte/ui/table/index.ts"
import { Button } from "#/components/svelte/ui/button/index.ts"
import externalLinkSvg from "#/assets/icons/external-link.svg?raw"
import { graphqlQueryToCurl, splitArray } from "#/lib/utilities.ts"
import * as Pagination from "#/components/svelte/ui/pagination/index.ts"
import GraphqlPlaygroundLink from "#/components/svelte/graphql-playground-link.svelte"

const graphqlQuery = dedent /* GraphQL */`
    query Chains {
      data: v1_chains {
        testnet
        chain_id
        logo_uri
        display_name
        explorers(limit: 1) {
          home_url
        }
      }
    }
  `

const curlCommand = graphqlQueryToCurl({
  query: graphqlQuery,
  url: "https://development.graphql.union.build/v1/graphql"
})

/**
 * set this as desired
 */
const rowsPerPage = 25
let pageNumber = $state(0)
const promise = $state(fetchChains())

let search = $state("")
const debouncedSearch = new Debounced(() => search.trim(), 1_000)

function filterRows(rows: Array<Array<string>>, inputSearch: string) {
  try {
    return rows.filter(row =>
      row.some(cell => cell?.toLowerCase()?.includes(inputSearch.toLowerCase()))
    )
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : error
    console.error(errorMessage)
    return rows
  }
}

async function fetchChains() {
  const response = await fetch("https://development.graphql.union.build/v1/graphql", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      query: graphqlQuery
    })
  })
  const json = await response.json()
  // @ts-expect-error
  const dataArray = json.data.data
  // @ts-expect-error
  const rows = dataArray.map(item => [
    item.testnet,
    item.chain_id,
    item.logo_uri,
    item.display_name,
    item.explorers?.at(0)?.home_url
  ]) as Array<Array<string>>

  return {
    data: {
      allRows: rows as Array<Array<string>>,
      total: rows.length,
      rowsChunks: splitArray({ array: rows, n: rowsPerPage }),
      headers: ["testnet", "chain id", "logo uri", "display name", "explorer"]
    }
  }
}
</script>

{#await promise}
  <div>Loading...</div>
{:then fetchedData}
  {@const { headers, rowsChunks, total, allRows } = fetchedData.data}
  {@const [perPage, count] = [Number(rowsChunks.at(0)?.length), total]}
  {@const rows = (
    debouncedSearch.current
      ? filterRows(allRows, debouncedSearch.current)
      : rowsChunks.at(pageNumber - 1 < 0 ? 0 : pageNumber - 1)
  ) as Array<Array<string>>}

  <section class="w-full flex h-min mt-4 justify-between align-middle gap-x-3">
    <GraphqlPlaygroundLink query={graphqlQuery} />
    <button
      type="button"
      title="Copy curl command"
      onclick={event => {
        navigator.clipboard.writeText(curlCommand)
        const element = event.currentTarget
        element.innerHTML = 'Copied!'
        setTimeout(() => {
          element.innerHTML = curlSvg
        }, 1_000)
      }}
      class="text-xs underline bg-transparent hover:bg-background/10 size-16 rounded-sm mr-auto hover:cursor-pointer font-mono"
    >
      {@html curlSvg}
    </button>

    <div class={cn('rounded-sm outline-[0.75px] w-1/2 my-auto')}>
      <input
        type="search"
        autocorrect="off"
        spellcheck="false"
        autocomplete="off"
        autocapitalize="off"
        placeholder="Search"
        bind:value={search}
        class={cn(
          'py-1 px-2 rounded-sm focus:outline-accent-200 focus-visible:ring-0 w-full my-auto outline outline-1 outline-neutral-500/70',
        )}
      />
    </div>
  </section>

  <Table.Root class="w-full border border-neutral-500 rounded-sm">
    <Table.Header class="w-full">
      <Table.Row class="w-full">
        {#each headers as header, index}
          {#if !header.includes('logo')}
            <Table.Head
              class={cn(
                'text-nowrap uppercase',
                index === 0 && 'w-[100px]',
                index === headers.length - 1 && 'text-right',
              )}
            >
              {header}
            </Table.Head>
          {/if}
        {/each}
      </Table.Row>
    </Table.Header>
    <Table.Body class="w-full">
      {#each rows as row, rowIndex}
        <Table.Row class={cn('w-full border-neutral-500')}>
          {#each row as cell, cellIndex}
            {@const lastColumn = cellIndex === row.length - 1}
            {@const logoColumn = cellIndex === 2}
            {#if logoColumn}
              <!--  -->
            {:else if lastColumn}
              {@const isUrl = URL.canParse(cell)}
              <Table.Cell class={cn('text-right text-nowrap border-neutral-500')}>
                {#if isUrl}
                  <Button
                    href={cell}
                    size="lg"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="hover:underline hover:text-accent-500 p-2 size-10 hover:bg-background/30 hover:cursor-pointer bg-transparent"
                  >
                    {@html externalLinkSvg}
                  </Button>
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

  <Pagination.Root {count} {perPage} class={cn(rowsPerPage >= count && 'hidden')}>
    {#snippet children({ pages, currentPage })}
      <Pagination.Content>
        <Pagination.Item>
          <Pagination.PrevButton class="mr-2 mt-1" onclick={_ => (pageNumber = currentPage)} />
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
