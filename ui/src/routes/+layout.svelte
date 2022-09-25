<script lang="ts">
  import "../app.postcss";
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
