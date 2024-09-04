<script lang="ts">
  let { children, class: CLASS, ...props} = $props()
</script>

<h5 class={`text-union-heading-primary text-lg font-bold font-supermolot ${CLASS}`} {...props}>
  {@render children()}
</h5>
