<script lang="ts">
import type { LayoutData } from "./$types.ts"
import DevTools from "$lib/components/dev-tools.svelte"

export let data: LayoutData

const [addressOne, addressTwo] = data.addressArray
</script>

<DevTools>
</DevTools>

<slot />

<style lang="postcss"></style>
