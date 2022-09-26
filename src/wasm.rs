use std::{cell::RefCell, str::FromStr, vec};

use wasm_bindgen::prelude::*;

use crate::{Side, OrderType};

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
pub fn place_market(id:u64, side: String, qty: u64) -> JsValue{
    let event = ORDER_BOOK.with(|book| {
        return book.borrow_mut().execute(OrderType::Market{
            id, 
            side: if side.to_uppercase() == "BID" { Side::Bid } else { Side::Ask }, 
            qty
        });
    });
    serde_wasm_bindgen::to_value(&event).unwrap()
}


#[wasm_bindgen]
#[allow(dead_code)]
pub fn place_limit(id:u64, side: String, qty: u64, price: u64) -> JsValue{
    alert(&side);
    let event = ORDER_BOOK.with(|book| {
        return book.borrow_mut().execute(OrderType::Limit{
            id, 
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
pub fn get_last_sequence() -> u64 {
    return ORDER_BOOK.with(|book| {
        book.borrow().last_sequence()
    })
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn add_random_orders() -> JsValue{
    let orders = vec![
        "1,limit,BID,10,19990",
        "2,limit,BID,20,19989",
        "3,limit,BID,3,19978",
        "4,limit,BID,4,19955",
        "5,limit,BID,10,19991",
        "6,limit,BID,20,19994",
        "7,limit,BID,3,19990",
        "8,limit,BID,4,19979",
        "9,limit,ASK,5,19990",
        "10,limit,ASK,12,19999",
        "11,limit,ASK,3,20012",
        "12,limit,ASK,4,20042",
        "13,limit,ASK,100,20000",
        "14,limit,ASK,20,20001",
        "15,limit,ASK,3,20003",
        "16,limit,ASK,4,20012",
        "17,limit,ASK,1,20011",
        "18,limit,ASK,2,20009",
        "19,limit,ASK,2,20006",
        "20,limit,ASK,2,20006",
    ];
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
