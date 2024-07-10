<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import ChevronLeft from "virtual:icons/lucide/chevron-left"
import Button from "$lib/components/ui/button/button.svelte"
import ChevronRight from "virtual:icons/lucide/chevron-right"
import * as Pagination from "$lib/components/ui/pagination/index.ts"

export let rowsPerPage: number
export let totalTableRows: number

export let status: "pending" | "done" = "done"

export let currentPage = 1

export let onOlderPage: (page: number) => void
export let olderDisabled = false
export let onNewerPage: (page: number) => void
export let newerDisabled = false
export let onCurrentClick: () => void

export let className = ""
export { className as class }
</script>

<Pagination.Root
  let:pages
  siblingCount={2000}
  perPage={rowsPerPage}
  count={totalTableRows}
  class={cn(className)}
>
  <Pagination.Content class="py-2 text-md uppercase font-supermolot mr-auto">
    <Pagination.Item>
      <Pagination.PrevButton
        disabled={status === 'pending' || newerDisabled}
        on:click={event => onNewerPage(Number(currentPage) - 1)}
        class={cn(status === 'pending' || newerDisabled ? 'cursor-not-allowed disabled' : '')}
      >
        <ChevronLeft class="size-6" />
        <span class="text-md uppercase font-supermolot">Newer</span>
      </Pagination.PrevButton>
    </Pagination.Item>
    <div class={cn('flex flex-col align-baseline gap-y-0.5')}>
      <div class={cn('flex flex-row uppercase')}>
        <Button
          size="sm"
          variant="default"
          on:click={onCurrentClick}
          title="Go to the first page"
          disabled={status === 'pending' || currentPage === 0}
          class={cn(
            'hover:bg-accent hover:text-black',
            status === 'pending' && 'cursor-not-allowed',
          )}
        >
          current
        </Button>
      </div>
    </div>
    <Pagination.Item>
      <Pagination.NextButton
        disabled={status === 'pending' || olderDisabled}
        on:click={event => onOlderPage(Number(currentPage) + 1)}
        class={cn(status === 'pending' || olderDisabled ? 'cursor-not-allowed' : '')}
      >
        <span class="text-md uppercase font-supermolot">Older</span>
        <ChevronRight class="size-6" />
      </Pagination.NextButton>
    </Pagination.Item>
  </Pagination.Content>
</Pagination.Root>
