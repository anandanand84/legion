use std::{str::FromStr};
use serde::{Serialize, Deserialize};
use strum_macros::{EnumString, FromRepr};

/// An order book side.
#[derive(Debug, Copy, Clone, PartialEq, EnumString, FromRepr, Default, Serialize, Deserialize)]
#[strum(serialize_all = "kebab_case")]
#[repr(u8)]
pub enum Side {
    /// The bid (or buy) side.
    #[default]
    Bid=0,
    /// The ask (or sell) side.
    Ask,
}

impl std::ops::Not for Side {
    type Output = Side;

    fn not(self) -> Self::Output {
        match self {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        }
    }
}


pub type Price = u64;
pub type Qty = u64;
pub type OrderId = u64;

/// An order to be executed by the order book.
#[derive(Debug, Copy, Clone)]
pub enum OrderType {
    /// A market order, which is either filled immediately (even partially), or
    /// canceled.
    Market {
        /// The unique ID of this order.
        id: OrderId,
        /// The order side. It will be matched against the resting orders on the
        /// other side of the order book.
        side: Side,
        /// The order quantity.
        qty: Qty,
    },
    /// A limit order, which is either filled immediately, or added to the order
    /// book.
    Limit {
        /// The unique ID of this order.
        id: OrderId,
        /// The order side. It will be matched against the resting orders on the
        /// other side of the order book.
        side: Side,
        /// The order quantity.
        qty: Qty,
        /// The limit price. The order book will only match this order with
        /// other orders at this price or better.
        price: Price,
    },
    // /// A Imediate or cancel order, which filled immediately the avilable qty at the price
    // ///  and cancels the remaining qty
    // IOC {
    //     /// The unique ID of this order.
    //     id: OrderId,
    //     /// The order side. It will be matched against the resting orders on the
    //     /// other side of the order book.
    //     side: Side,
    //     /// The order quantity.
    //     qty: Qty,
    //     /// The limit price. The order book will only match this order with
    //     /// other orders at this price or better.
    //     price: Price,
    // },
    // /// Fill or Kill order, which fills completely or rejects everything, no partial fills
    // FOK {
    //     /// The unique ID of this order.
    //     id: OrderId,
    //     /// The order side. It will be matched against the resting orders on the
    //     /// other side of the order book.
    //     side: Side,
    //     /// The order quantity.
    //     qty: Qty,
    //     /// The limit price. The order book will only match this order with
    //     /// other orders at this price or better.
    //     price: Price,
    // },
    /// A cancel order, which removes the order with the specified ID from the
    /// order book.
    Cancel {
        /// The unique ID of the order to be canceled.
        id: OrderId,
    },
}

use thiserror::Error;


#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum OrderParseError {
    #[error("Invalid fields count for order type")]
    InvalidFieldSize,
    #[error("Invalid order type")]
    InvalidOrderType,
    #[error("Invalid Integer")]
    InvalidInteger,
    #[error("Invalid Side")]
    InvalidSide
}

impl FromStr for OrderType {
    type Err=OrderParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fields = s.split(",").collect::<Vec<&str>>();
        let total_fields = fields.len();
        if  total_fields < 1 {
            return Err(OrderParseError::InvalidFieldSize)
        }
        let ordertype = fields[0];
        match ordertype {
            "market" => {
                if total_fields != 4 {
                    return Err(OrderParseError::InvalidFieldSize)
                }
                Ok(OrderType::Market { 
                    id: fields[1].parse::<u64>().map_err(|_| OrderParseError::InvalidInteger)?,
                    side: Side::from_str(fields[2]).map_err(|_| OrderParseError::InvalidSide)?, 
                    qty: fields[3].parse::<u64>().map_err(|_| OrderParseError::InvalidInteger)? , 
                })
            },
            "limit" => {
                if total_fields != 5 {
                    return Err(OrderParseError::InvalidFieldSize)
                }
                Ok(OrderType::Limit { 
                    id: fields[1].parse::<u64>().map_err(|_| OrderParseError::InvalidInteger)?,
                    side: Side::from_str(fields[2]).map_err(|_| OrderParseError::InvalidSide)?, 
                    qty: fields[3].parse::<u64>().map_err(|_| OrderParseError::InvalidInteger)?,
                    price: fields[4].parse::<u64>().map_err(|_| OrderParseError::InvalidInteger)?, 
                })
            },
            "cancel" => {
                if total_fields != 2 {
                    return Err(OrderParseError::InvalidFieldSize)
                }
                Ok(OrderType::Cancel { 
                    id: fields[1].parse::<u64>().unwrap()
                })
            },
            _ => {
                return Err(OrderParseError::InvalidOrderType) 
            }

        }
    }   
}


