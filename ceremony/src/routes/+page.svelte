<script lang="ts">
import TerminalWindow from "$lib/components/TerminalApp/TerminalWindow.svelte"
import Authenticate from "$lib/components/TerminalApp/Authenticate.svelte"
import { getState } from "$lib/state/index.svelte.ts"
import Print from "$lib/components/TerminalApp/Print.svelte"

const { user, terminal } = getState()
</script>

<TerminalWindow>
  {terminal.updateHistory("welcome to union ceremony")}
  {#if user.session}
    {terminal.updateHistory(`authenticated user: ${user.session.user.email}`)}
  {:else if user.session === null}
    <Authenticate {terminal}/>
  {:else}
    <Print>Loading...</Print>
  {/if}
</TerminalWindow>