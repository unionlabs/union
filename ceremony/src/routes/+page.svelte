<script lang="ts">
import { user } from "$lib/stores/user.svelte.ts"
import H1 from "$lib/components/typography/H1.svelte"
import { ContributorState } from "$lib/stores/state.svelte.ts"
import Ceremony from "$lib/components/Ceremony.svelte"
import Join from "$lib/components/Join.svelte"

let contributor: ContributorState = new ContributorState()

$effect(() => {
  const userId = user.session?.user.id
  if (userId) contributor.setUserId(userId)
})
</script>

<!--Todo handle when to not show ceremony component-->

{#if contributor}
  {#if contributor.loggedIn}
    <Ceremony {contributor}/>

  {:else if contributor.onWaitlist}
    <H1>Your on the list</H1>
  {:else}

    <!--Do this if no code and no waitlist?-->
    <Join />
  {/if}
{:else}
  <H1>Welcome to union ceremony</H1>
{/if}
