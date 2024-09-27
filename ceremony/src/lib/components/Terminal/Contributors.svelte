<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import { goto } from "$app/navigation"
import { onMount } from "svelte"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"
import type { Contributions } from "$lib/state/contributions.svelte.ts"
import type { Terminal } from "$lib/state/terminal.svelte.ts"

type State = {
  contributions: Contributions
  terminal: Terminal
}

const { contributions, terminal }: State = getState()

let focusedIndex = $state(0)
let buttons: Array<HTMLButtonElement> = []
let data = $state([])
onMount(() => {
  contributions.data.map(contribution => {
    data.push({ text: contribution.payload_id, action: contribution.public_key_hash })
  })
})

function trigger(value: any) {
  goto(`/contributions/${value}`)
  terminal.setTab(4)
  terminal.setHash(value)
}
</script>
<Buttons
        {data}
        trigger={(value) => trigger(value)}
/>
