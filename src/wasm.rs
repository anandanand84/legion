use std::{cell::RefCell, str::FromStr, vec};

use wasm_bindgen::prelude::*;

use crate::{Side, OrderType, OrderBook};

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

thread_local! {
    static ORDER_BOOK:RefCell<crate::OrderBook> = RefCell::new(crate::OrderBook::default())
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn get_book_state() -> JsValue {
    let state = ORDER_BOOK.with(|book| {
        let book_state = book.borrow_mut();
        return book_state.depth(199, true);
    });
    serde_wasm_bindgen::to_value(&state).unwrap()
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn place_market(id:u64, user_id: u64, side: String, qty: u64) -> JsValue{
    let event = ORDER_BOOK.with(|book| {
        return book.borrow_mut().execute(OrderType::Market{
            id, 
            user_id,
            side: if side.to_uppercase() == "BID" { Side::Bid } else { Side::Ask }, 
            qty
        });
    });
    serde_wasm_bindgen::to_value(&event).unwrap()
}


#[wasm_bindgen]
#[allow(dead_code)]
pub fn place_limit(id:u64, user_id: u64, side: String, qty: u64, price: u64) -> JsValue{
    alert(&side);
    let event = ORDER_BOOK.with(|book| {
        return book.borrow_mut().execute(OrderType::Limit{
            id, 
            user_id,
            side: if side.to_uppercase() == "BID" { Side::Bid } else { Side::Ask } , 
            qty,
            price
        });
    });
    serde_wasm_bindgen::to_value(&event).unwrap()
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn place_cancel(id:u64) -> JsValue{
    let event = ORDER_BOOK.with(|book| {
        return book.borrow_mut().execute(OrderType::Cancel{
            id
        });
    });
    serde_wasm_bindgen::to_value(&event).unwrap()
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn execute_order_text(order:String) -> JsValue {
    let event = ORDER_BOOK.with(|book| {
        let mut book_state = book.borrow_mut();
        let result = book_state.execute(OrderType::from_str(&order).unwrap());
        return result
    });
    serde_wasm_bindgen::to_value(&event).unwrap()
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn clear_book() -> () {
    return ORDER_BOOK.with(|book| {
        let mut bookref = book.borrow_mut();
        *bookref = OrderBook::default();
    })
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn get_bbo() -> Vec<u64> {
    return ORDER_BOOK.with(|book| {
        let depth = book.borrow().depth(1, false);
        let bid:Vec<u64> = depth.bids.iter().map(|level| vec![level.qty.clone(), level.price.clone()]).flatten().collect();
        let ask:Vec<u64> = depth.asks.iter().map(|level| vec![level.qty.clone(), level.price.clone()]).flatten().collect();
        let all:Vec<u64> = bid.into_iter().chain(ask.into_iter()).collect();
        return all;
    });
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn get_last_sequence() -> u64 {
    return ORDER_BOOK.with(|book| {
        book.borrow().last_sequence()
    })
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn add_random_orders() -> JsValue{
    let orders:Vec<String> = Vec::new();
    let mut events = Vec::new();
    ORDER_BOOK.with(|book| {
        let mut book_state = book.borrow_mut();
        for order in orders.iter() {
            let result = book_state.execute(OrderType::from_str(order).unwrap());
            events.push(result);
        }
    });
    serde_wasm_bindgen::to_value(&events).unwrap()
}
