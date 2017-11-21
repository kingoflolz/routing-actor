extern crate actix;
extern crate petgraph;
extern crate spade;
extern crate tokio_core;
extern crate futures;
extern crate rand;
#[macro_use]
extern crate actix_derive;

use actix::*;

mod node;
mod world;
mod connection;

fn main() {
    let system = System::new("test");

    let addr = Arbiter::system_registry().get::<world::World>();

    addr.send(world::Wake);

    system.run();
}
