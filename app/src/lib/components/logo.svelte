<script lang="ts">
let className: string | undefined = "size-12"
export { className as class }
</script>

<svg class={className} viewBox="0 0 256 256" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
  <path fill-rule="evenodd" clip-rule="evenodd" d="M168 160C168 195.346 139.346 224 104 224C68.6538 224 40 195.346 40 160H72C72 177.673 86.3269 192 104 192C121.673 192 136 177.673 136 160H168Z" fill="currentColor"/>
  <rect x="216" y="144" width="32" height="32" transform="rotate(-180 216 144)" fill="currentColor" class="square square-1"/>
  <rect x="136" y="112" width="32" height="32" fill="currentColor" class="square square-2"/>
  <rect x="120" y="144" width="32" height="32" transform="rotate(-180 120 144)" fill="currentColor" class="square square-3"/>
  <rect x="40" y="112" width="32" height="32" fill="currentColor" class="square square-4"/>
  <path fill-rule="evenodd" clip-rule="evenodd" d="M88 96C88 60.6538 116.654 32 152 32C187.346 32 216 60.6538 216 96L184 96C184 78.3269 169.673 64 152 64C134.327 64 120 78.3269 120 96L88 96Z" fill="currentColor"/>
</svg>

<style lang="postcss">
    @keyframes fadeInOut {
        0%, 100% { opacity: 0; }
        75% { opacity: 1; }
    }

    .square {
        opacity: 0;
        animation: fadeInOut 700ms infinite reverse;
    }

    .square-1 { animation-delay: 400ms; }
    .square-2 { animation-delay: 300ms; }
    .square-3 { animation-delay: 200ms; }
    .square-4 { animation-delay: 100ms; }
</style>
