#[macro_use]
extern crate lazy_static;
extern crate rand;

use std::collections::HashMap;
use crossbeam::channel::{ unbounded, Sender, Receiver };
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use rand::Rng;
use rand::seq::{SliceRandom, IteratorRandom};

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

    static ref ITEMS: Vec<&'static str> = {
        let mut v: Vec<&'static str> = Vec::new();

        for drink in DRINKS {
            v.push(drink);
        }

        for food in FOODS {
            v.push(food);
        }

        v
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
    customer_id: usize
}


// Can complete make any item, thus can complete any order
// Gets instructions for what food to make
struct Worker {
    id: usize,
    order_inbox: Receiver<Item>,
    cafe_bar: Sender<Item>,
}

impl Worker {
    fn new(id: usize, order_inbox: Receiver<Item>, cafe_bar: Sender<Item>) -> Self {
        Worker { id, order_inbox, cafe_bar }
    }

    fn work(self) -> JoinHandle<()> {
        println!("Spawning worker {}", self.id);
        thread::spawn(move || {
            loop {
                match self.order_inbox.recv() {
                    Ok(item) => {
                        println!("Worker {} received item '{}' for customer {}", self.id, item.item_name, item.customer_id);
                        thread::sleep(Duration::from_millis(50));
                        self.cafe_bar.send(item);
                    },
                    Err(err) => {
                        println!("Worker {} err'd on receiving order. Shutting down...: {}", self.id, err);
                        break;
                    },
                }
            }
        })
    }
}

// Arbiter between Customers and the Workers. Takes Orders from Customers, collects payment,
// dispatches Items in Orders to Worker(s)
struct Cashier {
    id: usize,
    customer_orders: Receiver<Order>,
    order_outbox: Sender<Item>,
}

impl Cashier {
    fn new(id: usize, customer_orders: Receiver<Order>, order_outbox: Sender<Item>) -> Self {
        Cashier { id, customer_orders, order_outbox }
    }

    fn work(self) -> JoinHandle<()> {
        println!("Spawning cashier {}", self.id);
        thread::spawn( move || {
            loop {
                match self.customer_orders.recv() {
                    Ok(order) => {
                        println!("Cashier {} received order from customer {}", self.id, order.customer_id);
                        for item in order.items {
                            thread::sleep(Duration::from_millis(50));
                            self.order_outbox.send(item);
                        }
                    },
                    Err(err) => {
                        println!("Cashier {} err'd on receiving order. Shutting down...: {}", self.id, err);
                        break;
                    },
                }
            }
        })
    }
}

// Items are set here after being made.
// Customers will occasionally check the bar to see if items from their order have been placed there.
struct CafeBar {
    finished_items: Receiver<Item>,
    customer_pickup: Sender<Item>,
}

impl CafeBar {
    fn new(finished_items: Receiver<Item>, customer_pickup: Sender<Item>) -> Self {
        CafeBar { finished_items, customer_pickup }
    }

    fn be_a_bar(self) -> JoinHandle<()> {
        println!("Spawning CafeBar");
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
        })
    }
}

// A customer goes to the Cashier to make an Order, then occasionally checks the CafeBar to see
// if their items have been made. They leave the store once they get all of the items in their order.
struct Customer {
    id: usize,
    order: Order,
    cashier: Sender<Order>,
    cafe_bar: Receiver<Item> // NOTE: customer picks up items as they come? or will wait until all items have been placed on the bar?
}

impl Customer {
    fn new(id: usize, cashier: Sender<Order>, cafe_bar: Receiver<Item>) -> Self {
        let num_items = rand::thread_rng().gen_range(0, 10);
        let items = (0..num_items).map(|i| {
            Item {
                item_name: ITEMS.choose(&mut rand::thread_rng()).unwrap().to_string(),
                customer_id: id
            }
        }).collect::<Vec<Item>>();

        Customer {
            id,
            order: Order {
                items,
                customer_id: id
            },
            cashier,
            cafe_bar
        }
    }

    fn work(self) -> JoinHandle<()> {
        println!("Spawning customer {}", self.id);
        thread::spawn(move || {
            self.cashier.send(self.order);
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
    // Create dedicated channels
    let (bar_out, customer_in) = unbounded::<Item>();
    let (worker_out, bar_in) = unbounded::<Item>();
    let (customer_out, cashier_in) = unbounded::<Order>();
    let (cashier_out, worker_in) = unbounded::<Item>();

    // Spawn customers
    let customer_handles = (0..3).map(|id| {
        let c = Customer::new(id, customer_out.clone(), customer_in.clone());
        c.work()
    }).collect::<Vec<JoinHandle<()>>>();

    // One cashier in our cafe
    let cashier = Cashier::new(0, cashier_in, cashier_out);
    cashier.work();

    // Spawn workers
    let worker_handles = (0..5).map(|id| {
        let w = Worker::new(id, worker_in.clone(), worker_out.clone());
        w.work()
    }).collect::<Vec<JoinHandle<()>>>();

    // Set up bar
    let bar = CafeBar::new(bar_in, bar_out);
    bar.be_a_bar();


    // wait for all customers to finish getting their orders
    for handle in customer_handles {
        handle.join();
    }

    println!("All customers have gotten their orders");
}
