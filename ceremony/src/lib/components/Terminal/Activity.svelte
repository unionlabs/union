<script lang="ts">
import Print from "$lib/components/Terminal/Print.svelte"
import {onDestroy, onMount} from "svelte";
import {Activity} from "$lib/state/activity.svelte.ts";

let activity: Activity | null = $state(null)

onMount(() => {
  activity = new Activity()
})

onDestroy(() => {
  if (activity) {
    activity = null;
  }
});

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

{#if activity && activity.data}
  <div class="flex flex-col-reverse">
    {#each activity.data as item, i (item)}
      {@const type = item.message.type}
      {@const user = item.message.user}
      <Print>
        {formatTimestamp(item.created_at)} -
        {#if type === "join_waitlist"}
          {user} joined the waitlist
        {:else if type === "redeem"}
          {user} has redeemed a code
        {:else if type === "join_queue"}
          {user} joined the queue
        {:else if type === "contribution_started"}
          {user} has started their contribution
        {:else if type === "contribution_submitted"}
          {user} has submitted their contribution
        {:else if type === "contribution_verified"}
          {user}'s contribution has just been verified
        {/if}
      </Print>
    {/each}
  </div>
{/if}