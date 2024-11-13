<script lang="ts">
import { Button as ButtonPrimitive } from "bits-ui"
import { type Events, type Props, buttonVariants } from "./index.js"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = Props
type $$Events = Events

  interface Props_1 {
    class?: $$Props["class"];
    value?: $$Props["value"];
    variant?: $$Props["variant"];
    size?: $$Props["size"];
    builders?: $$Props["builders"];
    name?: $$Props["name"];
    disabled?: $$Props["disabled"];
    children?: import('svelte').Snippet;
    [key: string]: any
  }

  let {
    class: className = undefined,
    value = "",
    variant = "default",
    size = "default",
    builders = [],
    name = "button",
    disabled = false,
    children,
    ...rest
  }: Props_1 = $props();

</script>

<ButtonPrimitive.Root
  {name}
  {builders}
  {disabled}
  class={cn(buttonVariants({ variant, size, className }))}
  type="button"
  {...rest}
  on:click
  on:keydown
>
  {#if children}{@render children()}{:else}
    {value}
  {/if}
</ButtonPrimitive.Root>
