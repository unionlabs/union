<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import { onMount } from "svelte"
import Print from "$lib/components/Terminal/Print.svelte"
import Ceremony from "$lib/components/Terminal/Ceremony.svelte"
import Waitlist from "$lib/components/Terminal/Waitlist.svelte"
import Join from "$lib/components/Terminal/Join.svelte"
import Authenticate from "$lib/components/Terminal/Authenticate.svelte"

const { user, terminal, contributor } = getState()

onMount(() => {
  terminal.setTab(1)
})
</script>

{#if user.loading}
  <Print>loading...</Print>
{:else}
  {#if user.session}
    {#if contributor.currentUserState === "hasRedeemed" || contributor.currentUserState === "inQueue"}
      <Ceremony/>
    {:else if contributor.currentUserState === "inWaitlist"}
      <Waitlist/>
    {:else if contributor.currentUserState === "join"}
      <Join/>
    {/if}
  {:else if user.session === null && terminal.tab === 1}
    <Authenticate />
  {/if}
{/if}
