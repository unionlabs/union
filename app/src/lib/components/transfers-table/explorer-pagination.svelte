<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { Label } from "$lib/components/ui/label"
import ChevronLeft from "virtual:icons/lucide/chevron-left"
import Button from "$lib/components/ui/button/button.svelte"
import ChevronRight from "virtual:icons/lucide/chevron-right"
import * as Pagination from "$lib/components/ui/pagination/index.ts"

export let timestamp: string
export let status: "pending" | "done" = "done"

export let currentPage = 1

export let live: boolean
export let onOlderPage: (page: number) => void
export let olderDisabled = false
export let onNewerPage: (page: number) => void
export let newerDisabled = false

export let onCurrentClick: () => void
</script>

<Pagination.Root
  let:pages
  siblingCount={2000}
  perPage={20}
  class="w-auto"
  count={20}
>
  <Pagination.Content
    class="py-2 text-sm uppercase font-supermolot w-full flex gap-x-1"
  >
    <div class={cn("flex flex-row uppercase")}>
      <Button
        size="sm"
        variant="default"
        on:click={(event) => {
          onCurrentClick()
        }}
        disabled={status === "pending" || live}
        title={live ? "Already on the newest page" : "Go to the first page"}
        class={cn(
          "hover:bg-accent hover:text-black",
          status === "pending" && "cursor-not-allowed"
        )}
      >
        {live ? "live" : "current"}
      </Button>
    </div>
    <Pagination.Item>
      <Pagination.PrevButton
        disabled={status === "pending" || newerDisabled || live}
        title={live ? "Already on the newest page" : "Go to the previous page"}
        on:click={(event) => {
          event.preventDefault()
          event.stopPropagation()
          onNewerPage(Number(currentPage) - 1)
        }}
        class={cn(
          "text-sm",
          status === "pending" || newerDisabled
            ? "cursor-not-allowed disabled"
            : ""
        )}
      >
        <ChevronLeft class="size-6" />
        <span class="text-sm uppercase font-supermolot">
          {live ? "newest" : "Newer"}
        </span>
      </Pagination.PrevButton>
    </Pagination.Item>
    <Label class="w-full">
      <time class="font-normal text-sm uppercase font-mono my-auto w-full">
        {timestamp}
      </time>
    </Label>
    <Pagination.Item>
      <Pagination.NextButton
        disabled={status === "pending" || olderDisabled}
        on:click={(event) => {
          onOlderPage(Number(currentPage) + 1)
        }}
        class={cn(
          "pr-0",
          status === "pending" || olderDisabled ? "cursor-not-allowed" : ""
        )}
      >
        <span class="text-sm uppercase font-supermolot">Older</span>
        <ChevronRight class="size-6" />
      </Pagination.NextButton>
    </Pagination.Item>
  </Pagination.Content>
</Pagination.Root>
