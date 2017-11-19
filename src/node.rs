use actix::{Actor, Address, Context};
use petgraph::graph::NodeIndex;
use connection::Connection;

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
        // println!("Node started {:?}", self.graph_index);
        // Arbiter::system().send(msgs::SystemExit(0));
    }
}