use std::collections::{BTreeMap, VecDeque};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub enum Side {
    Bid,  // Buy order
    Ask,  // Sell order
}

#[derive(Debug, Clone)]
pub struct Order {
    id: String,
    price: Decimal,
    quantity: Decimal,
    side: Side,
    timestamp: u64,
}

#[derive(Debug)]
pub struct Trade {
    maker_order_id: String,
    taker_order_id: String,
    price: Decimal,
    quantity: Decimal,
}

pub struct OrderBook {
    asks: BTreeMap<Decimal, VecDeque<Order>>,  // Sell orders sorted by price ascending
    bids: BTreeMap<Decimal, VecDeque<Order>>,  // Buy orders sorted by price descending
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            asks: BTreeMap::new(),
            bids: BTreeMap::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) -> Vec<Trade> {
        match order.side {
            Side::Bid => self.match_bid_order(order),
            Side::Ask => self.match_ask_order(order),
        }
    }

    fn match_bid_order(&mut self, mut bid: Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        
        while bid.quantity > Decimal::ZERO {
            let should_remove_price = {
                if let Some(mut entry) = self.asks.first_entry() {
                    let ask_price = *entry.key();
                    if ask_price > bid.price {
                        break;
                    }

                    let ask_orders = entry.get_mut();
                    if let Some(ask) = ask_orders.front_mut() {
                        let trade_quantity = bid.quantity.min(ask.quantity);
                        
                        trades.push(Trade {
                            maker_order_id: ask.id.clone(),
                            taker_order_id: bid.id.clone(),
                            price: ask_price,
                            quantity: trade_quantity,
                        });

                        bid.quantity -= trade_quantity;
                        ask.quantity -= trade_quantity;

                        // Remove filled ask order
                        if ask.quantity == Decimal::ZERO {
                            ask_orders.pop_front();
                        }
                    }

                    ask_orders.is_empty()
                } else {
                    break;
                }
            };

            if should_remove_price {
                self.asks.remove(&bid.price);
            }
        }

        // If there's remaining quantity, add to order book
        if bid.quantity > Decimal::ZERO {
            self.bids.entry(bid.price)
                .or_insert_with(VecDeque::new)
                .push_back(bid);
        }

        trades
    }

    fn match_ask_order(&mut self, mut ask: Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        
        while ask.quantity > Decimal::ZERO {
            if let Some(mut entry) = self.bids.last_entry() {
                let bid_price = *entry.key();
                if bid_price < ask.price {
                    break;
                }

                let bid_orders = entry.get_mut();
                if let Some(bid) = bid_orders.front_mut() {
                    let trade_quantity = ask.quantity.min(bid.quantity);
                    
                    trades.push(Trade {
                        maker_order_id: bid.id.clone(),
                        taker_order_id: ask.id.clone(),
                        price: bid_price,
                        quantity: trade_quantity,
                    });

                    ask.quantity -= trade_quantity;
                    bid.quantity -= trade_quantity;

                    // Remove filled bid order
                    if bid.quantity == Decimal::ZERO {
                        bid_orders.pop_front();
                    }
                }

                if bid_orders.is_empty() {
                    self.bids.remove(&ask.price);
                }
            } else {
                break;
            }
        }

        // If there's remaining quantity, add to order book
        if ask.quantity > Decimal::ZERO {
            self.asks.entry(ask.price)
                .or_insert_with(VecDeque::new)
                .push_back(ask);
        }

        trades
    }
} 
