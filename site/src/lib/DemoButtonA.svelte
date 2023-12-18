<!-- svelte-ignore a11y-missing-attribute -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<a
  class="px-4 py-2 border-2 border-accent text-accent font-jetbrains"
  on:click
  aria-label="Demo Button"
  on:focus
  on:submit
  {...$$restProps}
>
  <slot />
</a>
