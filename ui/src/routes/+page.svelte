<script lang="ts">
    import * as legion from "legion";
    import { Tooltip, TextArea, Button, Select, SelectItem, Slider, ButtonSet } from "carbon-components-svelte";
    import DepthRow from "../../src/components/market-depth/DepthRow.svelte";
    import restingorder from '../../../src/tests/restingorder.txt?raw'
    import marketordernoliquidityask from '../../../src/tests/marketorder-noliquidity-ask.txt?raw'
    import marketordernoliquidity from '../../../src/tests/marketorder-noliquidity.txt?raw'
    import LabelValue from "../../src/components/display/LabelValue.svelte";

    console.log(marketordernoliquidity);

    let tests = [
        { name: "--Select--", value: ""},
        { name: "Resting Orders", value: restingorder },
        { name: "Market Bid No Liquidity", value: marketordernoliquidity },
        { name: "Market Ask No Liquidity", value: marketordernoliquidityask },
    ]

    let spreadElement: HTMLElement = null;

    let spread = 0;
    // let status = legion.place_limit(1n, "BID", 2n, 20000n);
    // console.log(status);
    let events = legion.add_random_orders();
    let book = legion.get_book_state();

    $: renderBook(book);

    function centerBook() {
        setTimeout(()=> {
            spreadElement ? spreadElement.scrollIntoView({
                block: "center",
                inline: "center"
            }) : null;
        });
    }

    function renderBook(book:any) {
        book.asks.reduce((current:number, level: any)=> {
            level.total = current + level.qty;
            return level.total
        }, 0);
        book.bids.reverse().reduce((current:number, level: any)=> {
            level.total = current + level.qty;
            return level.total
        }, 0)

        if (book.bids.length && book.asks.length) {
            spread = book.asks[0].price - book.bids[0].price;
        }

        let max = Math.max(30, book.asks.length, book.bids.length);
        while (max > book.bids.length) {
            book.bids.push({ qty: '', price: '', total: ''})
        }
        while (max > book.asks.length) {
            book.asks.push({ qty: '', price: '', total: ''})
        }
        book.asks.reverse()
        centerBook();
    }

    console.log("book", book);

    let neworders = "";

    const sleep = (ms:number) => new Promise(r => setTimeout(r, ms));

    let delay = 50;

    async function clearBook() {
        legion.clear_book();
        book = legion.get_book_state();
        events = [];
        spread = 0;
    }

    async function processAllOrders() {
        for (const test of tests) {
            clearBook();
            await processOrders(test.value);
        }
    }

    async function processOrders(testorders:string) {
        let orders = testorders.split('\n');
        for (const orderString of orders) {
            await sleep(delay)
            let [order, result] = orderString.split('-')
            if (order.length != 0) {
                let last_processed = legion.get_last_sequence();
                let event;
                if (order.indexOf('cancel') != -1) {
                    event = legion.execute_order_text(order);
                }
                else if (order.indexOf('bbo')!=-1) {
                    let parsed:any = {};
                    let expectedString = orderString.split('bbo-')[1].split(',');
                    let [expectedBidQuantity, expectedBidPrice, expectedAskQuantity, expectedAskPrice] = expectedString.map((a)=>{
                                                                                                            return BigInt(a)
                                                                                                        });
                    let [bidQuantity, bidPrice, askQuantity, askPrice] = legion.get_bbo();
                    console.log("Expected", expectedBidQuantity, expectedBidPrice, expectedAskQuantity, expectedAskPrice);
                    console.log("Actual", bidQuantity, bidPrice, askQuantity, askPrice);
                    console.log(bidQuantity == expectedBidQuantity && bidPrice == expectedBidPrice
                        && askQuantity == expectedAskQuantity && askPrice == expectedAskPrice)
                    if (bidQuantity == expectedBidQuantity && bidPrice == expectedBidPrice
                        && askQuantity == expectedAskQuantity && askPrice == expectedAskPrice) {
                            parsed.success = true
                            parsed.message = 'BBO Match'
                    } else {
                        parsed.success = false
                        parsed.message = 'BBO Dont match'
                    }
                    events = [...events, parsed];
                    continue;
                } else {
                    event = legion.execute_order_text(`${last_processed + 1n},${order}`);
                }
                let eventType = Object.keys(event)[0];
                let orderEvent = event[eventType];
                let actual = ``
                let parsed = Object.assign({}, event[eventType]);
                parsed.type =eventType;
                switch (eventType) {
                    case "Open": 
                    case "Cancelled":
                    case "Rejected": 
                        actual = `${eventType.toLowerCase()},${orderEvent.id}`;
                        break;
                    case "Filled":
                    case "PartiallyFilled":
                        actual = `${eventType.toLowerCase()},${orderEvent.id},${orderEvent.filled_qty}`;
                        break;
                }
                console.log(actual, result);
                if(actual.indexOf(result) != -1) {
                    parsed.success=true;
                }
                
                events = [...events, parsed]
                book = legion.get_book_state();
            }   
        }
    }
