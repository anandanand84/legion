<script context="module">
    var sizeCacheMap = new Map();
    if (!String.prototype.padEnd) {
      String.prototype.padEnd = function padEnd(targetLength, padString) {
        targetLength = targetLength >> 0; //floor if number or convert non-number to 0;
        padString = String(padString || " ");
        if (this.length > targetLength) {
          return String(this);
        } else {
          targetLength = targetLength - this.length;
          if (targetLength > padString.length) {
            padString += padString.repeat(targetLength / padString.length); //append to original to ensure we are longer than needed
          }
          return String(this) + padString.slice(0, targetLength);
        }
      };
    }
  </script>
  
  <style>
    .hidden {
      display: none;
    }
  
    .up {
      color: var(--theme-up-color);
    }
  
    .down {
      color: var(--theme-down-color);
    }
  
    .hidden-character:after {
      content: " ";
    }
  </style>
  
  <span
    bind:this="{container}"
    style="min-width:{minWidth}"
    class:up="{showColor && price > 0}"
    class:down="{showColor && price < 0}"
  >
    <span class:hidden-character="{!priceNegative}">{sign}</span
    ><span class="select-none" style="opacity: 0.1">{hidden}</span
    ><ColorText
      {makeReadable}
      highlightIndex="{prefixHighlightIndex}"
      highlightLength="{prefixHighlightLength}"
      highlight="{highlight}"
      text="{prefix}"
    ></ColorText
    ><ColorText
      text=".{suffix}"
      highlightIndex="{suffixHighlightIndex}"
      highlightLength="{suffixHighlightLength}"
      highlight="{highlight}"
    ></ColorText
    ><span class="select-none" style="opacity: 0.2">{ignore}</span>
  </span>
  {#if showCopy}
  <div
    in:fade
    out:fade
    class="fixed p-2 text-white rounded"
    style="{`top:${mousey}; left:${mousex}px; background: #101a2c;font-size:10px;`}"
  >
    Text Copied!
  </div>
  {/if}
  <script>
    import ColorText, { splitValue } from "./ColorText.svelte";
    import { fade, fly } from "svelte/transition";
    import { onMount } from "svelte";
    export let showColor = false;
    export let price=0;
    export let decimalPlaces = 8;
    export let maxIntegerPlaces = 7;
    export let updateColor = "";
    let showCopy=false;
    let mousex;
    let mousey;
    let prefixLength;
    let suffixLength;
  
    let window;
    let container;
    let hidden;
    let upColor;
    let downColor;
    let changedColor;
    let prefix;
    let suffix;
    let ignore;
  
    $: priceNegative = price < 0;
    $: sign = priceNegative ? "-" : "";
  
    export let highlightCount = 3;
    export let highlight = false;
    export let makeReadable = true;
  
    let prefixHighlightIndex = -1;
    let prefixHighlightLength = 0;
    let suffixHighlightIndex = -1;
    let suffixHighlightLength = 0;
  
    let mounted = false;
    let minWidth = "100px";
  
  
    onMount(() => {
      mounted = true;
      charactersUpdate();
    });
  
    $: priceChanged(price, maxIntegerPlaces);
  
    $: charactersUpdate(decimalPlaces, maxIntegerPlaces);
  
    function priceChanged(newPrice, maxIntegerPlaces) {
      newPrice = parseFloat(Math.abs(newPrice));
      let priceString = newPrice.toFixed(decimalPlaces);
      let calculatedIgnore = "".padEnd(decimalPlaces, "0");
      let calculatedSuffix = "";
      let changedColor = upColor;
      let [calculatedPrefix, decimals] = priceString.split(".");
      hidden = "".padStart(maxIntegerPlaces - calculatedPrefix.length, "0");
      if (parseFloat(decimals) > 0) {
        let ignoreLength = 0;
        let characters = decimals.split();
        let index = decimals.length - 1;
        while (decimals[index] == 0) {
          ignoreLength++;
          index--;
        }
        [calculatedSuffix, calculatedIgnore] = splitValue(
          decimals,
          decimals.length - ignoreLength
        );
      }
      prefix = calculatedPrefix;
      ignore = calculatedIgnore;
      suffix = calculatedSuffix;
    }
  
    $: calculateHighlight(prefix, suffix);
  
    function calculateHighlight() {
      if (highlight) {
        prefixHighlightLength = highlightCount;
        let prefixLength = prefix.length;
        if (prefix == "0") {
          prefixLength = 0;
          prefixHighlightLength = 0;
        }
        prefixHighlightIndex = prefixLength - highlightCount;
        if (prefixHighlightIndex < 0) {
          suffixHighlightLength = prefixHighlightIndex * -1;
          prefixHighlightIndex = 0;
          if (prefixHighlightLength > 0) {
            suffixHighlightIndex = 1;
          } else {
            suffixHighlightIndex =
              suffix.length - parseFloat(suffix).toString().length + 1;
          }
        }
      }
    }
  
    function charactersUpdate() {
      if (!mounted) return;
      // if(decimalPlaces && maxIntegerPlaces) {
      //     let totalCharacters = decimalPlaces+maxIntegerPlaces+1;
      //     let textWidth = sizeCacheMap.get(totalCharacters);
      //     if(!textWidth) {
      //         let styles = getComputedStyle(container);
      //         upColor = updateColor || styles.getPropertyValue('--theme-up-color');
      //         downColor = updateColor || styles.getPropertyValue('--theme-down-color');
      //         var ctx = document.createElement('canvas').getContext('2d');
      //         ctx.font = styles.font;
      //         textWidth = ctx.measureText(''.padEnd(totalCharacters, '0')).width;
      //         sizeCacheMap.set(totalCharacters, textWidth);
      //     }
      //     minWidth = textWidth + 'px';
      // }
    }
  </script>
  