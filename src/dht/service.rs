// Copyright 2016 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

//! Protocol-agnostic service implementation

use actix::*;

use std::marker;
use std::collections::HashMap;
use rand::{thread_rng, Rng};

use super::{GenericId, GenericNodeTable, DHTNode};
use dht::knodetable::KNodeTable;

use node::Node;

use packet::*;

static MAX_NODE_COUNT: usize = 16;

pub type DHT = GenDHT<u64, Vec<u64>, KNodeTable<u64, Vec<u64>>, ()>;

/// Result of the find operations - either data or nodes closest to it.
#[derive(Debug)]
pub enum FindResult<TId, TAddr, TData> {
    Value(TData),
    ClosestNodes(Vec<DHTNode<TId, TAddr>>),
    Nothing
}

#[derive(Debug)]
pub struct GenDHT<TId, TAddr, TNodeTable, TData>
    where TId: GenericId,
          TNodeTable: GenericNodeTable<TId, TAddr>,
          TData: Send + Sync + Clone {
    _phantom: marker::PhantomData<TAddr>,
    node_id: TId,
    table: TNodeTable,
    data: HashMap<TId, TData>,
}

impl<TId, TAddr, TNodeTable, TData> GenDHT<TId, TAddr, TNodeTable, TData>
    where TId: GenericId,
          TNodeTable: GenericNodeTable<TId, TAddr>,
          TData: Send + Sync + Clone {
    pub fn new(node_id: TId) -> Self {
        GenDHT {
            node_id: node_id.clone(),
            table: TNodeTable::new(node_id),
            data: HashMap::new(),
            _phantom: marker::PhantomData,
        }
    }

    /// Process the ping request.
    ///
    /// Essentially remembers the incoming node and returns true.
    pub fn on_ping(&mut self, sender: &DHTNode<TId, TAddr>) -> bool {
        self.update(sender);
        true
    }
    /// Process the find request.
    pub fn on_find_node(&mut self, sender: &DHTNode<TId, TAddr>, id: &TId) -> Vec<DHTNode<TId, TAddr>> {
        let res = self.table.find(&id, MAX_NODE_COUNT);
        self.update(sender);
        res
    }
    /// Find a value or the closes nodes.
    pub fn on_find_value(&mut self, sender: &DHTNode<TId, TAddr>, id: &TId)
                         -> FindResult<TId, TAddr, TData> {
        self.update(sender);
        let data = &self.data;
        let table = &self.table;
        let res = match data.get(&id) {
            Some(value) => FindResult::Value(value.clone()),
            None => FindResult::ClosestNodes(table.find(&id, MAX_NODE_COUNT))
        };
        res
    }

    fn update(&mut self, node: &DHTNode<TId, TAddr>) {
        if node.id == self.node_id {
            return;
        }

        if !self.table.update(&node) {
        }
    }
}

impl Node {
    pub fn dht_tick(&mut self, ctx: &mut Context<Self>) {
        for n in &self.neighbours.clone() {
            if thread_rng().next_f32() < 0.1 {
                ctx.spawn(self.send_packet(Packet {
                    from: self.id,
                    des: n.id,
                    route: Vec::new(),
                    data: DHTLookup { goal: self.id },
                }).then(|item, actor, ctx| {
                    // Arbiter::system().send(msgs::SystemExit(0));
                    // println!("dht rec {:?}", item.clone().unwrap().unwrap());
                    for mut i in item.clone().unwrap().unwrap().data.reply {
                        i.route.append(&mut item.clone().unwrap().unwrap().get_full_route());
                        i.route = simplify_route(i.route);
                        actor.dht.update(&i);
                        // if thread_rng().next_f32() < 0.001 {
                        //     println!("{:?}", actor.dht)
                        // }
                    }
                    fut::ok::<(), (), Node>(())
                }));

                ctx.spawn(self.send_packet(Packet {
                    from: self.id,
                    des: n.id,
                    route: Vec::new(),
                    data: DHTLookup { goal: thread_rng().next_u64() },
                }).then(|item, actor, ctx| {
                    // println!("dht rec {:?}", item.clone().unwrap().unwrap());
                    for mut i in item.clone().unwrap().unwrap().data.reply {
                        i.route.append(&mut item.clone().unwrap().unwrap().get_full_route());
                        i.route = simplify_route(i.route);
                        actor.dht.update(&i)
                    }
                    // Arbiter::system().send(msgs::SystemExit(0));
                    fut::ok::<(), (), Node>(())
                }));

                // let mut r = self.send_packet(Packet {
                //     from: self.id,
                //     des: n.id,
                //     route: Vec::new(),
                //     data: Ping,
                // });
                // let mut f = ActorFuture::then(r, |item, actor, ctx| {
                //     println!("ping rec");
                //     fut::ok::<(), (), Node>(())
                // });
                // Arbiter::
            }
        }
    }
}

#[derive(Clone, Debug, Message)]
pub struct Ping;

impl PacketData for Ping {
    fn process(packet: &Packet<Self>, node: &mut Node) -> Response<Node, Packet<Ping>> {
        node.dht.on_ping(&DHTNode { id: packet.from, route: packet.route.clone() });
        Node::reply(())
    }
}

type DHTLookupReplyPacket = Packet<DHTLookupReply>;

#[derive(Clone, Debug, Message)]
#[Message(DHTLookupReplyPacket)]
pub struct DHTLookup {
    pub goal: u64,
}

#[derive(Clone, Debug, Message)]
pub struct DHTLookupReply {
    pub goal: u64,
    pub reply: Vec<DHTNode<u64, Vec<u64>>>,
}


impl PacketData for DHTLookup {
    fn process(packet: &Packet<Self>, node: &mut Node) -> Response<Node, Packet<Self>> {
        let r = node.dht.on_find_node(&DHTNode { id: packet.from, route: packet.get_full_route() }, &packet.data.goal);
        let p = packet.reverse();
        Node::reply(Packet::new(p, DHTLookupReply { reply: r, goal: packet.data.goal }))
    }
}

impl PacketData for DHTLookupReply {
    fn process(packet: &Packet<Self>, node: &mut Node) -> Response<Node, Packet<Self>> {
        Node::reply(())
    }
}
