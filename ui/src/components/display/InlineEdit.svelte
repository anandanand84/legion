<script>
    import { createEventDispatcher } from "svelte";
  
    export let loading = false;

    const dispatch = createEventDispatcher();
  
    let readonly = true,
      options = [],
      value = "",
      rows = 1,
      input,
      attrs,
      list;
  
    $: options.length && (list = Date.now());
    $: {
      const { value, rows, options, ...other } = $$props;
      attrs = other;
    }
  
    function edit() {
      readonly && (readonly = false);
      dispatch("edit", input);
    }
  
    function cancel() {
      input.value = value;
      readonly = true;
      dispatch("cancel", input);
    }
  
    function save() {
      value = input.value;
      readonly = true;
      dispatch("save", input);
    }
  
    export { value, rows, options };
  </script>
  
  <style>
    [class^="editable-"] {
      position: relative;
      display: inline-block;
      overflow: visible;
    }
    [class^="editable-"] div {
      position: absolute;
      overflow: visible;
    }
  
    input,
    textarea,
    button {
      margin: 0;
      resize: none;
      background: #1d1d1d;
    }
    [readonly],
    button {
      border: none;
      background: transparent;
    }
    button:hover {
      cursor: pointer;
    }
    [readonly]:hover {
      background: grey;
      cursor: url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAB60lEQVR42mNkIAHw8fFxycnJ9b1//37a06dPL4HEGEkxQEZGxkVERGQnkPnvz58/s1+8eFFHtAGamppFrKysIv/+/dvLzMzczMjIaAkUfkqUAeLi4kqSkpJXgUwmIH779+/fqv/////9/fs3D1EG6OrqrmNiYhID+tsd6IVSDg6OMqABl+/fv+9M0ABgoDkJCQntevPmjfmTJ0/OgsT09fXXAyml27dvG+E1gIuLi1lVVfU80Mmnr1y5kgwSk5aWNhYVFT0JjAm3hw8f7sNrgJqaWhbQkHagzWpAF7yE2n4E6PyXly5dCsYbjUBnCwKdf+vHjx/dN27c6AKJKSsrR/Ly8s4DRp82EN/Da4C2tvZEYLR53b17V/vz58+/QIlISUnpxq9fv5Zcu3atCqYOqwHAKNMCRt2Fjx8/hgNDej00HTSws7OnPnjwQP3Dhw9f8BoAjLYlwMQSDYzn6c+fP69kYWHhl5KSuv7ly5esO3fuLERWi9UALS2tuWxsbElQ7isgfgsMuM+3bt2y+P79+3+CBgADqxIYWG1IQv/evn1r/fjx4xPoahmBGcQPSDPDBIA2XODk5DQGprjVQO5fYNrfDQyLKcA434rNMkZgaIMUssAEgCE+++vXrzeABiS9e/duzuvXrx/gSysAfu/VCjmyfS4AAAAASUVORK5CYII="),
        text;
    }
    [readonly]:focus {
      outline: none;
    }
  
    .editable-absolute{
        position: relative;
    }

    .buttons-absolute{
        position: absolute;
        right: 0px !important;
        top: 0px !important;
    }

  </style>
  
  <svelte:body on:click={cancel} />
  
  <div class="editable-absolute">
    {#if rows > 1}
      <textarea
        bind:this={input}
        on:click|self|stopPropagation={edit}
        {...attrs}
        {readonly}
        {value}
        {rows}
        {list} />
    {:else}
      <input
        bind:this={input}
        on:click|self|stopPropagation={edit}
        {...attrs}
        {readonly}
        {value}
        {list} />
    {/if}
  
    {#if options.length}
      <datalist id={list}>
        {#each options as value}
          <option {value} />
        {/each}
      </datalist>
    {/if}
  
    {#if !readonly}
      <div class="buttons-absolute">
        <button on:click={save} type="button">
          <slot name="save">&check;</slot>
        </button>
        <button on:click={cancel} type="button">
          <slot name="cancel">&cross;</slot>
        </button>
      </div>
    {/if}
  </div>
  