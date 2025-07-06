<script lang="ts">
  import SignalLinking from "$lib/pages/SignalLinking.svelte";
  import Start from "$lib/pages/Start.svelte";
  import { addEventListener, match } from "../lib/bindings";

  let page: "start" | "signal-link" = "start";
  let signalLinkingUrl: string | undefined;

  addEventListener({}, (e) => {
    match(e, {
      SignalEvent(e) {},
      SignalStateChanged(s) {
        match(s, {
          None() {
            page = "start";
          },
          Linking({ url }) {
            page = "signal-link";
            signalLinkingUrl = url;
          },
          Registering() {},
          Connected() {},
        });
      },
    });
  });
</script>

<main class="container">
  {#if page === "start"}
    <Start />
  {:else if page === "signal-link"}
    <SignalLinking url={signalLinkingUrl!} />
  {/if}
</main>
