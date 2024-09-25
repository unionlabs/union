<script lang="ts">
import Code from "$lib/components/Code.svelte"
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import { getNumberSuffix } from "$lib/utils/utils.ts"
import Print from "$lib/components/TerminalApp/Print.svelte";

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()
</script>

<Print class="mb-4">You're on the waitlist </Print>
<Print class="mb-2">When the ceremony opens to the public, you will be <span class="text-union-accent-500">{contributor.waitListPosition}{getNumberSuffix(contributor.waitListPosition)}</span> in the public queue.</Print>
<Print class="mb-12">You will receive an email 12-18 hours before the public phase begins.</Print>
<Print class="mb-4">Received an invite?</Print>
<Print class="mb-4">You can skip the waitlist and join now.</Print>
<form class="flex flex-col items-center">
  <Code {contributor} secondary={true}/>
</form>