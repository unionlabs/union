<script lang="ts">
  import { cn } from '$lib/utilities/shadcn.ts'
  import ChevronLeft from 'virtual:icons/lucide/chevron-left'
  import ChevronRight from 'virtual:icons/lucide/chevron-right'
  import { toPrettyDateTimeFormat } from '$lib/utilities/date.ts'
  import * as Pagination from '$lib/components/ui/pagination/index.ts'

  export let timestamp: string
  export let rowsPerPage: number
  export let totalTableRows: number

  export let status: 'pending' | 'done' = 'done'

  export let onOlderPage: (page: number) => void
  export let onNewerPage: (page: number) => void
</script>

<Pagination.Root
  let:currentPage
  siblingCount={2000}
  perPage={rowsPerPage}
  count={totalTableRows}
  class={cn()}
>
  <Pagination.Content class="py-2 text-md uppercase font-supermolot">
    <Pagination.Item>
      <Pagination.PrevButton
        disabled={status === 'pending'}
        class={cn(status === 'pending' ? 'cursor-not-allowed' : '')}
        on:click={event => onNewerPage(Number(currentPage) - 1)}
      >
        <ChevronLeft class="size-6" />
        <span class="hidden sm:block text-md uppercase font-supermolot">Newer</span>
      </Pagination.PrevButton>
    </Pagination.Item>
    <div class={cn('flex flex-col align-baseline gap-y-0.5')}>
      <time class="font-normal text-md uppercase font-mono">
        {toPrettyDateTimeFormat(timestamp)}
      </time>
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
        disabled={status === 'pending'}
        on:click={event => onOlderPage(Number(currentPage) + 1)}
      >
        <span class="hidden sm:block text-md uppercase font-supermolot">Older</span>
        <ChevronRight class="size-6" />
      </Pagination.NextButton>
    </Pagination.Item>
  </Pagination.Content>
</Pagination.Root>
