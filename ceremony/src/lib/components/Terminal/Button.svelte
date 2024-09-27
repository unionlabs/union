<script lang="ts">
let { children, class: className = "", value = $bindable(), ...rest } = $props()
</script>

<button
        bind:this={value}
        class="px-2 focus:blink block outline-none focus:ring-transparent focus:border-none text-sm sm:text-base {className}"
        {...rest}>
  {@render children()}
</button>

<style>

</style>