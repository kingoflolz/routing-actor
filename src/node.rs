use actix::{msgs, Actor, Address, Arbiter, Context, System, Handler, Response, ResponseType};

use connection::Connection;

pub struct Node {
    neighbours: Vec<NeighbourData>,
    id: Option<Address<Node>>,
}

impl Node {
    pub fn new() -> Node {
        Node {
            neighbours: Vec::new(),
            id: None
        }
    }
}

struct NeighbourData {
    connection: Connection,
    id: Address<Node>,
}

struct Quality {
    latency: f32,
    bandwidth: f32,
    packet_loss: f32,
}

impl Actor for Node {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("I am alive!");
        Arbiter::system().send(msgs::SystemExit(0));
    }
}

pub struct InitSelf {
    pub id: Address<Node>
}

impl ResponseType for InitSelf {
    type Item = ();
    type Error = ();
}

impl Handler<InitSelf> for Node {
    fn handle(&mut self, msg: InitSelf, _ctx: &mut Context<Self>) -> Response<Node, InitSelf> {
        self.id = Some(msg.id);
        Self::reply(())
    }
}