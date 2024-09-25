<script lang="ts">
import { ContributorState, getContributorState } from "$lib/stores/state.svelte.ts"
import Spinner from "$lib/components/Spinner.svelte"
import Ceremony from "$lib/components/Ceremony.svelte"
import Join from "$lib/components/Join.svelte"
import Waitlist from "$lib/components/Waitlist.svelte"
import Authenticate from "$lib/components/TerminalApp/Authenticate.svelte"
import TerminalWindow from "$lib/components/TerminalApp/TerminalWindow.svelte"
import Print from "$lib/components/TerminalApp/Print.svelte"

const contributor: ContributorState = getContributorState()
</script>

<!--<section class="w-full h-svh flex flex-col justify-center items-center">-->
<!--  {#if contributor.loggedIn}-->
<!--    {#if !contributor.currentUserState}-->
<!--      <Spinner class="text-union-accent-500 size-6"/>-->
<!--    {:else if contributor.currentUserState === "hasRedeemed" || contributor.currentUserState === "inQueue"}-->
<!--      <Ceremony {contributor}/>-->
<!--    {:else if contributor.currentUserState === "inWaitlist"}-->
<!--      <Waitlist {contributor}/>-->
<!--    {:else if contributor.currentUserState === "join"}-->
<!--      <Join {contributor}/>-->
<!--    {/if}-->
<!--  {/if}-->
<!--</section>-->


<TerminalWindow>
  <Print>Hello</Print>
</TerminalWindow>