use orderbooklib::{dbgp, OrderBook, OrderStatus, Side};
use rand::Rng;
fn main() {
    println!("Creating new Orderbook");
    let mut ob = OrderBook::new("BTC".to_string());
    let mut rng = rand::thread_rng();
    for _ in 1..100000 {
        ob.add_limit_order(Side::Bid, rng.gen_range(1..5000), rng.gen_range(1..=500));
    }
    //dbgp!("{:#?}", ob);
    println!("Done adding orders, Starting to fill");

    for _ in 1..10 {
        for _ in 1..10000 {
            let fr = ob.add_limit_order(Side::Ask, rng.gen_range(1..5000), rng.gen_range(1..=500));
            // if matches! {fr.status, OrderStatus::Filled} {
            //     dbgp!("{:#?}, avg_fill_price {}", fr, fr.avg_fill_price());
            // }
        }

    }
    println!("Done!");
    ob.get_bbo();
}
