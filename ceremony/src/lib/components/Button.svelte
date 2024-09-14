<script lang="ts">
  import Spinner from "$lib/components/Spinner.svelte";

  let {
    children,
    class: className = "",
    loading = false,
    ...props
  } = $props()
</script>

<button
        {...props}
        class={`flex items-center gap-2 px-4 py-2 text-black font-bold hover:text-black bg-union-accent-500 uppercase ${className}`}>
  {@render children()}
  {#if loading}
    <Spinner class="size-4"/>
  {/if}
</button>