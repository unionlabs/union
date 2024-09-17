<script lang="ts">
import { user } from "$lib/stores/user.svelte.ts"
import H1 from "$lib/components/typography/H1.svelte"
import { ContributorState } from "$lib/stores/state.svelte.ts"
import Ceremony from "$lib/components/Ceremony.svelte"
import Join from "$lib/components/Join.svelte"
import Spinner from "$lib/components/Spinner.svelte"
import Blink from "$lib/components/Blink.svelte"
import H2 from "$lib/components/typography/H2.svelte"

let contributor: ContributorState = new ContributorState()

$effect(() => {
  const userId = user.session?.user.id
  if (userId) contributor.setUserId(userId)
})
</script>

{#if contributor.loggedIn}
  {#if !contributor.allowanceState}
    <Spinner class="text-union-accent-500 size-6"/>
  {:else if contributor.allowanceState === "hasRedeemed" || contributor.allowanceState === "inQueue"}
    <Ceremony {contributor}/>
  {:else if contributor.allowanceState === "inWaitlist"}
    <H1>You're on the list</H1>
  {:else if contributor.allowanceState === "join"}
    <Join {contributor}/>
  {/if}
{:else}
  <H2>
    <Blink/>
  </H2>
  <H1>Welcome to union ceremony</H1>
{/if}

