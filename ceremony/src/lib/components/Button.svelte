<script lang="ts">
let { children, class: className = "", ...props } = $props()
</script>

<button {...props} class={`flex items-center gap-2 px-4 py-2 text-black font-bold hover:text-black bg-union-accent-500 uppercase ${className}`}>
  {@render children()}
</button>