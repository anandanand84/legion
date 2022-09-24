use std::collections::HashMap;
use std::ops::{Index, IndexMut};

use crate::models::LimitOrder;

#[derive(Debug)]
pub struct OrderArena {
    order_map: HashMap<u64, LimitOrder>,
}

impl OrderArena {
    pub fn new(capacity: usize) -> Self {
        Self {
            order_map: HashMap::with_capacity(capacity),
        }
    }

    pub fn get(&self, id: u64) -> Option<&LimitOrder> {
        self.order_map.get(&id)
    }

    pub fn insert(&mut self, id: u64, price: u64, qty: u64) {
        self.order_map.insert(id, LimitOrder { id: id, qty, price });
    }

    pub fn delete(&mut self, id: &u64) -> bool {
        self.order_map.remove(id).map_or(false, |_| true)
    }
}

impl Index<u64> for OrderArena {
    type Output = LimitOrder;

    #[inline]
    fn index(&self, id: u64) -> &LimitOrder {
        &self.order_map.get(&id).unwrap()
    }
}

impl IndexMut<u64> for OrderArena {
    #[inline]
    fn index_mut(&mut self, id: u64) -> &mut LimitOrder {
        self.order_map.get_mut(&id).unwrap()
    }
}
