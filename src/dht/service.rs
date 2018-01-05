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
use actix::fut::result;
use actix::fut::FutureResult;
use std::any::Any;

use std::marker;
use std::collections::HashMap;
use rand::{thread_rng, Rng};

use super::{GenericId, GenericNodeTable, DHTNode};
use dht::knodetable::KNodeTable;

use node::Node;

use packet::*;

static MAX_NODE_COUNT: usize = 16;

pub type DHT = GenDHT<KNodeTable, ()>;

/// Result of the find operations - either data or nodes closest to it.
#[derive(Debug)]
pub enum FindResult<TData> {
    Value(TData),
    ClosestNodes(Vec<DHTNode>),
    Nothing
}

#[derive(Debug)]
pub struct GenDHT<TNodeTable, TData>
    where TNodeTable: GenericNodeTable,
          TData: Send + Sync + Clone {
    node_id: u64,
    table: TNodeTable,
    data: HashMap<u64, TData>,
}

impl<TNodeTable, TData> GenDHT<TNodeTable, TData>
    where TNodeTable: GenericNodeTable,
          TData: Send + Sync + Clone {
    pub fn new(node_id: u64) -> Self {
        GenDHT {
            node_id: node_id.clone(),
            table: TNodeTable::new(node_id),
            data: HashMap::new(),
        }
    }

    /// Process the ping request.
    ///
    /// Essentially remembers the incoming node and returns true.
    pub fn on_ping(&mut self, sender: &DHTNode) -> bool {
        self.update(sender);
        true
    }
    /// Process the find request.
    pub fn on_find_node(&mut self, sender: &DHTNode, id: &u64) -> Vec<DHTNode> {
        let res = self.table.find(&id, MAX_NODE_COUNT);
        self.update(sender);
        res
    }
    /// Find a value or the closes nodes.
    pub fn on_find_value(&mut self, sender: &DHTNode, id: &u64)
                         -> FindResult<TData> {
        self.update(sender);
        let data = &self.data;
        let table = &self.table;
        let res = match data.get(&id) {
            Some(value) => FindResult::Value(value.clone()),
            None => FindResult::ClosestNodes(table.find(&id, MAX_NODE_COUNT))
        };
        res
    }

    fn update(&mut self, node: &DHTNode) {
        if node.id == self.node_id {
            return;
        }

        if !self.table.update(&node) {}
    }
}

impl Node {
    fn dht_lookup(&self, goal: u64, current_nodes: Option<Vec<DHTNode>>, init: bool)
                  -> Box<ActorFuture<Item=DHTNode, Error=(), Actor=Node>> {
        println!("ID: {}, looking for goal {}", self.id, goal);
        let mut closest = current_nodes.unwrap_or(self.dht.table.find(&(goal), 16));

        // println!("c {:?} {}", closest, self.id);

        if init && closest.len() > 0 {
            closest.remove(0);
        }

        if closest.len() < 1 || closest[0].id == self.id {
            println!("ID: {}, not found {}, returning", self.id, goal);
            return Box::new(fut::err(()));
        }

        if closest[0].id == goal {
            let mut r = closest[0].clone();
            r.route.push(self.id);
            println!("ID: {}, found {}!", self.id, goal);
            return Box::new(fut::ok(r));
        }

        let mut r = closest[0].route.clone();

        println!("ID: {}, sending packet to {}, looking for goal {}", self.id, closest[0].id, goal);
        Box::new(self.send_packet(Packet {
            from: self.id,
            des: closest[0].id,
            route: r.clone(),
            data: DHTLookup { goal, path_to: r },
        }).then(move |item, actor, ctx| {
            let mut c = closest;
            match item.clone().unwrap() {
                Ok(response) => {
                    for i in response.data.reply.clone().iter_mut() {
                        i.route.append(&mut item.clone().unwrap().unwrap().get_full_route());
                        // i.route = simplify_route(i.route.clone());
                        actor.dht.update(&i)
                    }

                    let r = &response.data.reply[0];
                    if r.id == goal {
                        let mut r = r.clone();
                        let mut hop = response.clone().get_full_route();
                        hop.reverse();
                        r.prepend(&mut hop);
                        println!("ID: {}, found {}, replying", actor.id, goal);
                        return Box::new(fut::ok(DHTNode { id: 0u64, route: r.route })) as Box<ActorFuture<Item=DHTNode, Error=(), Actor=Node>>
                    } else {
                        println!("ID: {}, not found {}, recursing", actor.id, goal);
                        return actor.dht_lookup(goal, Some(response.data.reply.clone()), init)
                    }
                }
                Err(error) => {
                    println!("ID: {}, err", actor.id);
                    if c.len() > 0 {
                        return actor.dht_lookup(goal, Some(c), init)
                    } else {
                        println!("ID: {}, not found {}, replying", actor.id, goal);
                        return Box::new(fut::err(())) as Box<ActorFuture<Item=DHTNode, Error=(), Actor=Node>>
                    }
                }
            }
        }))

        // self.send_packet(Packet {
        //     from: self.id,
        //     des: n.id,
        //     route: Vec::new(),
        //     data: DHTLookup { goal: self.id },
        // }).then(|item, actor, ctx| {
        //     // Arbiter::system().send(msgs::SystemExit(0));
        //     // println!("dht rec {:?}", item.clone().unwrap().unwrap());
        //     for mut i in item.clone().unwrap().unwrap().data.reply {
        //         i.route.append(&mut item.clone().unwrap().unwrap().get_full_route());
        //         i.route = simplify_route(i.route);
        //         actor.dht.update(&i);
        //         if thread_rng().next_f32() < 0.001 {
        //             println!("{:?}", actor.dht)
        //         }
        //     }
        //     fut::ok::<(), (), Node>(())
        // });
    }

