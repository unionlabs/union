<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { Label } from "$lib/components/ui/label"
import ChevronLeft from "virtual:icons/lucide/chevron-left"
import Button from "$lib/components/ui/button/button.svelte"
import ChevronRight from "virtual:icons/lucide/chevron-right"
import * as Pagination from "$lib/components/ui/pagination/index.ts"
import { derived, type Readable } from "svelte/store"
import { toast } from "svelte-sonner"
import { encodeTimestampSearchParam, decodeTimestampSearchParam } from "$lib/timestamps"
import { goto } from "$app/navigation"
import { page } from "$app/stores"

let timestamp: Readable<string | null> = derived(page, $page => {
  const urlTimestamp = $page.url.searchParams.get("timestamp")
  if (!urlTimestamp) return null
  return decodeTimestampSearchParam(urlTimestamp)
})
export let explorerItems: Readable<Array<{ timestamp: any }>>

$: live = $timestamp === null
const onOlderPage = () => {
  const stamp = $explorerItems.at(-1)?.timestamp

  if (!stamp) {
    toast.error("Invalid older timestamp")
    return
  }

  // timestamp.set(stamp)
  goto(encodeTimestampSearchParam(stamp), {
    replaceState: true,
    state: { timestamp: stamp }
  })
}

const onNewerPage = () => {
  const stamp = $explorerItems.at(0)?.timestamp

  if (!stamp) {
    toast.error("Invalid newer timestamp")
    return
  }

  // timestamp.set(stamp)
  goto(encodeTimestampSearchParam(stamp), {
    replaceState: true,
    state: { timestamp: stamp }
  })
}

const onCurrentClick = () => {
  goto($page.url.pathname, { replaceState: true })
}
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
        disabled={live}
        title={live ? "Already on the newest page" : "Go to the first page"}
        class="hover:bg-accent hover:text-black"
      >
        {live ? "live" : "current"}
      </Button>
    </div>
    {#if $timestamp}
      <Pagination.Item>
        <Pagination.PrevButton
          disabled={status === "pending" || live}
          title={live ? "Already on the newest page" : "Go to the previous page"}
          on:click={onNewerPage}
          class={cn(
            "text-sm",
            status === "pending" 
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
          {$timestamp}
        </time>
      </Label>
    {/if}
    <Pagination.Item>
      <Pagination.NextButton
        disabled={status === "pending"}
        on:click={onOlderPage}
        class="pr-0"
      >
        <span class="text-sm uppercase font-supermolot">Older</span>
        <ChevronRight class="size-6" />
      </Pagination.NextButton>
    </Pagination.Item>
  </Pagination.Content>
</Pagination.Root>
