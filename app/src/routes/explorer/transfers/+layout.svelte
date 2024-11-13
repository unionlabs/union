<script lang="ts">
import "temporal-polyfill/global"
console.info("transfers layout")

let { children } = $props()
</script>

{@render children?.()}
