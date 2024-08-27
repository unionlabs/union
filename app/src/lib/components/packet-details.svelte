<script lang="ts">
import { packetDetailsFragment } from "$lib/graphql/fragments/packets.ts"
import { readFragment, type FragmentOf } from "gql.tada"

export let packetDetails: FragmentOf<typeof packetDetailsFragment>
const packet = readFragment(packetDetailsFragment, packetDetails)
</script>  

<pre class="text-sm">{JSON.stringify(packet, null, 2)}</pre>

