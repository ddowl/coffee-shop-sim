#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use crossbeam::channel::{ unbounded, Sender, Receiver };
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

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

impl Worker {
    fn new(order_inbox: Receiver<Item>, cafe_bar: Sender<Item>) -> Self {
        Worker { order_inbox, cafe_bar }
    }

    fn work(&self) -> JoinHandle<()> {
        thread::spawn(|| {
            thread::sleep(Duration::from_millis(1000));
        })
    }
}

// Arbiter between Customers and the Workers. Takes Orders from Customers, collects payment,
// dispatches Items in Orders to Worker(s)
struct Cashier {
    customer_orders: Receiver<Order>,
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
    order: Order,
    cashier: Sender<Order>,
    cafe_bar: Receiver<Item> // NOTE: customer picks up items as they come? or will wait until all items have been placed on the bar?
}

impl Customer {
    fn new(id: usize, cashier: Sender<Order>, cafe_bar: Receiver<Item>) -> Self {
        // TODO: generate random items for each order
        Customer {
            order: Order {
                items: vec![],
                customer_id: id
            },
            cashier,
            cafe_bar
        }
    }

    fn work(&self) -> JoinHandle<()> {
        println!("Spawning customer {}", self.order.customer_id);
        thread::spawn(|| {
            thread::sleep(Duration::from_millis(1000));
        })
    }
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

//    let handles = (1..10).map(|i| {
//        Worker::new(None, None).work()
//    }).collect();

    // Create dedicated channels
    let (bar_out, customer_in) = unbounded::<Item>();
    let (bar_in, worker_out) = unbounded::<Item>();
    let (customer_out, cashier_in) = unbounded::<Order>();
    let (cashier_out, worker_in) = unbounded::<Item>();

    let customer_handles = (0..3).map(|id| {
        let c = Customer::new(id, customer_out.clone(), customer_in.clone());
        c.work()
    }).collect::<Vec<JoinHandle<()>>>();

    // wait for all customers to finish getting their orders
    for handle in customer_handles {
        handle.join();
    }

    println!("All customers have gotten their orders");
}
