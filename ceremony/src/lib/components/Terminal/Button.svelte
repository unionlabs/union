<script lang="ts">
let { children, class: className = "", value = $bindable(), ...rest } = $props()
</script>

<button
        bind:this={value}
        class="block outline-none focus:ring-transparent focus:border-none hover:text-union-accent-500 focus:text-union-accent-500 text-sm sm:text-base {className}"
        {...rest}>
  {@render children()}
</button>