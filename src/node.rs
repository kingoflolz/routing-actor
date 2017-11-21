use actix::*;
use petgraph::graph::NodeIndex;
use connection::Connection;

use world;

pub struct Node {
    graph_index: NodeIndex,
    neighbours: Vec<NeighbourData>,
}

impl Node {
    pub fn new(graph_index: NodeIndex) -> Node {
        Node {
            neighbours: Vec::new(),
            graph_index,
        }
    }
}

struct NeighbourData {
    connection: Connection,
    address: SyncAddress<Node>,
}

struct Quality {
    latency: f32,
    bandwidth: f32,
    packet_loss: f32,
}

impl Actor for Node {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let world_addr = Arbiter::system_registry().get::<world::World>();

        world_addr.send(world::HelloWorld { addr: ctx.address(), graph_index: self.graph_index });

        // println!("Node started {:?}", self.graph_index);
        // Arbiter::system().send(msgs::SystemExit(0));
    }
}

// sent by node to another node to notify its presence
pub struct HelloNode {
    pub addr: SyncAddress<Node>,
    pub connection: Connection,
    // if this is a reply
    pub reply: bool,
}

impl ResponseType for HelloNode {
    type Item = ();
    type Error = ();
}

impl Handler<HelloNode> for Node {
    fn handle(&mut self, msg: HelloNode, ctx: &mut Context<Self>) -> Response<Self, HelloNode> {
        self.neighbours.push(NeighbourData { address: msg.addr.clone(), connection: msg.connection.clone() });

        // only send back message if message it did not originate to prevent loops
        if !msg.reply {
            msg.addr.send(HelloNode { addr: ctx.address(), reply: true, ..msg })
        }
        Self::reply(())
    }
}