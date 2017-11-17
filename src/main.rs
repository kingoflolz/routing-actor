extern crate actix;
extern crate petgraph;
extern crate spade;

use actix::{msgs, Actor, Address, Arbiter, Context, System, Handler, ResponseType};

mod node;
mod world;
mod connection;

use node::*;

fn main() {
    let system = System::new("test");

    let addr: Address<_> = Node::new().start();
    addr.send(InitSelf { id: addr.clone() });
    system.run();
}
