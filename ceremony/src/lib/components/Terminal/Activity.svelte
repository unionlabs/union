<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import Print from "$lib/components/Terminal/Print.svelte"

const { activity } = getState()

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
  return `${time} | ${datePart}`
}
</script>

{#if activity.data}
  <Print>ceremony activity</Print>
  {#each activity.data as item, i (item)}
    {@const type = item.message.type}
    {@const user = item.message.user}
    <Print>
      {formatTimestamp(item.created_at)} -
      {#if type === "join_waitlist"}
        {user} joined the waitlist
      {:else if type === "redeem"}
        {user} have redeemed a code
      {:else if type === "join_queue"}
        {user} joined the queue
      {:else if type === "contribution_started"}
        {user} have started their contribution
      {:else if type === "contribution_submitted"}
        {user} has submitted their contribution
      {:else if type === "contribution_verified"}
        {user} contribution just verified
      {/if}
    </Print>

  {/each}
{/if}