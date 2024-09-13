<script lang="ts">
import { user } from "$lib/stores/user.svelte.ts"
import H1 from "$lib/components/typography/H1.svelte"
import { ContributorState } from "$lib/stores/state.svelte.ts"
import Ceremony from "$lib/components/Ceremony.svelte"

let contributor: ContributorState = new ContributorState()

$effect(() => {
  const userId = user.session?.user.id
  if (userId) contributor.setUserId(userId)
})
</script>

{#if contributor}
  {#if contributor.loggedIn}

    <!--HANDLE INVITE CODE -->
    <!--HANDLE WAITLIST -->
    <Ceremony {contributor}/>

  {:else}
    <H1>Welcome to union ceremony</H1>
  {/if}
{/if}