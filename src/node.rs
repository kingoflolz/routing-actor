use actix::*;
use petgraph::graph::NodeIndex;
use connection::Connection;

use rand::{thread_rng, Rng};

use std::collections::HashMap;
use std::marker::Send;

use futures::Future;

use world;
use nc;
use packet::*;

use packet::PacketData;
use std::clone::Clone;

use dht::service::*;

pub struct Node {
    // going to be bigger in the future
    pub world: SyncAddress<world::World>,
    pub id: u64,
    pub graph_index: NodeIndex,
    pub neighbours: Vec<NeighbourData>,
    pub neighbours_map: HashMap<u64, usize>,
    pub nc: nc::NCNodeData,
    pub dht: DHT,
}

impl Node {
    pub fn new(graph_index: NodeIndex) -> Node {
        let id = thread_rng().next_u64();
        Node {
            world: Arbiter::system_registry().get::<world::World>(),
            id,
            neighbours: Vec::new(),
            graph_index,
            neighbours_map: HashMap::new(),
            nc: nc::NCNodeData::new(),
            dht: DHT::new(id)
        }
    }

    pub fn fwd<T: PacketData + Clone + Send + ResponseType + 'static>(&self, msg: Packet<T>) -> Response<Self, Packet<T>>
        where T::Item: Send, T::Error: Send {
        let mut msg = msg.clone();
        let next = msg.route.pop().unwrap_or(msg.des);
        let index = self.neighbours_map[&next];
        let f = self.neighbours[index].address.call(self, msg);
        Node::async_reply(ActorFuture::then(f, |item, actor, ctx| {
            match item.unwrap() {
                Ok(s) => fut::ok::<T::Item, T::Error, Node>(s),
                Err(e) => fut::err::<T::Item, T::Error, Node>(e)
            }
        }))
    }

    pub fn send_packet<T: PacketData + Clone + Send + ResponseType + 'static>(&self, msg: Packet<T>) -> Request<Node, Packet<T>>
        where T::Item: Send, T::Error: Send {
        let mut msg = msg.clone();
        let next = msg.route.pop().unwrap_or(msg.des);
        let index = self.neighbours_map[&next];
        self.neighbours[index].address.call(self, msg)
    }
}


#[derive(Clone)]
pub struct NeighbourData {
    pub id: u64,
    pub connection: Connection,
    pub address: SyncAddress<Node>,
}

struct Quality {
    latency: f32,
    bandwidth: f32,
    packet_loss: f32,
}

impl Actor for Node {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.world.send(world::HelloWorld { addr: ctx.address(), graph_index: self.graph_index, id: self.id });

        // println!("Node started {:?}", self.graph_index);
        // Arbiter::system().send(msgs::SystemExit(0));
    }
}

// out of band messages

// sent by node to another node to notify its presence
#[derive(Message)]
pub struct HelloNode {
    pub id: u64,
    pub pipe: SyncAddress<Node>,
    pub connection: Connection,
    // if this is a reply
    pub reply: bool,
}

impl Handler<HelloNode> for Node {
    fn handle(&mut self, msg: HelloNode, ctx: &mut Context<Self>) -> Response<Self, HelloNode> {
        if !self.neighbours_map.contains_key(&msg.id) {
            self.neighbours_map.insert(msg.id, self.neighbours.len());
            self.neighbours.push(NeighbourData { address: msg.pipe.clone(), connection: msg.connection.clone(), id: msg.id });
            // only send back message if message it did not originate to prevent loops
            if !msg.reply {
                msg.pipe.send(HelloNode { pipe: ctx.address(), reply: true, id: self.id, ..msg })
            }
        }
        Self::reply(())
    }
}

#[derive(Message)]
pub struct Tick;

impl Handler<Tick> for Node {
    fn handle(&mut self, _msg: Tick, ctx: &mut Context<Self>) -> Response<Self, Tick> {
        self.dht_tick(ctx);

        Self::reply(())
    }
}

// in band messages
impl<T: PacketData + Clone + Send + ResponseType + 'static> Handler<Packet<T>> for Node where <T as ResponseType>::Item: Send, <T as ResponseType>::Error: Send {
    fn handle(&mut self, msg: Packet<T>, ctx: &mut Context<Self>) -> Response<Self, Packet<T>> {
        if thread_rng().next_f32() < 0.01 {
            self.world.send(world::Sent);
        }
        if msg.des == self.id {
            assert_eq!(msg.route.len(), 0);
            let r = T::process(&msg, self);
            r
        } else {
            assert!(msg.route.len() > 0);
            self.fwd(msg)
        }
    }
}

impl<T: SearchPacketData + Clone + Send + ResponseType> Handler<SearchPacket<T>> for Node {
    fn handle(&mut self, msg: SearchPacket<T>, ctx: &mut Context<Self>) -> Response<Self, SearchPacket<T>> {
        T::process(&msg, self)
    }
}