    pub fn dht_tick(&mut self, ctx: &mut Context<Self>) {
        println!("ID: {}, Table {:?}", self.id, self.dht);
        for n in &self.neighbours.clone() {
            let mut r = self.send_packet(Packet {
                from: self.id,
                des: n.id,
                route: vec![n.id],
                data: Ping,
            });

            // ctx.spawn(self.dht_lookup(self.id, None, true).then(|item, ctx, context| {
            //     fut::ok::<(), (), Node>(())
            // }));
            ctx.spawn(self.dht_lookup(thread_rng().next_u64(), None, false).then(|item, actor, context| {
                fut::ok::<(), (), Node>(())
            }));

                // ctx.spawn(self.send_packet(Packet {
                //     from: self.id,
                //     des: n.id,
                //     route: Vec::new(),
                //     data: DHTLookup { goal: thread_rng().next_u64() },
                // }).then(|item, actor, ctx| {
                //     // println!("dht rec {:?}", item.clone().unwrap().unwrap());
                //     for mut i in item.clone().unwrap().unwrap().data.reply {
                //         i.route.append(&mut item.clone().unwrap().unwrap().get_full_route());
                //         i.route = simplify_route(i.route);
                //         actor.dht.update(&i)
                //     }
                //     // Arbiter::system().send(msgs::SystemExit(0));
                //     fut::ok::<(), (), Node>(())
                // }));

                // let mut f = ActorFuture::then(r, |item, actor, ctx| {
                //     println!("ping rec");
                //     fut::ok::<(), (), Node>(())
                // });
                // Arbiter::
            }
        self.dht_init = true;
    }
}

#[derive(Clone, Debug, Message)]
pub struct Ping;

impl PacketData for Ping {
    fn process(packet: &Packet<Self>, node: &mut Node) -> Response<Node, Packet<Ping>> {
        let mut r = packet.get_full_route().clone();
        r.push(packet.from);
        r.reverse();
        node.dht.on_ping(&DHTNode { id: packet.from, route: r });
        Node::reply(())
    }
}

type DHTLookupReplyPacket = Packet<DHTLookupReply>;

#[derive(Clone, Debug, Message)]
#[Message(DHTLookupReplyPacket)]
pub struct DHTLookup {
    pub goal: u64,
    pub path_to: Vec<u64>,
}

#[derive(Clone, Debug, Message)]
pub struct DHTLookupReply {
    pub goal: u64,
    pub reply: Vec<DHTNode>,
}


impl PacketData for DHTLookup {
    fn process(packet: &Packet<Self>, node: &mut Node) -> Response<Node, Packet<Self>> {
        println!("ID: {}, processed DHT lookup", node.id);
        let r = node.dht.on_find_node(&DHTNode { id: packet.from, route: packet.route.reverse() }, &packet.data.goal);
        let p = packet.reverse();
        Node::reply(Packet::new(Packet { from: Node.id, des: packet.from }, DHTLookupReply { reply: r, goal: packet.data.goal }))
    }
}

impl PacketData for DHTLookupReply {
    fn process(packet: &Packet<Self>, node: &mut Node) -> Response<Node, Packet<Self>> {
        Node::reply(())
    }
}
