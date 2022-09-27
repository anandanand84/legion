use std::collections::BTreeMap;

use crate::rejectmessages::{LIQUIDITY_NOT_AVAILABLE, self};
use crate::arena::OrderArena;
use crate::models::{
    BookDepth, BookLevel, FillMetadata, OrderEvent, OrderType, Side, Trade, OrderId, Qty, Price, UserId,
};

const DEFAULT_ARENA_CAPACITY: usize = 10_000;
const DEFAULT_QUEUE_CAPACITY: usize = 10;

/// An order book that executes orders serially through the [`execute`] method.
///
/// [`execute`]: #method.execute
#[derive(Debug)]
pub struct OrderBook {
    last_processed_order_id: u64,
    last_trade: Option<Trade>,
    traded_volume: Qty,
    min_ask: Price,
    max_bid: Price,
    asks: BTreeMap<Price, Vec<OrderId>>,
    bids: BTreeMap<Price, Vec<OrderId>>,
    arena: OrderArena,
    default_queue_capacity: usize,
    track_stats: bool,
}

impl Default for OrderBook {
    /// Create an instance representing a single order book, with stats tracking
    /// disabled, a default arena capacity of 10,000 and a default queue
    /// capacity of 10.
    fn default() -> Self {
        Self::new(DEFAULT_ARENA_CAPACITY, DEFAULT_QUEUE_CAPACITY, false)
    }
}

impl OrderBook {
    /// Create an instance representing a single order book.
    ///
    /// The `arena_capacity` parameter represents the number of orders that will
    /// be pre-allocated.
    ///
    /// The `queue_capacity` parameter represents the capacity of each vector
    /// storing orders at the same price point.
    ///
    /// The `track_stats` parameter indicates whether to enable volume and
    /// trades tracking (see [`last_trade`] and [`traded_volume`]).
    ///
    /// [`last_trade`]: #method.last_trade
    /// [`traded_volume`]: #method.traded_volume
    pub fn new(
        arena_capacity: usize,
        queue_capacity: usize,
        track_stats: bool,
    ) -> Self {
        Self {
            last_processed_order_id: 0,
            last_trade: None,
            traded_volume: 0,
            min_ask: std::u64::MAX,
            max_bid: 0u64,
            asks: BTreeMap::new(),
            bids: BTreeMap::new(),
            arena: OrderArena::new(arena_capacity),
            default_queue_capacity: queue_capacity,
            track_stats,
        }
    }

    #[doc(hidden)]
    pub fn _asks(&self) -> Vec<(Price, Vec<OrderId>)> {
        self.asks.clone().into_iter().collect()
    }

    #[doc(hidden)]
    pub fn _bids(&self) -> Vec<(Price, Vec<OrderId>)> {
        self.bids.clone().into_iter().collect()
    }

    /// Return the lowest ask price, if present.
    #[inline(always)]
    pub fn min_ask(&self) -> Price {
        self.min_ask
    }

    /// Return the highest bid price, if present.
    #[inline(always)]
    pub fn max_bid(&self) -> Price {
        self.max_bid
    }

    /// Return the difference of the lowest ask and highest bid, if both are
    /// present.
    #[inline(always)]
    pub fn spread(&self) -> Price { 
        self.min_ask - self.max_bid
    }

    /// Return the last sequence processed
    #[inline(always)]
    pub fn last_sequence(&self) -> u64 {
        self.last_processed_order_id
    }

    /// Return the last trade recorded while stats tracking was active as a
    /// [`Trade`] object, if present.
    ///
    /// [`Trade`]: struct.Trade.html
    #[inline(always)]
    pub fn last_trade(&self) -> Option<Trade> {
        self.last_trade
    }

    /// Return the total traded volume for all the trades that occurred while
    /// the stats tracking was active.
    #[inline(always)]
    pub fn traded_volume(&self) -> Qty {
        self.traded_volume
    }

    /// Return the order book depth as a [`BookDepth`] struct, up to the
    /// specified level. Bids and offers at the same price level are merged in a
    /// single [`BookLevel`] struct.
    ///
    /// [`BookDepth`]: struct.BookDepth.html
    /// [`BookLevel`]: struct.BookLevel.html
    pub fn depth(&self, levels: usize, include_orders: bool) -> BookDepth {
        let mut asks: Vec<BookLevel> = Vec::with_capacity(levels);
        let mut bids: Vec<BookLevel> = Vec::with_capacity(levels);

        for (ask_price, queue) in self.asks.iter() {
            let mut qty = 0;
            for idx in queue {
                qty += self.arena[*idx].qty;
            }
            if qty > 0 {
                asks.push(BookLevel {
                    price: *ask_price,
                    qty,
                    orders: if include_orders { queue.iter().map(|order_id| self.arena[*order_id].clone()).collect() } else { vec![]}
                });
            }
        }

        for (bid_price, queue) in self.bids.iter() {
            let mut qty = 0;
            for idx in queue {
                qty += self.arena[*idx].qty;
            }
            if qty > 0 {
                bids.push(BookLevel {
                    price: *bid_price,
                    qty,
                    orders: if include_orders { queue.iter().map(|order_id| self.arena[*order_id].clone()).collect() } else { vec![]}
                });
            }
        }

        BookDepth { levels, asks, bids }
    }

    /// Toggle the stats tracking on or off, depending on the `track` parameter.
    pub fn track_stats(&mut self, track: bool) {
        self.track_stats = track;
    }

