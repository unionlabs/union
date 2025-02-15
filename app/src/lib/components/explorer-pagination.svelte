<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { Label } from "$lib/components/ui/label"
import ChevronLeft from "virtual:icons/lucide/chevron-left"
import Button from "$lib/components/ui/button/button.svelte"
import ChevronRight from "virtual:icons/lucide/chevron-right"
import { derived, type Readable } from "svelte/store"
import { toPrettyDateTimeFormat } from "$lib/utilities/date.ts"
import { toast } from "svelte-sonner"
import { encodeTimestampSearchParam, decodeTimestampSearchParam } from "$lib/timestamps"
import { goto } from "$app/navigation"
import { page } from "$app/stores"
import { timestamp } from "$lib/stores/page.ts"

export let explorerItems: Readable<Array<{ timestamp: any }>>
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
function convertUTCToLocal(timestampStr: string): string {
  // Parse the timestamp string into a UTC Date object
  const [datePart, timePart] = timestampStr.split(" ")
  const [year, month, day] = datePart.split("-").map(Number)
  const [hour, minute, second] = timePart.split(":").map(Number)

  const utcDate = new Date(Date.UTC(year, month - 1, day, hour, minute, second))

  // Convert to local timezone
  const localDate = new Date(utcDate.getTime()) // Automatically adjusts to local time

  // Format to "YYYY-MM-DD HH:MM:SS"
  const formattedDate = `${localDate.getFullYear()}-${String(localDate.getMonth() + 1).padStart(2, "0")}-${String(localDate.getDate()).padStart(2, "0")} ${String(localDate.getHours()).padStart(2, "0")}:${String(localDate.getMinutes()).padStart(2, "0")}:${String(localDate.getSeconds()).padStart(2, "0")}`

  return formattedDate
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
    <time class="font-normal text-sm font-mono">{convertUTCToLocal($timestamp)}</time>
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
