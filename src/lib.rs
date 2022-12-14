//! Legion implements a single-threaded order book. To use Legion, create an
//! order book instance with default parameters, and send orders for execution:
//!
//! ```rust
//! use legion::{FillMetadata, OrderBook, OrderEvent, OrderType, Side };
//!
//! let mut ob = OrderBook::default();
//! let event = ob.execute(OrderType::Market { id: 0, user_id: 1, qty: 1, side: Side::Bid });
//!
//! let event = ob.execute(OrderType::Limit { id: 1, user_id: 1, price: 120, qty: 3, side: Side::Ask });
//! assert_eq!(event, OrderEvent::Open { id: 1 });
//!
//! let event = ob.execute(OrderType::Market { id: 2, user_id: 1, qty: 4, side: Side::Bid });
//! assert_eq!(
//!     event,
//!     OrderEvent::PartiallyFilled {
//!         id: 2,
//!         filled_qty: 3,
//!         fills: vec![
//!             FillMetadata {
//!                 taker_id: 2,
//!                 maker_id: 1,
//!                 qty: 3,
//!                 price: 120,
//!                 taker_side: Side::Bid,
//!                 total_fill: true,
//!             }
//!         ],
//!     },
//! );
//! ```
//!
//! Legion only deals in integer price points and quantities. Prices and
//! quantities are represented as unsigned 64-bit integers. If the traded
//! instrument supports fractional prices and quantities, the conversion needs to
//! be handled by the user. At this time, Legion does not support negative prices.

#![warn(missing_docs, missing_debug_implementations, rustdoc::broken_intra_doc_links)]

mod arena;
mod models;
mod orderbook;
mod utils;
mod wasm;
mod rejectmessages;
mod orderbook_test;

pub use models::{
    BookDepth, BookLevel, FillMetadata, OrderEvent, OrderType, Side, Trade,
};
pub use rejectmessages::LIQUIDITY_NOT_AVAILABLE;
pub use orderbook::OrderBook;

