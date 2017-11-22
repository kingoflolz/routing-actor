use actix::*;
use petgraph::graph::NodeIndex;
use connection::Connection;

use rand::{thread_rng, Rng};

use world;
use nc;
use packet::*;

pub struct Node {
    id: u64,
    // going to be bigger in the future
    graph_index: NodeIndex,
    neighbours: Vec<NeighbourData>,
    nc: nc::NCNodeData,
}

impl Node {
    pub fn new(graph_index: NodeIndex) -> Node {
        Node {
            id: thread_rng().next_u64(),
            neighbours: Vec::new(),
            graph_index,
            nc: nc::NCNodeData::new(),
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

        world_addr.send(world::HelloWorld { addr: ctx.address(), graph_index: self.graph_index, id: self.id });

        // println!("Node started {:?}", self.graph_index);
        // Arbiter::system().send(msgs::SystemExit(0));
    }
}

// out of band messages

// sent by node to another node to notify its presence
#[derive(Message)]
pub struct HelloNode {
    pub addr: SyncAddress<Node>,
    pub connection: Connection,
    // if this is a reply
    pub reply: bool,
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

#[derive(Message)]
pub struct Tick;

impl Handler<Tick> for Node {
    fn handle(&mut self, _msg: Tick, _ctx: &mut Context<Self>) -> Response<Self, Tick> {
        Self::reply(())
    }
}

// in band messages
impl<T: PacketData> Handler<Packet<T>> for Node {
    fn handle(&mut self, msg: Packet<T>, ctx: &mut Context<Self>) -> Response<Self, Packet<T>> {
        T::process(&msg, self);
        Self::reply(())
    }
}

impl<T: PacketData> Handler<SearchPacket<T>> for Node {
    fn handle(&mut self, msg: SearchPacket<T>, ctx: &mut Context<Self>) -> Response<Self, SearchPacket<T>> {
        T::process_search(&msg, self);
        Self::reply(())
    }
}