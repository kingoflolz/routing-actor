extern crate actix;
extern crate petgraph;
extern crate spade;

use actix::{msgs, Actor, Address, Arbiter, Context, System, Handler, ResponseType, SyncAddress};

mod node;
mod world;
mod connection;

use node::*;

use std::thread;
use world::World;

fn main() {
    let system = System::new("test");

    let mut threads: Vec<SyncAddress<Arbiter>> = Vec::new();

    for i in 0..4 {
        threads.push(Arbiter::new(format!("Core {}", i)))
    }

    World::new(&threads).start::<Address<World>>();

    system.run();
}
