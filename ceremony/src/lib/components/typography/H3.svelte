<script lang="ts">
let {children, class: CLASS, ...props} = $props()
</script>

<h3 class={`text-union-heading-primary text-2xl font-bold font-supermolot mb-3 ${CLASS}` {...props}}>
  {@render children()}
</h3>
