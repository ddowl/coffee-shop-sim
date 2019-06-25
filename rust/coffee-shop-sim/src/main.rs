#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use crossbeam::channel::{ unbounded, Sender, Receiver };

lazy_static! {
    static ref MENU_PRICES: HashMap<&'static str, f32> = {
        let mut m = HashMap::new();
        m.insert("Drip Coffee", 2.50);
        m.insert("Latte", 3.00);
        m.insert("Mocha", 4.00);
        m.insert("Muffin", 5.00);
        m.insert("Omelette", 5.00);
        m.insert("Toast", 2.00);
        m.insert("Egg Bacon English Muffin Sandwich", 15.00);
        m
    };
}

static DRINKS: &'static [&'static str] = &["Drip Coffee", "Latte", "Mocha"];
static FOODS: &'static [&'static str] = &["Muffin", "Omelette", "Toast", "Egg Bacon English Muffin Sandwich"];

struct Order {
    items: Vec<Item>,
    customer_id: usize
}

struct Item {
    item_name: String,
    customer_id: Option<usize>
}


// Can complete make any item, thus can complete any order
// Gets instructions for what food to make
struct Worker {
    order_inbox: Receiver<Item>,
    cafe_bar: Sender<Item>,
}

// Arbiter between Customers and the Workers. Takes Orders from Customers, collects payment,
// dispatches Order to Worker(s)
struct Cashier {
    customer_orders: Receiver<Item>,
    order_outbox: Sender<Item>,
}

// Items are set here after being made.
// Customers will occasionally check the bar to see if items from their order have been placed there.
struct CafeBar {
    finished_items: Receiver<Item>,
    customer_pickup: Sender<Item>,
}

// A customer goes to the Cashier to make an Order, then occasionally checks the CafeBar to see
// if their items have been made. They leave the store once they get all of the items in their order.
struct Customer {
    order: Order
}

/*
Different actor systems in a coffee:
- one worker completes all items in order
- pool of workers completes all items in order
- Item dependencies: certain food is cooked by Chef workers, drinks made by Barista workers
- Kitchen resource dependencies: certain foods can only cooked with stoves/ovens/microwaves, etc, by Chefs
*/

fn main() {
// Create a channel of unbounded capacity.
    let (s, r) = unbounded();

// Send a message into the channel.
    s.send("Hello, world!").unwrap();

// Receive the message from the channel.
    assert_eq!(r.recv(), Ok("Hello, world!"));
}