    /// Execute an order, returning immediately an event indicating the result.
    pub fn execute(&mut self, event: OrderType) -> OrderEvent {
        let order_id = event.get_id();
        let order_type = event.get_type();
        
        // Having order id sequence to only increase is very important which helps in optimizing the order search during cancel.
        // and helps reconstructing the btreemaps orders from the hashmap 
        if order_type != "cancel" {
            if self.last_processed_order_id >=  order_id {
                return OrderEvent::Rejected { id: order_id, message: rejectmessages::INVALID_ORDER_NUMBER }
            }
            self.last_processed_order_id = order_id;
        }

        let event = self._execute(event);
        if !self.track_stats {
            return event;
        }

        match event.clone() {
            OrderEvent::Filled {
                id: _,
                filled_qty,
                fills,
            } => {
                self.traded_volume += filled_qty;
                // If we are here, fills is not empty, so it's safe to unwrap it
                let last_fill = fills.last().unwrap();
                self.last_trade = Some(Trade {
                    total_qty: filled_qty,
                    avg_price: fills
                        .iter()
                        .map(|fm| fm.price * fm.qty)
                        .sum::<u64>() as f64
                        / (filled_qty as f64),
                    last_qty: last_fill.qty,
                    last_price: last_fill.price,
                });
            }
            OrderEvent::PartiallyFilled {
                id: _,
                filled_qty,
                fills,
            } => {
                self.traded_volume += filled_qty;
                // If we are here, fills is not empty, so it's safe to unwrap it
                let last_fill = fills.last().unwrap();
                self.last_trade = Some(Trade {
                    total_qty: filled_qty,
                    avg_price: fills
                        .iter()
                        .map(|fm| fm.price * fm.qty)
                        .sum::<u64>() as f64
                        / (filled_qty as f64),
                    last_qty: last_fill.qty,
                    last_price: last_fill.price,
                });
            }
            _ => {}
        }
        event
    }

    fn _execute(&mut self, event: OrderType) -> OrderEvent {
        match event {
            OrderType::Market { id, user_id:_, side, qty } => {
                let (fills, partial, filled_qty) = self.market(id, side, qty);
                if fills.is_empty() {
                    OrderEvent::Rejected { id, message: LIQUIDITY_NOT_AVAILABLE  }
                } else if partial {
                    OrderEvent::PartiallyFilled {
                        id,
                        filled_qty,
                        fills,
                    }
                } else {
                    OrderEvent::Filled {
                        id,
                        filled_qty,
                        fills,
                    }
                }
            }
            OrderType::Limit { id, user_id, side, qty, price,} => {
                let (fills, partial, filled_qty) =
                    self.limit(id, user_id, side, qty, price);
                if fills.is_empty() {
                    OrderEvent::Open { id }
                } else if partial {
                    OrderEvent::PartiallyFilled {
                        id,
                        filled_qty,
                        fills,
                    }
                } else {
                    OrderEvent::Filled {
                        id,
                        filled_qty,
                        fills,
                    }
                }
            }
            OrderType::Cancel { id } => {
                self.cancel(id);
                OrderEvent::Canceled { id }
            }
        }
    }

    fn cancel(&mut self, id: OrderId) -> bool {
        if let Some(order) = self.arena.get(id) {
            if let Some(ref mut queue) = self.asks.get_mut(&order.price) {
                if let Some(i) = queue.iter().position(|i| *i == id) {
                    queue.remove(i);
                }
            }
            if let Some(ref mut queue) = self.bids.get_mut(&order.price) {
                if let Some(i) = queue.iter().position(|i| *i == id) {
                    queue.remove(i);
                }
            }
        }
        self.update_min_ask();
        self.update_max_bid();
        self.arena.delete(&id)
    }

    fn finalize_execution(&mut self, fills: &Vec<FillMetadata>) {
        fills.iter().for_each(|fill| {
            let maker_id = fill.order_2;
            let maker_side = !fill.taker_side;
            let qty = fill.qty;
            let remove_maker_order = fill.total_fill;
            let levels = if maker_side == Side::Bid { &mut self.bids } else { &mut self.asks };  
            let entry = levels.entry(fill.price).or_insert(Vec::with_capacity(self.default_queue_capacity));
            let index = entry.binary_search(&maker_id);
            if remove_maker_order {
                if let Ok(index) = index {
                    entry.remove(index);
                }
                self.arena.delete(&maker_id);
            } else { 
                self.arena[maker_id].qty -= qty;                
            }
        });
        self.update_max_bid();
        self.update_min_ask();
    }

    fn market(
        &mut self,
        id: OrderId,
        side: Side,
        qty: u64,
    ) -> (Vec<FillMetadata>, bool, u64) {
        let mut fills = Vec::new();

        let remaining_qty = match side {
            Side::Bid => self.match_with_asks(id, qty, &mut fills, None),
            Side::Ask => self.match_with_bids(id, qty, &mut fills, None),
        };
        self.finalize_execution(&fills);
        let partial = remaining_qty > 0;
        (fills, partial, qty - remaining_qty)
    }

    fn limit(
        &mut self,
        id: OrderId,
        user_id: UserId,
        side: Side,
        qty: u64,
        price: u64,
    ) -> (Vec<FillMetadata>, bool, u64) {
        let mut partial = false;
        let remaining_qty;
        let mut fills: Vec<FillMetadata> = Vec::new();

        match side {
            Side::Bid => {
                remaining_qty = self.match_with_asks(id, qty, &mut fills, Some(price));
                self.finalize_execution(&fills);
                if remaining_qty > 0 {
                    partial = true;
                    let queue_capacity = self.default_queue_capacity;
                    //mutation
                    self.arena.insert(id, user_id, price, remaining_qty);
                    self.bids
                        .entry(price)
                        .or_insert_with(|| Vec::with_capacity(queue_capacity))
                        .push(id);
                    if price > self.max_bid {
                        self.max_bid = price;
                    }
                }
            }
            Side::Ask => {
                remaining_qty = self.match_with_bids(id, qty, &mut fills, Some(price));
                self.finalize_execution(&fills);
                if remaining_qty > 0 {
                    partial = true;
                    self.arena.insert(id, user_id, price, remaining_qty);
                    let queue_capacity = self.default_queue_capacity;
                    self.asks
                        .entry(price)
                        .or_insert_with(|| Vec::with_capacity(queue_capacity))
                        .push(id);
                    if price < self.min_ask {
                        self.min_ask = price;
                    }
                }
            }
        }

        (fills, partial, qty - remaining_qty)
    }

