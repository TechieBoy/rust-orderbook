use rand::Rng;
use std::collections::{BTreeMap, HashMap, VecDeque};

// Change BTreeMap to Vec?
// Tests
// README/Blog

// Removed from Open source version
// Price qty serialize
// Deserialize Binance/FTX api endpoints.
// Log list of trades done.
#[macro_export]
macro_rules! dbgp {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
}
#[derive(Debug)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug)]
pub enum OrderStatus {
    Uninitialized,
    Created,
    Filled,
    PartiallyFilled,
}
#[derive(Debug)]
pub struct FillResult {
    // Orders filled (qty, price)
    pub filled_orders: Vec<(u64, u64)>,
    pub remaining_qty: u64,
    pub status: OrderStatus,
}

impl FillResult {
    fn new() -> Self {
        FillResult {
            filled_orders: Vec::new(),
            remaining_qty: u64::MAX,
            status: OrderStatus::Uninitialized,
        }
    }

    pub fn avg_fill_price(&self) -> f32 {
        let mut total_price_paid = 0;
        let mut total_qty = 0;
        for (q, p) in &self.filled_orders {
            total_price_paid += p * q;
            total_qty += q;
        }
        return total_price_paid as f32 / total_qty as f32;
    }
}

#[derive(Debug)]
pub struct Order {
    pub order_id: u64,
    pub qty: u64,
}
#[derive(Debug)]
struct HalfBook {
    s: Side,
    price_map: BTreeMap<u64, usize>,
    price_levels: Vec<VecDeque<Order>>,
}

impl HalfBook {
    pub fn new(s: Side) -> Self {
        HalfBook {
            s,
            price_map: BTreeMap::new(),
            price_levels: Vec::with_capacity(50_000),
        }
    }

    pub fn get_total_qty(&self, price: u64) -> u64 {
        self.price_levels[self.price_map[&price]]
            .iter()
            .map(|s| s.qty)
            .sum()
    }
}

