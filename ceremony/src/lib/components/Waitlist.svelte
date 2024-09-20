<script lang="ts">
import H4 from "$lib/components/typography/H4.svelte"
import Code from "$lib/components/Code.svelte"
import Text from "$lib/components/typography/Text.svelte"
import H1 from "$lib/components/typography/H1.svelte"
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import { getNumberSuffix } from "$lib/utils/utils.ts"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()
</script>

<H1 class="mb-4">You're on the waitlist </H1>
<Text class="mb-2">When the ceremony opens to the public, you will be <span class="text-union-accent-500">{contributor.waitListPosition}{getNumberSuffix(contributor.waitListPosition)}</span> in the public queue.</Text>
<Text class="mb-12">You will receive an email 12-18 hours before the public phase begins.</Text>
<H4 class="mb-4">Received an invite?</H4>
<Text class="mb-4">You can skip the waitlist and join now.</Text>
<form class="flex flex-col items-center">
  <Code {contributor} secondary={true}/>
</form>