/// An event resulting from the execution of an order.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum OrderEvent {
    /// Indicating that the corresponding order was not filled. It is only sent
    /// in response to market orders.
    Rejected {
        /// The ID of the order this event is referring to.
        id: OrderId
    },
    /// Indicating that the corresponding order was placed on the order book. It
    /// is only send in response to limit orders.
    Placed {
        /// The ID of the order this event is referring to.
        id: OrderId,
    },
    /// Indicating that the corresponding order was removed from the order book.
    /// It is only sent in response to cancel orders.
    Canceled {
        /// The ID of the order this event is referring to.
        id: OrderId,
    },
    /// Indicating that the corresponding order was only partially filled. It is
    /// sent in response to market or limit orders.
    PartiallyFilled {
        /// The ID of the order this event is referring to.
        id: OrderId,
        /// The filled quantity.
        filled_qty: Qty,
        /// A vector with information on the order fills.
        fills: Vec<FillMetadata>,
    },
    /// Indicating that the corresponding order was filled completely. It is
    /// sent in response to market or limit orders.
    Filled {
        /// The ID of the order this event is referring to.
        id: OrderId,
        /// The filled quantity.
        filled_qty: Qty,
        /// A vector with information on the order fills.
        fills: Vec<FillMetadata>,
    },
}

/// Information on a single order fill. When an order is matched with multiple
/// resting orders, it generates multiple `FillMetadata` values.
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct FillMetadata {
    /// The ID of the order that triggered the fill (taker).
    pub order_1: OrderId,
    /// The ID of the matching order.
    pub order_2: OrderId,
    /// The quantity that was traded.
    pub qty: Qty,
    /// The price at which the trade happened.
    pub price: Price,
    /// The side of the taker order (order 1)
    pub taker_side: Side,
    /// Whether this order was a total (true) or partial (false) fill of the
    /// maker order.
    pub total_fill: bool,
}

/// A snapshot of the order book up to a certain depth level. Multiple orders at
/// the same price points are merged into a single [`BookLevel`] struct.
///
/// [`BookLevel`]: /struct.BookLevel.html
#[derive(Debug, Clone, PartialEq)]
pub struct BookDepth {
    /// The requested level. This field will always contain the level that was
    /// requested, even if some or all levels are empty.
    pub levels: usize,
    /// A vector of price points with the associated quantity on the ask side.
    pub asks: Vec<BookLevel>,
    /// A vector of price points with the associated quantity on the bid side.
    pub bids: Vec<BookLevel>,
}

/// A single level in the order book. This struct is used both for the bid and
/// ask side.
#[derive(Debug, Clone, PartialEq)]
pub struct BookLevel {
    /// The price point this level represents.
    pub price: Price,
    /// The total quantity of all orders resting at the specified price point.
    pub qty: Qty,
}

/// A trade that happened as part of the matching process.
#[derive(Debug, Copy, Clone)]
pub struct Trade {
    /// The total quantity transacted as part of this trade.
    pub total_qty: Qty,
    /// The volume-weighted average price computed from all the order fills
    /// within this trade.
    pub avg_price: f64,
    /// The price of the last fill that was part of this trade.
    pub last_price: Price,
    /// The quantity of the last fill that was part of this trade.
    pub last_qty: Qty,
}

#[derive(Debug, PartialEq)]
pub struct LimitOrder {
    pub id: OrderId,
    pub qty: Qty,
    pub price: Price,
}

#[cfg(test)]
mod test {
    use super::Side;

    #[test]
    fn side_negation() {
        assert_eq!(!Side::Ask, Side::Bid);
        assert_eq!(!Side::Bid, Side::Ask);
    }
}
