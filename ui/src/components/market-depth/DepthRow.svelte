<style>

    :root {
        --theme-up-color: #4caf50;
        --theme-down-color: #fc6a42;
    }

    .depth-row {
        min-height: 42px;
        min-width: 326px;
        line-height: 1rem;
        position: relative;
        font-size: 14px;
        display: flex;
        justify-content: space-between;
        text-align: right;
        word-break: normal;
        flex-wrap: nowrap;
        white-space: nowrap;
        font-size: 1rem;
    }

    .hover-background {
        background-color: rgba(0, 0, 0, .12);
    }

    .depth-row:hover {
        background-color: rgba(0, 0, 0, .12);
        cursor: pointer;
    }

    .bid {
        color: var(--theme-up-color);
    }

    .ask {
        color: var(--theme-down-color);
    }

    .flex-text {
        word-break: normal;
        flex-wrap: nowrap;
        white-space: nowrap;
        justify-content : end;
        border: 1px solid #302b2b
    }


    .size-chart {
        height: 17px;
        position: absolute;
        min-width: 2px;
        opacity: 0.5;
    }

    .ask-size-background {
        stroke: var(--theme-down-color);
        fill: var(--theme-down-color);
    }

    .bid-size-background {
        stroke: var(--theme-up-color);
        fill: var(--theme-up-color);
    }

</style>

<div class="p-3 depth-row flex-text" on:click="{priceClick}" class:ask={!bid} class:bid={bid}>
    {#if price > 0}
         <svg class="size-chart" width="38" height="17" aria-labelledby="Depth Chart" role="img">
             <g class:bid-size-background={bid} class:ask-size-background={!bid}>
                 <rect width="{relativeSize}" height="17"></rect>
             </g>
         </svg>
         <!-- <span class\$="size-chart" style\$="height:15px;width:{relativeSize}px;"></span> -->
         <Price maxIntegerPlaces="{sizeInteger}" decimalPlaces="{sizeDecimals}" price="{size}"></Price>
         <Price maxIntegerPlaces="{priceInteger}" decimalPlaces="{priceDecimals}" price="{price}"></Price>
         <Price maxIntegerPlaces="{sizeInteger}" decimalPlaces="{2}" price="{value}"></Price>
    {/if}
</div>

<script>

    import Price from '../display/Price.svelte'
    export let bid = true;
    export let relativeSize="";
    export let sizeInteger = 10;
    export let sizeDecimals = 0;
    export let priceInteger = 10;
    export let priceDecimals = 0;
    export let price = 0;
    export let size = 0;
    export let value = 0;

    function priceClick() {

    }

</script>
