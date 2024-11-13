<script lang="ts">
  import { createBubbler, passive } from 'svelte/legacy';

  const bubble = createBubbler();
import type { HTMLInputAttributes } from "svelte/elements"
import type { InputEvents } from "./index.js"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = HTMLInputAttributes
type $$Events = InputEvents



// Workaround for https://github.com/sveltejs/svelte/issues/9305

  interface Props {
    class?: $$Props["class"];
    value?: $$Props["value"];
    // Fixed in Svelte 5, but not backported to 4.x.
    readonly?: $$Props["readonly"];
    [key: string]: any
  }

  let { class: className = undefined, value = $bindable(undefined), readonly = undefined, ...rest }: Props = $props();
</script>

<input
  class={cn(
    'flex h-10 w-full border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
    className,
  )}
  bind:value
  {readonly}
  onblur={bubble('blur')}
  onchange={bubble('change')}
  onclick={bubble('click')}
  onfocus={bubble('focus')}
  onfocusin={bubble('focusin')}
  onfocusout={bubble('focusout')}
  onkeydown={bubble('keydown')}
  onkeypress={bubble('keypress')}
  onkeyup={bubble('keyup')}
  onmouseover={bubble('mouseover')}
  onmousemove={bubble('mousemove')}
  onmouseenter={bubble('mouseenter')}
  onmouseleave={bubble('mouseleave')}
  onpaste={bubble('paste')}
  oninput={bubble('input')}
  use:passive={['wheel', () => bubble('wheel')]}
  {...rest}
/>