    fn match_with_asks(
        &mut self,
        id: OrderId,
        qty: u64,
        fills: &mut Vec<FillMetadata>,
        limit_price: Option<u64>,
    ) -> u64 {
        let mut remaining_qty = qty;
        // let mut update_bid_ask = false;
        for (ask_price, queue) in self.asks.iter_mut() {
            if queue.is_empty() {
                continue;
            }
            // if (update_bid_ask || self.min_ask == u64::MAX) && !queue.is_empty() {
            //     self.min_ask = *ask_price;
            //     update_bid_ask = false;
            // }
            if let Some(lp) = limit_price {
                if lp < *ask_price {
                    break;
                }
            }
            if remaining_qty == 0 {
                break;
            }
            let filled_qty = Self::simulate_queue_fills(
                &self.arena,
                queue,
                remaining_qty,
                id,
                Side::Bid,
                fills,
            );
            // if queue.is_empty() {
            //     update_bid_ask = true;
            // }
            remaining_qty -= filled_qty;
        }

        // self.update_min_ask();
        remaining_qty
    }

    fn match_with_bids(
        &mut self,
        id: OrderId,
        qty: Qty,
        fills: &mut Vec<FillMetadata>,
        limit_price: Option<Price>,
    ) -> u64 {
        let mut remaining_qty = qty;
        // let mut update_bid_ask = false;
        for (bid_price, queue) in self.bids.iter_mut().rev() {
            if queue.is_empty() {
                continue;
            }
            // if (update_bid_ask || self.max_bid == 0) && !queue.is_empty() {
            //     self.max_bid = *bid_price;
            //     update_bid_ask = false;
            // }
            if let Some(lp) = limit_price {
                if lp > *bid_price {
                    break;
                }
            }
            if remaining_qty == 0 {
                break;
            }
            let filled_qty = Self::simulate_queue_fills(
                &self.arena,
                queue,
                remaining_qty,
                id,
                Side::Ask,
                fills,
            );
            // if queue.is_empty() {
            //     update_bid_ask = true;
            // }
            remaining_qty -= filled_qty;
        }

        // self.update_max_bid();
        remaining_qty
    }

    fn update_min_ask(&mut self) {
        let mut cur_asks = self.asks.iter().filter(|(_, q)| !q.is_empty());
        self.min_ask = cur_asks.next().map(|(p, _)| *p).unwrap_or(u64::MAX);
    }

    fn update_max_bid(&mut self) {
        let mut cur_bids =
            self.bids.iter().rev().filter(|(_, q)| !q.is_empty());
        self.max_bid = cur_bids.next().map(|(p, _)| *p).unwrap_or(0u64);
    }

    fn simulate_queue_fills(
        arena: &OrderArena,
        opposite_orders: &Vec<OrderId>,
        remaining_qty: u64,
        id: u64,
        side: Side,
        fills: &mut Vec<FillMetadata>,
    ) -> u64 {
        let mut qty_to_fill = remaining_qty;
        let mut filled_qty = 0;
        
        for (_, head_order_id) in opposite_orders.iter().enumerate() {
            if qty_to_fill == 0 {
                break;
            }
            let head_order = &arena[*head_order_id];
            let traded_price = head_order.price;
            let available_qty = head_order.qty;
            if available_qty == 0 {
                continue;
            }
            let traded_quantity: u64;
            let filled;

            if qty_to_fill >= available_qty {
                traded_quantity = available_qty;
                qty_to_fill -= available_qty;
                filled = true;
            } else {
                traded_quantity = qty_to_fill;
                qty_to_fill = 0;
                filled = false;
            }
            let fill = FillMetadata {
                order_1: id,
                order_2: head_order.id,
                qty: traded_quantity,
                price: traded_price,
                taker_side: side,
                total_fill: filled,
            };
            fills.push(fill);
            filled_qty += traded_quantity;
        }
        filled_qty
    }
}

#[cfg(test)]
mod test {
    use crate::{
        BookDepth, BookLevel, FillMetadata, OrderBook, OrderEvent, OrderType,
        Side, Trade, rejectmessages::LIQUIDITY_NOT_AVAILABLE, models::LimitOrder,
    };
    use std::collections::BTreeMap;

    const DEFAULT_QUEUE_SIZE: usize = 10;
    const BID_ASK_COMBINATIONS: [(Side, Side); 2] =
        [(Side::Bid, Side::Ask), (Side::Ask, Side::Bid)];

    // In general, floating point values cannot be compared for equality. That's
    // why we don't derive PartialEq in lobster::models, but we do it here for
    // our tests in some very specific cases.
    impl PartialEq for Trade {
        fn eq(&self, other: &Self) -> bool {
            self.total_qty == other.total_qty
                && (self.avg_price - other.avg_price).abs() < 1.0e-6
                && self.last_qty == other.last_qty
                && self.last_price == other.last_price
        }
    }

