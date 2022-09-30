<script lang="ts">
  import "../app.postcss";
  import "carbon-components-svelte/css/all.css";
  let theme = "g90"; // "white" | "g10" | "g80" | "g90" | "g100"
  $: document.documentElement.setAttribute("theme", theme);
  import init from "legion";
  let initialized = false;
  init().then(() => {
    initialized = true;
  });
</script>

{#if initialized}
  <slot />
{:else}
<div class="w-full h-full flex flex-row items-center justify-around bg-red-100">
  <div class="m-auto">Loading...</div>
</div>
{/if}
