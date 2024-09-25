<script lang="ts">
  import Authenticate from "$lib/components/TerminalApp/Authenticate.svelte"
  import {getState} from "$lib/state/index.svelte.ts"
  import Print from "$lib/components/TerminalApp/Print.svelte"
  import {onMount} from "svelte";
  import Ceremony from "$lib/components/TerminalApp/Ceremony.svelte";
  import Join from "$lib/components/TerminalApp/Join.svelte";
  import Waitlist from "$lib/components/TerminalApp/Waitlist.svelte";

  const {user, terminal, contributor} = getState()

  onMount(() => {
    terminal.setTab(1)
  })
</script>

{terminal.updateHistory("Welcome to union ceremony")}
{#if user.loading}
  <Print>loading...</Print>
{:else}
  {#if user.session}
    {terminal.updateHistory(`Authenticated user: ${user.session.user.email}`)}
    {#if contributor.currentUserState === "hasRedeemed" || contributor.currentUserState === "inQueue"}
      <Ceremony/>
    {:else if contributor.currentUserState === "inWaitlist"}
      <Waitlist />
    {:else if contributor.currentUserState === "join"}
      <Join/>
    {/if}

  {:else if user.session === null && terminal.tab === 1}
    <Authenticate {terminal} />
  {/if}
{/if}
