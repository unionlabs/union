<script lang="ts">
import { getLiveLogsState } from "$lib/stores/live.svelte.ts"
import Text from "$lib/components/typography/Text.svelte"
import H2 from "$lib/components/typography/H2.svelte"

const liveLogs = getLiveLogsState()

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  const time = date.toLocaleString("en-GB", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false
  })
  const datePart = date.toLocaleString("en-GB", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric"
  })
  return `${time} - ${datePart}`
}
</script>

<div class="w-full flex p-4 items-center justify-center flex-col">
  <H2 class="mb-2">Activity log</H2>
  <div class="flex flex-col px-5 py-3 border w-full max-h-[400px] overflow-hidden overflow-y-auto max-w-xl gap-2">
    {#each liveLogs.data as item, i (item)}
      {@const type = item.message.original.type}
        <div class="flex flex-col">
          <Text>
            <span class="opacity-70 text-xs">{formatTimestamp(item.created_at)}</span><br>
            <span class="!text-union-accent-500">&gt</span>
            {#if type === "join_waitlist"}
              Someone joined the waitlist

            {:else if type === "redeem"}
              A user have redeemed a code

            {:else if type === "join_queue"}
              Someone joined the queue

            {:else if type === "contribution_started"}
              A user have started their contribution

            {:else if type === "contribution_submitted"}
              A user has submitted their contribution

            {:else if type === "contribution_verified"}
              A contribution just verified

            {/if}
          </Text>
        </div>

    {/each}
  </div>
</div>