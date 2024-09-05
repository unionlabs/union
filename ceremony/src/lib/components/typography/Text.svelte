<script lang="ts">
let { children, class: className = "", ...props } = $props()
</script>

<p class={`text-union-text-primary leading-6 font-mono ${className}`} {...props}>
  {@render children()}
</p>
