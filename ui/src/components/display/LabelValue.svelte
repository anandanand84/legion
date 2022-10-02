<style>
    .header-label {
      color: #aeaeb2;
    }
  </style>
  <script lang="ts">
    import InlineEdit from './InlineEdit.svelte';
  
    export let label="";
    export let value="";
    export let title="";
    
    export let editable = false;
    export let editoptions:any = [];
  
    let color = "white";

  </script>
  <div class="flex flex-row justify-between items-center">
    <div class="w-full">
      <span class="text-sm mb-1 header-label">{label}</span>
      <div class="text-sm overflow-hidden" style="text-overflow: ellipsis;">
        {#if editable}
          <InlineEdit options={editoptions} on:save bind:value="{value}" position="absolute"></InlineEdit>
        {:else}
           <span title="{title}">{value}</span>
        {/if}
      </div>
    </div>
  </div>
  