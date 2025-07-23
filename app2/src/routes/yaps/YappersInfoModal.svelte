<script lang="ts">
import Modal from "$lib/components/ui/Modal.svelte"

interface Props {
  isOpen: boolean
  onClose: () => void
}

const { isOpen, onClose }: Props = $props()
</script>

<Modal
  {isOpen}
  {onClose}
  class="w-full max-w-2xl h-[80vh] max-h-[600px] flex flex-col"
>
  <div class="overflow-y-auto p-6 space-y-6">
    <h2 class="text-2xl font-bold text-white">Welcome to Mad Yaps: Ride Into the Yapocalypse</h2>

    <div class="space-y-6 text-zinc-300 leading-relaxed">
      <div>
        <p class="mb-4 text-orange-200">
          The chains shattered. Bridges turned to bone. Giants sank beneath ash, clutching the last
          shards of liquidity as the old world crumbled into silence.
        </p>
        <p class="mb-4">
          From this ruin, a faction clawed its way back into the light: The Union. Forged in
          bytefire by the Tech Syndicate and sanctified by the House of Holy Buildoors. They did not
          come to save the world. They came to rebuild it, trustless and wild.
        </p>
        <p class="mb-4">
          Now, the call goes out: On July 24, the Yapocalypse begins.
        </p>
        <p class="mb-4">
          The Yap Boys, feral riders of the feed, take to the sands, howling gospel in the name of
          coordination. Their mission: recruit, restore, unite. A fresh leaderboard rises from the
          wreckage, pointing the way to Yaphalla.
        </p>
        <p class="mb-4">
          One truth remains: this is your last opportunity to bind yourself to the Dashboard sigil
          before the Yapocalypse begins—or be lost to the void. The top yappers shall ascend beyond
          myth. The unspeaking will vanish beneath the dust.
        </p>
        <p class="mb-4">
          The new world draws near. The wastelands howl for rebirth.
        </p>
        <p class="mb-4 text-orange-400 font-semibold">
          In an insane world, Union is the only sane option.
        </p>
        <p class="text-white font-bold">
          Are you up for the challenge?
        </p>
      </div>

      <div>
        <h3 class="text-lg font-semibold text-white mb-3">What is Mad Yaps?</h3>
        <p class="mb-4">
          Mad Yaps is the Union's Kaito rewards program, consisting of two Seasons (for now).
        </p>
        <p class="mb-4">
          <strong class="text-white">Season 0</strong> ended today. The timeframe for Season 0 is
          from Union's inception (October 2023) to today (July 24, 2025). A screenshot of the
          leaderboard has been taken which can be viewed in the Union Dashboard.
        </p>
        <p class="mb-4">
          <strong class="text-white">Season 1</strong> begins now. It's open to newcomers and OGs
          alike, to accounts big and small—offering a final chance to make a splash in Union's
          journey to mainnet. This leaderboard serves two purposes:
        </p>
        <ul class="list-disc list-inside mb-4 ml-4">
          <li>Gives smaller accounts another chance to climb.</li>
          <li>Helps latecomers kick off their Union journey with impact.</li>
        </ul>
        <p>
          Both seasons have separate reward pools. Season 0 yappers will be eligible for both
          leaderboards if you continue yapping. If you haven't yapped already, now's the time.
        </p>
      </div>

      <div>
        <h3 class="text-lg font-semibold text-white mb-3">Reward Distribution</h3>
        <p class="mb-4">
          Union is officially allocating <strong class="text-orange-400"
          >1% of its token supply</strong> to the Kaito rewards program. This initiative is meant to
          reward Union's early believers, active yappers, and community ambassadors for both their
          past contributions and their continued efforts to make Union a household name as a
          cross-chain Layer-1.
        </p>

        <h4 class="text-md font-semibold text-white mb-2">Rewards Breakdown</h4>
        <p class="mb-3">
          <strong class="text-white">Mad Yaps (0.75%):</strong>
        </p>
        <ul class="list-disc list-inside mb-4 ml-4">
          <li>
            <strong>Season 0:</strong> Honors Union's original commitment of $1M+ in rewards
            announced previously. It will be distributed based on all-time Union mindshare up to
            today.
          </li>
          <li>
            <strong>Season 1:</strong> The new season starts today, July 24 and runs through public
            mainnet, rewarding yappers based on their mindshare during this timeframe.
          </li>
        </ul>
        <p class="mb-4">
          <strong class="text-white">Kaito Ecosystem (0.25%):</strong> Directed to Kaito stakers,
          genesis NFT holders, and the broader Kaito community. This allocation is consistent with
          reward structures from similar projects participating in Kaito Earn campaigns.
        </p>
      </div>

      <div>
        <h3 class="text-lg font-semibold text-white mb-3">Things To Note</h3>
        <ul class="list-disc list-inside mb-4 ml-4 space-y-2">
          <li>
            10% of the rewards in Season 0 and 1 (0.10% of the token supply) are reserved for native
            speakers of China and Korea.
          </li>
          <li>
            Union core team members will not receive rewards from any of the allocations outlined
            above. Their mindshares will be added to the Season 1 pool to give community an extra
            boost on this final sprint.
          </li>
          <li>
            Season 1 mindshare data will be updated periodically as full reports are received from
            Kaito, approximately every 2–3 days.
          </li>
        </ul>
      </div>

      <div>
        <h3 class="text-lg font-semibold text-white mb-3">Join the Madness</h3>
        <p class="mb-4">
          Ready to yap your way into Union history? Here's how to get started:
        </p>
        <ol class="list-decimal list-inside mb-4 ml-4 space-y-2">
          <li>
            <strong class="text-white">Sign up on the Union Dashboard:</strong> If you haven't
            already, this is your first step to enter the arena. Connect your X account and wallets.
          </li>
          <li>
            <strong class="text-white">Start Yapping:</strong> If it's your first time, start by
            sharing educational content about Union's mission and tech. Make sure to engage with
            fellow yappers and join the ongoing conversation with the Union community. Please read
            this yapper guide as a reference.
          </li>
          <li>
            <strong class="text-white">Climb the Leaderboard:</strong> Your impact is tracked in
            real-time. The more thoughtful, consistent, and resonant your yaps, the higher you can
            rise. Final rewards will be distributed based on your leaderboard ranking at the end of
            the season.
          </li>
        </ol>
        <p class="mb-4">
          Yappers, this is the mad sprint, the final chorus echoing across the wastelands. Whether
          you're a seasoned yapper or a fresh voice in the wilds, your moment is now. When the dust
          settles, only the truly mad shall be remembered.
        </p>
        <p class="text-orange-400 font-bold text-lg">
          The Yapocalypse has begun.
        </p>
      </div>
    </div>
  </div>
</Modal>
