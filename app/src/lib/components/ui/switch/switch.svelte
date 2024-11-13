<script lang="ts">
import { Switch as SwitchPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = SwitchPrimitive.Props
type $$Events = SwitchPrimitive.Events

  interface Props {
    class?: $$Props["class"];
    checked?: $$Props["checked"];
    [key: string]: any
  }

  let { class: className = undefined, checked = $bindable(undefined), ...rest }: Props = $props();

</script>

<SwitchPrimitive.Root
  bind:checked
  class={cn(
    'peer inline-flex h-[24px] w-[44px] shrink-0 cursor-pointer items-center  border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=unchecked]:bg-primary',
    className,
  )}
  {...rest}
  on:click
  on:keydown
>
  <SwitchPrimitive.Thumb
    class={cn(
      'pointer-events-none block h-5 w-5 data-[state=unchecked]:bg-muted data-[state=unchecked]:dark:bg-muted-foreground shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=checked]:bg-accent data-[state=unchecked]:translate-x-0',
    )}
  />
</SwitchPrimitive.Root>
