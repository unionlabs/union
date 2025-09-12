<script lang="ts">
import { matchOption } from "$lib/Snippets.svelte"
import type { Option } from "effect/Option"
type Props = {
  value: Option<unknown>
}
const { value }: Props = $props()
</script>

{#snippet onSome(a: unknown)}
  <div data-testid="Some">{a}</div>
{/snippet}

{#snippet onNone()}
  <div data-testid="None"></div>
{/snippet}

{@render matchOption(value, { onSome, onNone })}
