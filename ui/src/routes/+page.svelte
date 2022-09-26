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
        let orders = neworders.split('\n');
        orders.forEach((order)=> {
            if (order.length != 0) {
                let last_processed = legion.get_last_sequence();
                let event = legion.execute_order_text(`${last_processed + 1n},${order}`);
                events = [...events.reverse(), event]
                book = legion.get_book_state();
            }
        })
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
        <span>order types: limit,market,ioc,fok</span>
        <span>user_id,ordertype,side,quantity,price</span>
        <span>ex: </span>
        <span>1,limit,bid,10,20000</span>
        <span>2,limit,ask,10,20001</span>
        <textarea style="border:1px solid grey;" bind:value="{neworders}" />
        <button on:click={processOrders}>Submit</button>
    </div>
</div>