</script>

<style>
    .transparent-border {
        border: 1px solid #302b2b
    }

    .level {
        min-width: 100px;
    }
    ::-webkit-scrollbar {
        display: none;
    }

    .event-red {
        background-color: #99333b;
    }
    .event-green {
        background-color: #339945;
    }
</style>

<div class="w-full h-full flex flex-row items-center justify-around pt-5 pb-5">
    <div class="h-full overflow-auto hide-scroll shadow-2xl">
        {#each book.asks as ask}
            <DepthRow sizeInteger={5} priceInteger={5} priceDecimals={0} bid={false} price={ask.price} size={ask.qty} value={ask.total}></DepthRow>
            <!-- <div class="relative flex flex-row justify-around ml-3 mr-5 p-3">
                <div>
                    {#if ask.qty > 0}
                         <Tooltip direction="right">
                             {#each ask.orders as order}
                             <div class="flex flex-row justify-between p-3">
                                 <span>{order.id}</span>
                                 <span>{order.user_id}</span>
                                 <span>{order.qty}</span>
                             </div>
                             {/each}
                         </Tooltip>
                    {/if}
                </div>
                <span class="level text-right">{ask.qty}</span>
                <span class="level text-right">{ask.price}</span>
                <span class="level text-right">{ask.total}</span>
            </div> -->
        {/each}
        <div bind:this={spreadElement} class="w-full flex flex-row justify-around cursor-pointer" on:click="{centerBook}">
            <div class="p-3"> Spread: {spread}</div>
        </div>
        {#each book.bids as bid}
            <DepthRow sizeInteger={5} priceInteger={5} priceDecimals={0} bid={true} price={bid.price} size={bid.qty} value={bid.total}></DepthRow>
            
            <!-- <div class="flex flex-row justify-around ml-3 p-3">
                <div>
                {#if bid.qty > 0}
                     <Tooltip direction="right">
                         {#each bid.orders as order}
                         <div class="flex flex-row justify-between p-3">
                             <span>{order.id}</span>
                             <span>{order.user_id}</span>
                             <span>{order.qty}</span>
                         </div>
                         {/each}
                     </Tooltip>
                {/if}
                </div>
                <span class="level text-right">{bid.qty}</span>
                <span class="level text-right">{bid.price}</span>
                <span class="level text-right">{bid.total}</span>
            </div> -->
        {/each}
    </div>
    <div style="overflow:auto;min-width:450px;" class="h-full m-0 p-5">
        {#each events.reverse() as event}
        <div class="transparent-border p-3" class:event-red={!event.success} class:event-green={event.success} style="width: 400px;">
            {#if event.message?.length > 0}
                <h5 class="flex flex-row justify-between mb-5">
                    {event.message}
                </h5>
            {/if}
            <div class="flex flex-row justify-between">
                {#if event.id}
                     <LabelValue label="OrderID:" value="{event.id}" title=""></LabelValue>
                     <LabelValue label="Qty:" value="{event.filled_quantity || 'NA'}" title=""></LabelValue>
                     <LabelValue label="Status:" value="{event.type}" title=""></LabelValue>
                {/if}
            </div>
        </div>
        {/each}
    </div>
    <div class="flex flex-col justify-start h-full">
        <!-- <span>order types: limit,market,ioc,fok</span>
        <span>user_id,ordertype,side,quantity,price</span>
        <span>ex: </span>
        <span>1,limit,bid,10,20000</span>
        <span>2,limit,ask,10,20001</span>
        <span>22,cancel</span> -->
        <div>
            <Slider
                labelText="Delay Between Orders"
                min={0}
                max={10000}
                maxLabel="10000 msec"
                bind:value={delay}
                />
            <Select bind:selected={neworders}>
                {#each tests as test}
                     <SelectItem value="{test.value}" text="{test.name}" />
                {/each}
            </Select>
        </div>
        <textarea style="padding:10px;font-size:1.2rem; line-height:2rem; background:#393939;" bind:value="{neworders}" cols="50" class="flex-1 mt-3a"></textarea>
        <ButtonSet>
            <Button on:click={()=> clearBook()}>Clear Book</Button> <Button on:click={()=> processOrders(neworders)}>Test</Button> <Button on:click={processAllOrders}>Test ALL</Button>
        </ButtonSet>
    </div>
</div>