    fn init_ob(events: Vec<OrderType>) -> (OrderBook, Vec<OrderEvent>) {
        let mut ob = OrderBook::default();
        ob.track_stats(true);
        let mut results = Vec::new();
        for e in events {
            results.push(ob.execute(e));
        }
        (ob, results)
    }

    fn _init_book(orders: Vec<(u64, u64)>) -> BTreeMap<u64, Vec<u64>> {
        let mut bk = BTreeMap::new();
        for (p, i) in orders {
            bk.entry(p)
                .or_insert_with(|| Vec::with_capacity(DEFAULT_QUEUE_SIZE))
                .push(i);
        }
        bk
    }

    fn init_book(orders: Vec<(u64, u64)>) -> Vec<(u64, Vec<u64>)> {
        _init_book(orders).into_iter().collect()
    }

    fn init_book_holes(
        orders: Vec<(u64, u64)>,
        holes: Vec<u64>,
    ) -> Vec<(u64, Vec<u64>)> {
        let mut bk = _init_book(orders);
        for h in holes {
            bk.insert(h, Vec::new());
        }
        bk.into_iter().collect()
    }

    #[test]
    fn empty_book() {
        let (ob, results) = init_ob(Vec::new());
        assert_eq!(results, Vec::new());
        assert_eq!(ob.min_ask(), u64::MAX);
        assert_eq!(ob.max_bid(), 0u64);
        assert_eq!(ob._asks(), Vec::new());
        assert_eq!(ob._bids(), Vec::new());
        assert_eq!(ob.spread(), u64::MAX);
        assert_eq!(ob.traded_volume(), 0);
        assert_eq!(
            ob.depth(2, false),
            BookDepth {
                levels: 2,
                asks: Vec::new(),
                bids: Vec::new()
            }
        );
        assert_eq!(ob.last_trade(), None);
    }

    #[test]
    fn one_resting_order() {
        for (bid_ask, _) in &BID_ASK_COMBINATIONS {
            let (ob, results) = init_ob(vec![OrderType::Limit {
                id: 1,
                user_id: 1,
                side: *bid_ask,
                qty: 12,
                price: 395,
            }]);
            assert_eq!(results, vec![OrderEvent::Open { id: 1 }]);
            if *bid_ask == Side::Bid {
                assert_eq!(ob.min_ask(), u64::MAX);
                assert_eq!(ob.max_bid(), 395);
                assert_eq!(ob._asks(), Vec::new());
                assert_eq!(ob._bids(), init_book(vec![(395, 1)]));
                assert_eq!(ob.spread(), u64::MAX - 395);
                assert_eq!(ob.traded_volume(), 0);
                assert_eq!(
                    ob.depth(3, false),
                    BookDepth {
                        levels: 3,
                        asks: Vec::new(),
                        bids: vec![BookLevel {
                            price: 395,
                            qty: 12,
                            orders: vec![]
                        }],
                    }
                );
                assert_eq!(ob.last_trade(), None);
            } else {
                assert_eq!(ob.min_ask(), 395);
                assert_eq!(ob.max_bid(), 0u64);
                assert_eq!(ob._asks(), init_book(vec![(395, 1)]));
                assert_eq!(ob._bids(), Vec::new());
                assert_eq!(ob.spread(), 395);
                assert_eq!(ob.traded_volume(), 0);
                assert_eq!(
                    ob.depth(4, false),
                    BookDepth {
                        levels: 4,
                        asks: vec![BookLevel {
                            price: 395,
                            qty: 12,
                            orders: vec![]
                        }],
                        bids: Vec::new()
                    }
                );
                assert_eq!(ob.last_trade(), None);
            }
        }
    }

