extern crate actix;
extern crate petgraph;
extern crate spade;
extern crate tokio_core;
extern crate futures;
extern crate rand;
#[macro_use]
extern crate actix_derive;

use actix::{Actor, Address, Arbiter, System, SyncAddress};

mod node;
mod world;
mod connection;

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
