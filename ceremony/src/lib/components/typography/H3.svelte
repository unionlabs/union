<script lang="ts">
let { children, class: className = "", ...props } = $props()
</script>
<h3 class={`text-union-heading-primary text-2xl font-bold font-supermolot mb-3 ${className}`} {...props}>
  {@render children()}
</h3>
