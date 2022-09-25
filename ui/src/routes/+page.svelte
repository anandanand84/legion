<script lang="ts">
    import * as legion from "legion";

    console.log(legion);
    // let status = legion.place_limit(1n, "BID", 2n, 20000n);
    // console.log(status);
    let events = legion.add_random_orders();
    let book = legion.get_book_state();

    $: asks = book.asks.reverse();
    $: bids = book.bids.reverse();

    console.log("book", book);

    let neworders = "";

    function processOrders() {
        if (neworders.length != 0) {
            console.log(neworders);
            let event = legion.execute_order_text(neworders);
            console.log(event);
            events = [...events, event]
            book = legion.get_book_state();
        }
    }
</script>

<style>
    .level {
        min-width: 100px;
    }
</style>

<div class="w-full h-full flex flex-row items-center justify-around">
    <div class="m-auto">
        {#each asks as ask}
            <div class="flex flex-row justify-around ml-3 mr-3 bg-red-100 p-3">
                <span class="level text-right">{ask.qty}</span><span class="level text-right">{ask.price}</span>
            </div>
        {/each}
        <div class="p-3"> Spread: </div>
        {#each bids as bid}
            <div class="flex flex-row justify-around ml-3 mr-3 bg-green-100 p-3">
                <span class="level text-right">{bid.qty}</span><span class="level text-right">{bid.price}</span>
            </div>
        {/each}
    </div>
    <div style="overflow:auto; max-height: 600px; p-3">
        {#each events.reverse() as event}
            <div class="flex flex-row justify-around ml-3 mr-3 bg-grey-100 p-3">
                <span class="level text-right">{Object.keys(event)[0]}</span>
                <span class="level text-right">{event[Object.keys(event)[0]].id}</span>
                <span class="level text-right">{event[Object.keys(event)[0]].filled_qty ? event[Object.keys(event)[0]].filled_qty : ""}</span>
            </div>
        {/each}
    </div>
    <div class="flex flex-col justify-between">
        <textarea style="border:1px solid grey;" bind:value="{neworders}" />
        <button on:click={processOrders}>Submit</button>
    </div>
</div>

