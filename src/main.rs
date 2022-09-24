use legion::OrderBook;



fn main() {
    let mut book = OrderBook::default();
    let event = book.execute(legion::OrderType::Limit { id: 1, side: legion::Side::Bid, qty: 100, price: 199 });
    let event2 = book.execute(legion::OrderType::Market { id: 1, side: legion::Side::Ask, qty: 100 });
    println!("{:?}",event);
    println!("{:?}",event2);
}
