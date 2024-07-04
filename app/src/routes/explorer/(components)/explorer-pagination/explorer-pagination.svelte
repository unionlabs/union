<script lang="ts">
  import { cn } from '$lib/utilities/shadcn.ts'
  import ChevronLeft from 'virtual:icons/lucide/chevron-left'
  import ChevronRight from 'virtual:icons/lucide/chevron-right'
  import * as Pagination from '$lib/components/ui/pagination/index.ts'

  export let timestamp: string
  export let rowsPerPage: number
  export let totalTableRows: number

  export let onNextPage: (page: number) => void
  export let onPreviousPage: (page: number) => void
</script>

<Pagination.Root
  let:pages
  let:currentPage
  siblingCount={2000}
  perPage={rowsPerPage}
  count={totalTableRows}
>
  <Pagination.Content class="py-2">
    <Pagination.Item>
      <Pagination.PrevButton
        disabled={currentPage === 1}
        on:click={() => onPreviousPage(Number(currentPage) - 1)}
      >
        <ChevronLeft class="h-4 w-4" />
        <span class="hidden sm:block">Previous</span>
      </Pagination.PrevButton>
    </Pagination.Item>
    <div class={cn('flex flex-col align-baseline gap-y-0.5')}>
      <time class="">{timestamp}</time>
      <!-- <div class={cn('flex flex-row')}>
        {#each pages as page (page.key)}
          {#if page.type === 'ellipsis'}
            <Pagination.Item>
              <Pagination.Ellipsis />
            </Pagination.Item>
          {:else}
            <Pagination.Item>
              <Pagination.Link class="text-xs" {page} isActive={currentPage === page.value}>
                {page.value}
              </Pagination.Link>
            </Pagination.Item>
          {/if}
        {/each}
      </div> -->
    </div>
    <Pagination.Item>
      <Pagination.NextButton
        disabled={currentPage === pages.length}
        on:click={() => onNextPage(Number(currentPage) + 1)}
      >
        <span class="hidden sm:block">Next</span>
        <ChevronRight class="h-4 w-4" />
      </Pagination.NextButton>
    </Pagination.Item>
  </Pagination.Content>
  <pre>{JSON.stringify(
      { timestamp, rowsPerPage, currentPage, offset: (Number(currentPage) - 1) * rowsPerPage },
      undefined,
      2,
    )}</pre>
</Pagination.Root>

<style lang="postcss"></style>