    #[test]
    fn two_resting_orders() {
        for (bid_ask, ask_bid) in &BID_ASK_COMBINATIONS {
            let (ob, results) = init_ob(vec![
                OrderType::Limit {
                    user_id: 1,
                    id: 1,
                    side: *bid_ask,
                    qty: 12,
                    price: 395,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 2,
                    side: *ask_bid,
                    qty: 2,
                    price: 398,
                },
            ]);
            if *bid_ask == Side::Bid {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Open { id: 2 }
                    ]
                );
                assert_eq!(ob.min_ask(), 398);
                assert_eq!(ob.max_bid(), 395);
                assert_eq!(ob._asks(), init_book(vec![(398, 2)]));
                assert_eq!(ob._bids(), init_book(vec![(395, 1)]));
                assert_eq!(ob.spread(), 3);
                assert_eq!(ob.traded_volume(), 0);
                assert_eq!(
                    ob.depth(4, false),
                    BookDepth {
                        levels: 4,
                        asks: vec![BookLevel { price: 398, qty: 2, orders: vec![] }],
                        bids: vec![BookLevel {
                            price: 395,
                            qty: 12,
                            orders: vec![]
                        }],
                    }
                );
                assert_eq!(ob.last_trade(), None);
            } else {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Filled {
                            id: 2,
                            filled_qty: 2,
                            fills: vec![FillMetadata {
                                order_1: 2,
                                order_2: 1,
                                qty: 2,
                                price: 395,
                                taker_side: *ask_bid,
                                total_fill: false,
                            }],
                        }
                    ]
                );
                assert_eq!(ob.min_ask(), 395);
                assert_eq!(ob.max_bid(), 0u64);
                assert_eq!(ob._asks(), init_book(vec![(395, 1)]));
                assert_eq!(ob._bids(), init_book(vec![]));
                assert_eq!(ob.spread(), 395);
                assert_eq!(ob.traded_volume(), 2);
                assert_eq!(
                    ob.depth(4, false),
                    BookDepth {
                        levels: 4,
                        asks: vec![BookLevel {
                            price: 395,
                            qty: 10,
                            orders: vec![]
                        }],
                        bids: Vec::new(),
                    }
                );
                assert_eq!(
                    ob.last_trade(),
                    Some(Trade {
                        total_qty: 2,
                        avg_price: 395.0,
                        last_qty: 2,
                        last_price: 395,
                    })
                );
            }
        }
    }

    #[test]
    fn two_resting_orders_merged() {
        for (bid_ask, _) in &BID_ASK_COMBINATIONS {
            let (ob, results) = init_ob(vec![
                OrderType::Limit {
                    user_id: 1,
                    id: 1,
                    side: *bid_ask,
                    qty: 12,
                    price: 395,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 2,
                    side: *bid_ask,
                    qty: 2,
                    price: 395,
                },
            ]);
            assert_eq!(
                results,
                vec![
                    OrderEvent::Open { id: 1 },
                    OrderEvent::Open { id: 2 }
                ]
            );
            if *bid_ask == Side::Bid {
                assert_eq!(ob.min_ask(), u64::MAX);
                assert_eq!(ob.max_bid(), 395);
                assert_eq!(ob._asks(), Vec::new());
                assert_eq!(
                    ob._bids(),
                    init_book(vec![(395, 1), (395, 2)])
                );
                assert_eq!(ob.spread(), u64::MAX-395);
                assert_eq!(ob.traded_volume(), 0);
                assert_eq!(
                    ob.depth(3, false),
                    BookDepth {
                        levels: 3,
                        asks: Vec::new(),
                        bids: vec![BookLevel {
                            price: 395,
                            qty: 14,
                            orders: vec![]
                        }],
                    }
                );
                assert_eq!(ob.last_trade(), None);
            } else {
                assert_eq!(ob.min_ask(), 395);
                assert_eq!(ob.max_bid(), 0u64);
                assert_eq!(
                    ob._asks(),
                    init_book(vec![(395, 1), (395, 2)])
                );
                assert_eq!(ob._bids(), Vec::new());
                assert_eq!(ob.spread(), 395);
                assert_eq!(ob.traded_volume(), 0);
                assert_eq!(
                    ob.depth(3, false),
                    BookDepth {
                        levels: 3,
                        asks: vec![BookLevel {
                            price: 395,
                            qty: 14,
                            orders: vec![]
                        }],
                        bids: Vec::new(),
                    }
                );
                assert_eq!(ob.last_trade(), None);
            }
        }
    }

    #[test]
    fn two_resting_orders_stacked() {
        for (bid_ask, _) in &BID_ASK_COMBINATIONS {
            let (ob, results) = init_ob(vec![
                OrderType::Limit {
                    user_id: 1,
                    id: 1,
                    side: *bid_ask,
                    qty: 12,
                    price: 395,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 2,
                    side: *bid_ask,
                    qty: 2,
                    price: 398,
                },
            ]);
            assert_eq!(
                results,
                vec![
                    OrderEvent::Open { id: 1 },
                    OrderEvent::Open { id: 2 }
                ]
            );
            if *bid_ask == Side::Bid {
                assert_eq!(ob.min_ask(), u64::MAX);
                assert_eq!(ob.max_bid(), 398);
                assert_eq!(ob._asks(), Vec::new());
                assert_eq!(
                    ob._bids(),
                    init_book(vec![(398, 2), (395, 1)])
                );
                assert_eq!(ob.spread(), u64::MAX - 398);
            } else {
                assert_eq!(ob.min_ask(), 395);
                assert_eq!(ob.max_bid(), 0u64);
                assert_eq!(
                    ob._asks(),
                    init_book(vec![(398, 2), (395, 1)])
                );
                assert_eq!(ob._bids(), Vec::new());
                assert_eq!(ob.spread(), 395);
            }
        }
    }

    #[test]
    fn three_resting_orders_stacked() {
        for (bid_ask, ask_bid) in &BID_ASK_COMBINATIONS {
            let (ob, results) = init_ob(vec![
                OrderType::Limit {
                    user_id: 1,
                    id: 1,
                    side: *bid_ask,
                    qty: 12,
                    price: 395,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 2,
                    side: *ask_bid,
                    qty: 2,
                    price: 399,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 3,
                    side: *bid_ask,
                    qty: 2,
                    price: 398,
                },
            ]);
            if *bid_ask == Side::Bid {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Open { id: 2 },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(ob.min_ask(), 399);
                assert_eq!(ob.max_bid(), 398);
                assert_eq!(ob._asks(), init_book(vec![(399, 2)]));
                assert_eq!(
                    ob._bids(),
                    init_book(vec![(398, 3), (395, 1)])
                );
                assert_eq!(ob.spread(), 1);
            } else {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Filled {
                            id: 2,
                            filled_qty: 2,
                            fills: vec![FillMetadata {
                                order_1: 2,
                                order_2: 1,
                                qty: 2,
                                price: 395,
                                taker_side: *ask_bid,
                                total_fill: false,
                            }],
                        },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(ob.min_ask(), 395);
                assert_eq!(ob.max_bid(), 0);
                assert_eq!(
                    ob._asks(),
                    init_book(vec![(398, 3), (395, 1)])
                );
                assert_eq!(ob._bids(), init_book(vec![]));
                assert_eq!(ob.spread(), 395);
            }
        }
    }

    #[test]
    fn crossing_limit_order_partial() {
        for (bid_ask, ask_bid) in &BID_ASK_COMBINATIONS {
            let (mut ob, results) = init_ob(vec![
                OrderType::Limit {
                    user_id: 1,
                    id: 1,
                    side: *bid_ask,
                    qty: 12,
                    price: 395,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 2,
                    side: *ask_bid,
                    qty: 2,
                    price: 399,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 3,
                    side: *bid_ask,
                    qty: 2,
                    price: 398,
                },
            ]);
            let result = ob.execute(OrderType::Limit {
                user_id: 1,
                id: 4,
                side: *ask_bid,
                qty: 1,
                price: 397,
            });

            if *bid_ask == Side::Bid {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Open { id: 2 },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(
                    result,
                    OrderEvent::Filled {
                        id: 4,
                        filled_qty: 1,
                        fills: vec![FillMetadata {
                            order_1: 4,
                            order_2: 3,
                            qty: 1,
                            price: 398,
                            taker_side: *ask_bid,
                            total_fill: false,
                        }]
                    }
                );
                assert_eq!(ob.min_ask(), 399);
                assert_eq!(ob.max_bid(), 398);
                assert_eq!(ob._asks(), init_book(vec![(399, 2)]));
                assert_eq!(
                    ob._bids(),
                    init_book(vec![(398, 3), (395, 1)])
                );
                assert_eq!(ob.spread(), 1);
            } else {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Filled {
                            id: 2,
                            filled_qty: 2,
                            fills: vec![FillMetadata {
                                order_1: 2,
                                order_2: 1,
                                qty: 2,
                                price: 395,
                                taker_side: *ask_bid,
                                total_fill: false,
                            }],
                        },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(
                    result,
                    OrderEvent::Filled {
                        id: 4,
                        filled_qty: 1,
                        fills: vec![FillMetadata {
                            order_1: 4,
                            order_2: 1,
                            qty: 1,
                            price: 395,
                            taker_side: *ask_bid,
                            total_fill: false,
                        }]
                    }
                );
                assert_eq!(ob.min_ask(), 395);
                assert_eq!(ob.max_bid(), 0);
                assert_eq!(
                    ob._asks(),
                    init_book(vec![(398, 3), (395, 1)])
                );
                assert_eq!(ob._bids(), init_book(vec![]));
                assert_eq!(ob.spread(), 395);
            }
        }
    }

    #[test]
    fn crossing_limit_order_matching() {
        for (bid_ask, ask_bid) in &BID_ASK_COMBINATIONS {
            let (mut ob, results) = init_ob(vec![
                OrderType::Limit {
                    user_id: 1,
                    id: 1,
                    side: *bid_ask,
                    qty: 12,
                    price: 395,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 2,
                    side: *ask_bid,
                    qty: 2,
                    price: 399,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 3,
                    side: *bid_ask,
                    qty: 2,
                    price: 398,
                },
            ]);
            let result = ob.execute(OrderType::Limit {
                user_id: 1,
                id: 4,
                side: *ask_bid,
                qty: 2,
                price: 397,
            });

            if *bid_ask == Side::Bid {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Open { id: 2 },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(
                    result,
                    OrderEvent::Filled {
                        id: 4,
                        filled_qty: 2,
                        fills: vec![FillMetadata {
                            order_1: 4,
                            order_2: 3,
                            qty: 2,
                            price: 398,
                            taker_side: *ask_bid,
                            total_fill: true,
                        }]
                    }
                );
                assert_eq!(ob.min_ask(), 399);
                assert_eq!(ob.max_bid(), 395);
                assert_eq!(ob._asks(), init_book(vec![(399, 2)]));
                assert_eq!(
                    ob._bids(),
                    init_book_holes(vec![(395, 1)], vec![398])
                );
                assert_eq!(ob.spread(), 4);
            } else {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Filled {
                            id: 2,
                            filled_qty: 2,
                            fills: vec![FillMetadata {
                                order_1: 2,
                                order_2: 1,
                                qty: 2,
                                price: 395,
                                taker_side: *ask_bid,
                                total_fill: false,
                            }],
                        },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(
                    result,
                    OrderEvent::Filled {
                        id: 4,
                        filled_qty: 2,
                        fills: vec![FillMetadata {
                            order_1: 4,
                            order_2: 1,
                            qty: 2,
                            price: 395,
                            taker_side: *ask_bid,
                            total_fill: false,
                        }]
                    }
                );
                assert_eq!(ob.min_ask(), 395);
                assert_eq!(ob.max_bid(), 0u64);
                assert_eq!(
                    ob._asks(),
                    init_book(vec![(395, 1), (398, 3)])
                );
                assert_eq!(ob._bids(), init_book(vec![]));
                assert_eq!(ob.spread(), 395);
            }
        }
    }

    #[test]
    fn crossing_limit_order_over() {
        for (bid_ask, ask_bid) in &BID_ASK_COMBINATIONS {
            let (mut ob, results) = init_ob(vec![
                OrderType::Limit {
                    user_id: 1,
                    id: 1,
                    side: *bid_ask,
                    qty: 12,
                    price: 395,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 2,
                    side: *ask_bid,
                    qty: 2,
                    price: 399,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 3,
                    side: *bid_ask,
                    qty: 2,
                    price: 398,
                },
            ]);
            let result = ob.execute(OrderType::Limit {
                user_id: 1,
                id: 4,
                side: *ask_bid,
                qty: 5,
                price: 397,
            });

            if *bid_ask == Side::Bid {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Open { id: 2 },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(
                    result,
                    OrderEvent::PartiallyFilled {
                        id: 4,
                        filled_qty: 2,
                        fills: vec![FillMetadata {
                            order_1: 4,
                            order_2: 3,
                            qty: 2,
                            price: 398,
                            taker_side: *ask_bid,
                            total_fill: true,
                        }]
                    }
                );
                assert_eq!(ob.min_ask(), 397);
                assert_eq!(ob.max_bid(), 395);
                assert_eq!(
                    ob._asks(),
                    init_book(vec![(399, 2), (397, 4)])
                );
                assert_eq!(
                    ob._bids(),
                    init_book_holes(vec![(395, 1)], vec![398])
                );
                assert_eq!(ob.spread(), 2);
            } else {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Filled {
                            id: 2,
                            filled_qty: 2,
                            fills: vec![FillMetadata {
                                order_1: 2,
                                order_2: 1,
                                qty: 2,
                                price: 395,
                                taker_side: *ask_bid,
                                total_fill: false,
                            }],
                        },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(
                    result,
                    OrderEvent::Filled {
                        id: 4,
                        filled_qty: 5,
                        fills: vec![FillMetadata {
                            order_1: 4,
                            order_2: 1,
                            qty: 5,
                            price: 395,
                            taker_side: *ask_bid,
                            total_fill: false,
                        }]
                    }
                );
                assert_eq!(ob.min_ask(), 395);
                assert_eq!(ob.max_bid(), 0);
                assert_eq!(
                    ob._asks(),
                    init_book(vec![(395, 1), (398, 3)])
                );
                assert_eq!(ob._bids(), init_book(vec![]));
                assert_eq!(ob.spread(), 395);
            }
        }
    }

    #[test]
    fn market_order_rejected() {
        for (_, ask_bid) in &BID_ASK_COMBINATIONS {
            let (mut ob, _) = init_ob(vec![]);
            let result = ob.execute(OrderType::Market {
                user_id: 1,
                id: 1,
                side: *ask_bid,
                qty: 5,
            });

            assert_eq!(result, OrderEvent::Rejected { id: 1, message: LIQUIDITY_NOT_AVAILABLE });
        }
    }

    #[test]
    fn market_order_partially_filled() {
        for (bid_ask, ask_bid) in &BID_ASK_COMBINATIONS {
            let (mut ob, results) = init_ob(vec![
                OrderType::Limit {
                    user_id: 1,
                    id: 1,
                    side: *bid_ask,
                    qty: 12,
                    price: 395,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 2,
                    side: *ask_bid,
                    qty: 2,
                    price: 399,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 3,
                    side: *bid_ask,
                    qty: 2,
                    price: 398,
                },
            ]);
            let result = ob.execute(OrderType::Market {
                user_id: 1,
                id: 4,
                side: *ask_bid,
                qty: 15,
            });

            if *bid_ask == Side::Bid {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Open { id: 2 },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(
                    result,
                    OrderEvent::PartiallyFilled {
                        id: 4,
                        filled_qty: 14,
                        fills: vec![
                            FillMetadata {
                                order_1: 4,
                                order_2: 3,
                                qty: 2,
                                price: 398,
                                taker_side: *ask_bid,
                                total_fill: true,
                            },
                            FillMetadata {
                                order_1: 4,
                                order_2: 1,
                                qty: 12,
                                price: 395,
                                taker_side: *ask_bid,
                                total_fill: true,
                            }
                        ]
                    }
                );
                assert_eq!(ob.min_ask(), 399);
                assert_eq!(ob.max_bid(), 0);
                assert_eq!(ob._asks(), init_book(vec![(399, 2)]));
                assert_eq!(ob._bids(), init_book_holes(vec![], vec![395, 398]));
                assert_eq!(ob.spread(), 399);
            } else {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Filled {
                            id: 2,
                            filled_qty: 2,
                            fills: vec![FillMetadata {
                                order_1: 2,
                                order_2: 1,
                                qty: 2,
                                price: 395,
                                taker_side: *ask_bid,
                                total_fill: false,
                            }],
                        },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(
                    result,
                    OrderEvent::PartiallyFilled {
                        id: 4,
                        filled_qty: 12,
                        fills: vec![
                            FillMetadata {
                                order_1: 4,
                                order_2: 1,
                                qty: 10,
                                price: 395,
                                taker_side: *ask_bid,
                                total_fill: true,
                            },
                            FillMetadata {
                                order_1: 4,
                                order_2: 3,
                                qty: 2,
                                price: 398,
                                taker_side: *ask_bid,
                                total_fill: true,
                            }
                        ]
                    }
                );
                assert_eq!(ob.min_ask(), u64::MAX);
                assert_eq!(ob.max_bid(), 0);
                assert_eq!(ob._asks(), init_book_holes(vec![], vec![395, 398]));
                assert_eq!(ob._bids(), init_book(vec![]));
                assert_eq!(ob.spread(), u64::MAX);
            }
        }
    }

    #[test]
    fn market_order_filled() {
        for (bid_ask, ask_bid) in &BID_ASK_COMBINATIONS {
            let (mut ob, results) = init_ob(vec![
                OrderType::Limit {
                    user_id: 1,
                    id: 1,
                    side: *bid_ask,
                    qty: 12,
                    price: 395,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 2,
                    side: *ask_bid,
                    qty: 2,
                    price: 399,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 3,
                    side: *bid_ask,
                    qty: 2,
                    price: 398,
                },
            ]);
            let result = ob.execute(OrderType::Market {
                user_id: 1,
                id: 4,
                side: *ask_bid,
                qty: 7,
            });

            if *bid_ask == Side::Bid {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Open { id: 2 },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(
                    result,
                    OrderEvent::Filled {
                        id: 4,
                        filled_qty: 7,
                        fills: vec![
                            FillMetadata {
                                order_1: 4,
                                order_2: 3,
                                qty: 2,
                                price: 398,
                                taker_side: *ask_bid,
                                total_fill: true,
                            },
                            FillMetadata {
                                order_1: 4,
                                order_2: 1,
                                qty: 5,
                                price: 395,
                                taker_side: *ask_bid,
                                total_fill: false,
                            }
                        ]
                    }
                );
                assert_eq!(ob.min_ask(), 399);
                assert_eq!(ob.max_bid(), 395);
                assert_eq!(ob._asks(), init_book(vec![(399, 2)]));
                assert_eq!(
                    ob._bids(),
                    init_book_holes(vec![(395, 1)], vec![398])
                );
                assert_eq!(ob.spread(), 4);
                assert_eq!(ob.arena.get(3), None);
                assert_eq!(ob.arena.get(1), Some(&LimitOrder{ user_id: 1, id: 1, qty: 7, price: 395 }));
            } else {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Filled {
                            id: 2,
                            filled_qty: 2,
                            fills: vec![FillMetadata {
                                order_1: 2,
                                order_2: 1,
                                qty: 2,
                                price: 395,
                                taker_side: *ask_bid,
                                total_fill: false,
                            }],
                        },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(
                    result,
                    OrderEvent::Filled {
                        id: 4,
                        filled_qty: 7,
                        fills: vec![FillMetadata {
                            order_1: 4,
                            order_2: 1,
                            qty: 7,
                            price: 395,
                            taker_side: *ask_bid,
                            total_fill: false,
                        }]
                    }
                );
                assert_eq!(ob.min_ask(), 395);
                assert_eq!(ob.max_bid(), 0);
                assert_eq!(
                    ob._asks(),
                    init_book(vec![(395, 1), (398, 3)])
                );
                assert_eq!(ob._bids(), init_book(vec![]));
                assert_eq!(ob.spread(), 395);
                assert_eq!(ob.arena.get(3), Some(&LimitOrder { user_id: 1, id: 3, qty: 2, price: 398 }));
                assert_eq!(ob.arena.get(1), Some(&LimitOrder{ user_id: 1, id: 1, qty: 3, price: 395 }));
            }
        }
    }

    #[test]
    fn cancel_non_existing_order() {
        let (mut ob, _) = init_ob(vec![]);
        let result = ob.execute(OrderType::Cancel { id: 0 });
        assert_eq!(result, OrderEvent::Canceled { id: 0 });
        assert_eq!(ob.min_ask(), u64::MAX);
        assert_eq!(ob.max_bid(), 0);
        assert_eq!(ob._asks(), Vec::new());
        assert_eq!(ob._bids(), Vec::new());
        assert_eq!(ob.spread(), u64::MAX);
        assert_eq!(ob.arena.get(0), None);
    }

    #[test]
    fn cancel_resting_order() {
        for (bid_ask, _) in &BID_ASK_COMBINATIONS {
            let (mut ob, results) = init_ob(vec![OrderType::Limit {
                user_id: 1,
                id: 1,
                side: *bid_ask,
                qty: 12,
                price: 395,
            }]);
            let result = ob.execute(OrderType::Cancel { id: 1 });
            assert_eq!(results, vec![OrderEvent::Open { id: 1 }]);
            assert_eq!(result, OrderEvent::Canceled { id: 1 });
            assert_eq!(ob.min_ask(), u64::MAX);
            assert_eq!(ob.max_bid(), 0);
            if *bid_ask == Side::Bid {
                assert_eq!(ob._asks(), Vec::new());
                assert_eq!(ob._bids(), init_book_holes(vec![], vec![395]));
            } else {
                assert_eq!(ob._asks(), init_book_holes(vec![], vec![395]));
                assert_eq!(ob._bids(), Vec::new());
            }
            assert_eq!(ob.spread(), u64::MAX);
            assert_eq!(ob.arena.get(1), None);
        }
    }

    #[test]
    fn cancel_resting_order_of_many() {
        for (bid_ask, ask_bid) in &BID_ASK_COMBINATIONS {
            let (mut ob, results) = init_ob(vec![
                OrderType::Limit {
                    user_id: 1,
                    id: 1,
                    side: *bid_ask,
                    qty: 12,
                    price: 395,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 2,
                    side: *ask_bid,
                    qty: 2,
                    price: 399,
                },
                OrderType::Limit {
                    user_id: 1,
                    id: 3,
                    side: *bid_ask,
                    qty: 2,
                    price: 398,
                },
            ]);
            let result = ob.execute(OrderType::Cancel { id: 1 });
            if *bid_ask == Side::Bid {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Open { id: 2 },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(result, OrderEvent::Canceled { id: 1 });
                assert_eq!(ob.min_ask(), 399);
                assert_eq!(ob.max_bid(), 398);
                assert_eq!(ob._asks(), init_book(vec![(399, 2)]));
                assert_eq!(
                    ob._bids(),
                    init_book_holes(vec![(398, 3)], vec![395])
                );
                assert_eq!(ob.spread(), 1);
            } else {
                assert_eq!(
                    results,
                    vec![
                        OrderEvent::Open { id: 1 },
                        OrderEvent::Filled {
                            id: 2,
                            filled_qty: 2,
                            fills: vec![FillMetadata {
                                order_1: 2,
                                order_2: 1,
                                qty: 2,
                                price: 395,
                                taker_side: *ask_bid,
                                total_fill: false,
                            }],
                        },
                        OrderEvent::Open { id: 3 }
                    ]
                );
                assert_eq!(result, OrderEvent::Canceled { id: 1 });
                assert_eq!(ob.min_ask(), 398);
                assert_eq!(ob.max_bid(), 0);
                assert_eq!(
                    ob._asks(),
                    init_book_holes(vec![(398, 3)], vec![395])
                );
                assert_eq!(ob._bids(), init_book(vec![]));
                assert_eq!(ob.spread(), 398);
            }
        }
    }
}
