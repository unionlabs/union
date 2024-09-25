<script lang="ts">
import Tweet from "$lib/components/Tweet.svelte"
import Print from "$lib/components/TerminalApp/Print.svelte";


</script>
<div class="flex flex-col justify-center items-center gap-4 text-center">
  <Print>Thank you!</Print>
  <Print>Your contribution is complete. Thank you for securing the Union network. <br> Tweet your cryptographic attestation for extra transparency:</Print>
  <Tweet/>
  <a href="/contributions" class="underline underline-offset-4 decoration-union-accent-500 text-white uppercase">View contributions</a>
</div>