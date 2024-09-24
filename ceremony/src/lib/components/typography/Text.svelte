<script lang="ts">
let { children, class: className = "", ...props } = $props()
</script>

<p class={`text-union-text-primary font-mono leading-7 ${className}`} {...props}>
  {@render children()}
</p>
