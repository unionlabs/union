<script lang="ts">
import { type IconName, UnionIcons, type IconVariant } from "./union-icons.ts"

let className: string | undefined = undefined
export let name: IconName | undefined
export let theme: IconVariant = "color"
export { className as class }

let component: any

function getIconComponent(name: IconName | undefined, theme: IconVariant): any | undefined {
  return name && UnionIcons[name]?.variants[theme]
}

$: component = getIconComponent(name, theme)
</script>

{#if component}
  <svelte:component this={component} class={className} />
{:else}
  <p>Icon not found or not available in this variant.</p>
{/if}
