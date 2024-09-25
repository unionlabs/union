<script lang="ts">
  import Code from "$lib/components/Code.svelte"
  import {getNumberSuffix} from "$lib/utils/utils.ts"
  import {getState} from "$lib/state/index.svelte.ts";

  const {contributor, terminal} = getState()

</script>

{terminal.updateHistory("You're on the waitlist")}
{terminal.updateHistory(`When the ceremony opens to the public, you will be ${contributor.waitListPosition}${getNumberSuffix(contributor.waitListPosition)} in the public queue.`)}
{terminal.updateHistory("You will receive an email 12-18 hours before the public phase begins.")}
{terminal.updateHistory("Received an invite? You can skip the waitlist and join now.")}
<Code />
