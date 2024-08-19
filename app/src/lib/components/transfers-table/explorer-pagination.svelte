<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { Label } from "$lib/components/ui/label"
import ChevronLeft from "virtual:icons/lucide/chevron-left"
import Button from "$lib/components/ui/button/button.svelte"
import ChevronRight from "virtual:icons/lucide/chevron-right"
import { derived, type Readable } from "svelte/store"
import { toast } from "svelte-sonner"
import { encodeTimestampSearchParam, decodeTimestampSearchParam } from "$lib/timestamps"
import { goto } from "$app/navigation"
import { page } from "$app/stores"

export let explorerItems: Readable<Array<{ timestamp: any }>>

let timestamp: Readable<string | null> = derived(page, $page => {
  const urlTimestamp = $page.url.searchParams.get("timestamp")
  if (!urlTimestamp) return null
  return decodeTimestampSearchParam(urlTimestamp)
})
$: live = $timestamp === null

const onOlderPage = () => {
  const stamp = $explorerItems.at(-1)?.timestamp
  if (!stamp) return toast.error("Invalid older timestamp")

  goto(encodeTimestampSearchParam(stamp), {
    replaceState: true,
    state: { timestamp: stamp }
  })
}

const onNewerPage = () => {
  const stamp = $explorerItems.at(0)?.timestamp
  if (!stamp) return toast.error("Invalid newer timestamp")

  goto(encodeTimestampSearchParam(stamp), {
    replaceState: true,
    state: { timestamp: stamp }
  })
}

const onCurrentClick = () => {
  goto($page.url.pathname, { replaceState: true })
}
</script>

<div class="flex items-center gap-4 mt-4">
  <Button
    size="sm"
    variant="default"
    on:click={onCurrentClick}
    disabled={live}
    title={live ? "Already on the newest page" : "Go to the first page"}
  >
    {live ? "live" : "current"}
  </Button>
  {#if $timestamp}
    <Button
      size="sm"
      variant="secondary"
      title={live ? "Already on the newest page" : "Go to the previous page"}
      on:click={onNewerPage}
      class="pl-2"
    >
      <ChevronLeft class="size-6" />
      Newer
    </Button>
    <Label class="w-full">
      <time class="font-normal text-sm uppercase font-mono w-full">
        {$timestamp}
      </time>
    </Label>
  {/if}
  <Button
    size="sm"
    variant="secondary"
    on:click={onOlderPage}
    class="pr-2"
  >
    Older
    <ChevronRight class="size-6" />
  </Button>
</div>
