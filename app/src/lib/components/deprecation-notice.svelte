<script lang="ts">
import * as Card from "$lib/components/ui/card/index.ts"
</script>

<main class="flex flex-col gap-6 items-center overflow-y-scroll max-h-full py-6 px-3 sm:px-6 w-full dark:bg-muted">
  <Card.Root class="w-full max-w-3xl text-center">
    <Card.Header>
      <Card.Title>Thank you for testing Union App V1!</Card.Title>
    </Card.Header> 
    <Card.Content class="flex flex-col gap-4">

        <video
          id="glitch-video"
          class="w-full h-full"
          loop
          muted
          autoplay
          playsinline
          data-video="glitch"
          on:canplay={function() {
            this.autoplay = true
          }}
          on:loadeddata={function() {
            this.autoplay = true
          }}
          on:loadedmetadata={function() {
            this.muted = true
          }}
        >
          <source src="https://pub-32dd1494f0fa423cb1013941269ecce9.r2.dev/app1-thankyou.webm" type="video/webm" />
        </video>
      <p>Thank you for testing Union! Thanks to your help, we processed <b>&gt56,600,108 transfers!</b> The testnet is temporarily paused for the V2 upgrade. Follow <a class="font-bold text-accent" href="https://x.com/union_build">@union_build</a> for updates.</p>
    </Card.Content>
  </Card.Root>
</main>
