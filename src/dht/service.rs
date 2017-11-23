// Copyright 2016 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

//! Protocol-agnostic service implementation

use std::marker;
use std::collections::HashMap;

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
    pub fn dht_tick(&mut self) {
        for n in &self.neighbours {
            self.fwd(Packet {
                des: n.id,
                route: Vec::new(),
                data: Ping { from: self.id },
            });
        }
    }
}

#[derive(Clone, Debug)]
pub struct Ping {
    pub from: u64
}

impl PacketData for Ping {
    fn process(packet: &Packet<Self>, node: &mut Node) {
        node.dht.on_ping(&DHTNode { id: packet.data.from, route: packet.route.clone() });
    }
}