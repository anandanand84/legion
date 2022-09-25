<script lang="ts">
    import * as legion from "legion";

    console.log(legion);
    // let status = legion.place_limit(1n, "BID", 2n, 20000n);
    // console.log(status);
    let state = legion.add_random_orders();
    console.log(state);
    let book = legion.get_book_state();

    console.log("book", book);
    
</script>

<div class="w-full h-full flex flex-row items-center justify-around">
    <div class="m-auto">Pages</div>
</div>