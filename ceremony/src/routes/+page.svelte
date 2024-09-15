<script lang="ts">
import { user } from "$lib/stores/user.svelte.ts"
import H1 from "$lib/components/typography/H1.svelte"
import { ContributorState } from "$lib/stores/state.svelte.ts"
import Ceremony from "$lib/components/Ceremony.svelte"
import Join from "$lib/components/Join.svelte"
import Spinner from "$lib/components/Spinner.svelte"
import Counter from "$lib/components/Counter/index.svelte"

let contributor: ContributorState = new ContributorState()

$effect.pre(() => {
  const userId = user.session?.user.id
  if (userId) contributor.setUserId(userId)
})

const targetTimestamp = 1726812000
</script>

<!--Fix jump between state on load-->
{#if contributor.loggedIn}
  <!--{#if !contributor.allowanceState}-->
  <!--  <Spinner class="text-union-accent-500 size-6"/>-->
  <!--{:else if contributor.allowanceState === "invited"}-->
  <!--  <Ceremony {contributor}/>-->
  <!--{:else if contributor.allowanceState === "waitingList"}-->
  <!--  <H1>You're on the list</H1>-->
  <!--{:else if contributor.allowanceState === "join"}-->
  <!--  <Join {contributor}/>-->
  <!--{/if}-->
{:else}
  <Counter {targetTimestamp}/>
{/if}

