<script lang="ts">
  let {children, class: CLASS, ...props} = $props()
</script>

<h2 class={`text-union-heading-primary text-3xl font-bold font-supermolot mb-3.5 ${CLASS}`} {...props}>
  {@render children()}
</h2>
