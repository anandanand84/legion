<style>
	.highlight {
		font-size: 2em;
	}

	.readable {
		border-bottom: 1px solid #c7bebe8a;
	}
</style>

<script context="module">
    export const splitValue = function(value, index) {
        return [value.substring(0, index), value.substring(index)]
    }
		
	var splitLength = function(value, index, length) {
			return [value.substring(0, index),value.substring(index, index+length), value.substring(index+length)]
	}
</script>

<script>
	export let highlight=false;
    export let text='';
	export let highlightIndex=0
    export let highlightLength=0;
	export let makeReadable=false;
	
	let prefix='';
	let suffix='';
	let highlightText='';

	
    $:priceChanged(text, highlightLength, highlightIndex)

    function priceChanged() {
		suffix='';
		prefix='';
		highlightText='';
		if(text && text.length > 0 && highlightLength > 0 && highlightIndex >=0) {
			let values = splitLength(text, highlightIndex, highlightLength);
			prefix = [values[0]];
			suffix=values[2];
			highlightText=values[1];
			return;
		}
		if(makeReadable) {
			prefix = parseInt(text).toLocaleString().split(',');
		} else {
			prefix = [text];	
		}
		suffix='';
    }

</script>

{#each prefix as item,index}{#if index!=0}<span class:readable={makeReadable}>{item.slice(0,1)}</span><span>{item.slice(1,item.length)}</span>{:else}<span>{item}</span>{/if}{/each}<span class:highlight={highlight}>{highlightText}</span><span>{suffix}</span>

