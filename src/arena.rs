use std::collections::HashMap;
use std::ops::{Index, IndexMut};

use crate::models::LimitOrder;

#[derive(Debug)]
pub struct OrderArena {
    order_map: HashMap<u128, LimitOrder>,
}

impl OrderArena {
    pub fn new(capacity: usize) -> Self {
        let mut list = Self {
            order_map: HashMap::with_capacity(capacity),
        };
        list
    }

    pub fn get(&self, id: u128) -> Option<&LimitOrder> {
        self.order_map.get(&id)
    }

    #[cfg(test)]
    pub fn get_full(&self, id: u128) -> Option<(u128, u64, u64)> {
        self.order_map
            .get(&id)
            .map(|order| (order.id, order.price, order.qty))
    }

    pub fn insert(&mut self, id: u128, price: u64, qty: u64) {
        self.order_map.insert(id, LimitOrder { id: id, qty, price });
    }

    pub fn delete(&mut self, id: &u128) -> bool {
        self.order_map.remove(id).map_or(false, |x| true)
    }
}

impl Index<u128> for OrderArena {
    type Output = LimitOrder;

    #[inline]
    fn index(&self, id: u128) -> &LimitOrder {
        &self.order_map.get(&id).unwrap()
    }
}

impl IndexMut<u128> for OrderArena {
    #[inline]
    fn index_mut(&mut self, id: u128) -> &mut LimitOrder {
        self.order_map.get_mut(&id).unwrap()
    }
}

// #[cfg(test)]
// mod test {
//     use super::OrderArena;

//     #[test]
//     fn growing_arena() {
//         // All the integer casting below is necessary because we are using the
//         // indices to compute the prices. It's a contrived example and the size
//         // casts do not result in overflows.
//         //
//         // This test also addresses a bug that only occurred after all the
//         // pre-allocated limit orders were used. The new limit orders would be
//         // created with a swapped quantity and price, which unfortunately have
//         // the same type (u64) and the compiler could not catch that bug.
//         for capacity in 0_u64..30 {
//             let mut arena = OrderArena::new(capacity as usize);
//             for i in 0_u64..capacity {
//                 arena.insert(i as u128, i * 100 + i, 2 * i);
//             }
//             for i in 0_u64..capacity {
//                 assert_eq!(
//                     arena.get_full(i as u128),
//                     Some((i * 100 + i, 2 * i, (capacity - i) as usize - 1))
//                 );
//             }
//             for i in capacity..2 * capacity {
//                 assert_eq!(arena.get_full(i as u128), None);
//             }
//             for i in capacity..2 * capacity {
//                 arena.insert(i as u128, i * 100 + i, 2 * i);
//             }
//             for i in 0..capacity {
//                 assert_eq!(
//                     arena.get_full(i as u128),
//                     Some((i * 100 + i, 2 * i, (capacity - i) as usize - 1))
//                 );
//             }
//             for i in capacity..2 * capacity {
//                 assert_eq!(
//                     arena.get_full(i as u128),
//                     Some((i * 100 + i, 2 * i, i as usize,))
//                 );
//             }
//         }
//     }
// }