// TODO: Make bid and offer price Option types
#[derive(Debug)]
pub struct OrderBook {
    symbol: String,
    best_bid_price: u64,
    best_offer_price: u64,
    bid_book: HalfBook,
    ask_book: HalfBook,
    // For fast cancels Order id -> (Side, Price_level)
    order_loc: HashMap<u64, (Side, usize)>,
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        OrderBook {
            symbol,
            best_bid_price: u64::MIN,
            best_offer_price: u64::MAX,
            bid_book: HalfBook::new(Side::Bid),
            ask_book: HalfBook::new(Side::Ask),
            order_loc: HashMap::with_capacity(50_000),
        }
    }

    pub fn cancel_order(&mut self, order_id: u64) -> Result<&str, &str> {
        if let Some((side, price_level)) = self.order_loc.get(&order_id) {
            let currdeque = match side {
                Side::Bid => self.bid_book.price_levels.get_mut(*price_level).unwrap(),
                Side::Ask => self.ask_book.price_levels.get_mut(*price_level).unwrap(),
            };
            currdeque.retain(|x| x.order_id != order_id);
            self.order_loc.remove(&order_id);
            Ok("Successfully cancelled order")
        } else {
            Err("No such order id")
        }
    }

    fn create_new_limit_order(&mut self, s: Side, price: u64, qty: u64) -> u64 {
        let mut rng = rand::thread_rng();
        let order_id: u64 = rng.gen();
        let book = match s {
            Side::Ask => &mut self.ask_book,
            Side::Bid => &mut self.bid_book,
        };
        let order = Order { order_id, qty };

        if let Some(val) = book.price_map.get(&price) {
            book.price_levels[*val].push_back(order);
            self.order_loc.insert(order_id, (s, *val));
        } else {
            let new_loc = book.price_levels.len();
            book.price_map.insert(price, new_loc);
            let mut vec_deq = VecDeque::new();
            vec_deq.push_back(order);
            book.price_levels.push(vec_deq);
            self.order_loc.insert(order_id, (s, new_loc));
        }
        order_id
    }

    fn update_bbo(&mut self) {
        for (p, u) in self.bid_book.price_map.iter().rev() {
            if !self.bid_book.price_levels[*u].is_empty() {
                self.best_bid_price = *p;
                break;
            }
        }
        for (p, u) in self.ask_book.price_map.iter() {
            if !self.ask_book.price_levels[*u].is_empty() {
                self.best_offer_price = *p;
                break;
            }
        }
    }

    pub fn add_limit_order(&mut self, s: Side, price: u64, order_qty: u64) -> FillResult {
        fn match_at_price_level(
            price_level: &mut VecDeque<Order>,
            incoming_order_qty: &mut u64,
            order_loc: &mut HashMap<u64, (Side, usize)>,
        ) -> u64 {
            let mut done_qty = 0;
            for o in price_level.iter_mut() {
                if o.qty <= *incoming_order_qty {
                    *incoming_order_qty -= o.qty;
                    done_qty += o.qty;
                    o.qty = 0;
                    order_loc.remove(&o.order_id);
                } else {
                    o.qty -= *incoming_order_qty;
                    done_qty += *incoming_order_qty;
                    *incoming_order_qty = 0;
                }
            }
            price_level.retain(|x| x.qty != 0);
            done_qty
        }

        let mut remaining_order_qty = order_qty;
        dbgp!(
            "Got order with qty {}, at price {}",
            remaining_order_qty,
            price
        );
        let mut fill_result = FillResult::new();
        match s {
            Side::Bid => {
                let askbook = &mut self.ask_book;
                let price_map = &mut askbook.price_map;
                let price_levels = &mut askbook.price_levels;
                let mut price_map_iter = price_map.iter();

                if let Some((mut x, _)) = price_map_iter.next() {
                    while price >= *x {
                        let curr_level = price_map[x];
                        let matched_qty = match_at_price_level(
                            &mut price_levels[curr_level],
                            &mut remaining_order_qty,
                            &mut self.order_loc,
                        );
                        if matched_qty != 0 {
                            dbgp!("Matched {} qty at level {}", matched_qty, x);
                            fill_result.filled_orders.push((matched_qty, *x));
                        }
                        if let Some((a, _)) = price_map_iter.next() {
                            x = a;
                        } else {
                            break;
                        }
                    }
                }
            }
            Side::Ask => {
                let bidbook = &mut self.bid_book;
                let price_map = &mut bidbook.price_map;
                let price_levels = &mut bidbook.price_levels;
                let mut price_map_iter = price_map.iter();

                if let Some((mut x, _)) = price_map_iter.next_back() {
                    while price <= *x {
                        let curr_level = price_map[x];
                        let matched_qty = match_at_price_level(
                            &mut price_levels[curr_level],
                            &mut remaining_order_qty,
                            &mut self.order_loc,
                        );
                        if matched_qty != 0 {
                            dbgp!("Matched {} qty at level {}", matched_qty, x);
                            fill_result.filled_orders.push((matched_qty, *x));
                        }
                        if let Some((a, _)) = price_map_iter.next_back() {
                            x = a;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        fill_result.remaining_qty = remaining_order_qty;
        if remaining_order_qty != 0 {
            dbgp!(
                "Still remaining qty {} at price level {}",
                remaining_order_qty,
                price
            );
            if remaining_order_qty == order_qty {
                fill_result.status = OrderStatus::Created;
            } else {
                fill_result.status = OrderStatus::PartiallyFilled;
            }
            self.create_new_limit_order(s, price, remaining_order_qty);
        } else {
            fill_result.status = OrderStatus::Filled;
        }
        self.update_bbo();

        fill_result
    }

    pub fn get_bbo(&self) {
        let total_bid_qty = self.bid_book.get_total_qty(self.best_bid_price);
        let total_ask_qty = self.ask_book.get_total_qty(self.best_offer_price);

        println!("Best bid {}, qty {}", self.best_bid_price, total_bid_qty);
        println!("Best ask {}, qty {}", self.best_offer_price, total_ask_qty);
        println!(
            "Spread is {:.6},",
            ((self.best_offer_price - self.best_bid_price) as f64 / self.best_offer_price as f64)
                as f32
        );
    }
}
