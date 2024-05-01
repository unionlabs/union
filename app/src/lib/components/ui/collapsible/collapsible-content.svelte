<script lang="ts">
import { Collapsible as CollapsiblePrimitive } from "bits-ui"
import { slide } from "svelte/transition"

type $$Props = CollapsiblePrimitive.ContentProps

export let transition: $$Props["transition"] = slide
export let transitionConfig: $$Props["transitionConfig"] = {
  duration: 150
}
</script>

<CollapsiblePrimitive.Content {transition} {transitionConfig} {...$$restProps}>
	<slot />
</CollapsiblePrimitive.Content>
