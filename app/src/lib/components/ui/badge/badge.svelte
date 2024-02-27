<script lang="ts">
  import { cn } from '$lib/utilities/shadcn.ts'
  import { badgeVariants, type Variant } from '.'

  let className: string | undefined | null
  export let href: string | undefined
  export let variant: Variant = 'default'
  export { className as class }
</script>

<svelte:element
  this={href ? 'a' : 'span'}
  {href}
  class={cn(badgeVariants({ variant, className }))}
  {...$$restProps}
>
  <slot />
</svelte:element>
