<script lang="ts">
  import { addEventListener, command } from "../bindings";
  let deviceName = $state("");
</script>

<input
  id="greet-input"
  placeholder="Enter device name"
  bind:value={deviceName}
/>
<button onclick={() => command("LinkSignal", { deviceName })}
  >Link device</button
>
