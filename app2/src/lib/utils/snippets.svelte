<script lang="ts" module>
import type { Option } from "effect" 
import type { Snippet } from "svelte";
export { mapOption, matchOption, empty }
</script>

{#snippet empty()}{/snippet}

{#snippet mapOption<T>(self: Option.Option<T>, onSome: Snippet<[T]>)}
  {#if self._tag === "Some"}
    {@render onSome(self.value)}
  {/if}
{/snippet}

{#snippet matchOption<T>(self: Option.Option<T>, onSome: Snippet<[T]>, onNone?: Snippet<[]> | undefined)}
  {#if self._tag === "Some"}
    {@render onSome(self.value)}
  {:else}
    {@render (onNone ?? empty)()}
  {/if}
{/snippet}

