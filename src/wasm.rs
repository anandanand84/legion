use std::cell::RefCell;

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
pub fn greet() {
    alert("Hello");
}

#[wasm_bindgen]
pub fn get_book_state() -> JsValue {
    let state = ORDER_BOOK.with(|book| {
        let book_state = book.borrow_mut();
        return (book_state._bids(), book_state._asks())
    });
    serde_wasm_bindgen::to_value(&state).unwrap()
}

#[wasm_bindgen]
pub fn place_market(id:u64, side: u8, qty: u64) -> JsValue{
    let event = ORDER_BOOK.with(|book| {
        return book.borrow_mut().execute(OrderType::Market{
            id, 
            side:Side::from_repr(side).unwrap_or(Side::Bid), 
            qty
        });
    });
    serde_wasm_bindgen::to_value(&event).unwrap()
}


#[wasm_bindgen]
pub fn place_limit(id:u64, side: u8, qty: u64, price: u64) -> JsValue{
    let event = ORDER_BOOK.with(|book| {
        return book.borrow_mut().execute(OrderType::Limit{
            id, 
            side:Side::from_repr(side).unwrap_or(Side::Bid), 
            qty,
            price
        });
    });
    serde_wasm_bindgen::to_value(&event).unwrap()
}

#[wasm_bindgen]
pub fn place_cancel(id:u64) -> JsValue{
    let event = ORDER_BOOK.with(|book| {
        return book.borrow_mut().execute(OrderType::Cancel{
            id
        });
    });
    serde_wasm_bindgen::to_value(&event).unwrap()
}