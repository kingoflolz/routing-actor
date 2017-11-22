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
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::{GenericId, GenericNodeTable, Node};


static MAX_NODE_COUNT: usize = 16;


/// Result of the find operations - either data or nodes closest to it.
#[derive(Debug)]
pub enum FindResult<TId, TAddr, TData> {
    Value(TData),
    ClosestNodes(Vec<Node<TId, TAddr>>),
    Nothing
}

/// Handler - implementation of DHT requests.
pub struct Handler<TId, TAddr, TNodeTable, TData>
    where TId: GenericId,
          TNodeTable: GenericNodeTable<TId, TAddr>,
          TData: Send + Sync + Clone {
    _phantom: marker::PhantomData<TAddr>,
    node_id: TId,
    table: Arc<RwLock<TNodeTable>>,
    data: Arc<RwLock<HashMap<TId, TData>>>,
    clean_needed: bool,
}

/// Protocol agnostic DHT service.
///
/// Its type parameters are `TNodeTable` - the node table implementation
/// (see e.g. `KNodeTable`) and `TData` - stored data type.
///
/// The service starts a network listening loop in a separate thread.
pub struct Service<TId, TAddr, TNodeTable, TData>
    where TId: GenericId,
          TNodeTable: GenericNodeTable<TId, TAddr>,
          TData: Send + Sync + Clone {
    handler: Handler<TId, TAddr, TNodeTable, TData>,
    node_id: TId,
    table: Arc<RwLock<TNodeTable>>,
    data: Arc<RwLock<HashMap<TId, TData>>>
}


impl<TId, TAddr, TNodeTable, TData> Service<TId, TAddr, TNodeTable, TData>
    where TId: GenericId,
          TAddr: Send + Sync,
          TNodeTable: GenericNodeTable<TId, TAddr>,
          TData: Send + Sync + Clone {
    /// Create a service with a random ID.
    pub fn new(node_table: TNodeTable) -> Service<TId, TAddr, TNodeTable, TData> {
        let node_id = node_table.random_id();
        Service::new_with_id(node_table, node_id)
    }
    /// Create a service with a given ID.
    pub fn new_with_id(node_table: TNodeTable, node_id: TId)
                       -> Service<TId, TAddr, TNodeTable, TData> {
        let table = Arc::new(RwLock::new(node_table));
        let data = Arc::new(RwLock::new(HashMap::new()));
        let handler = Handler {
            _phantom: marker::PhantomData,
            node_id: node_id.clone(),
            table: table.clone(),
            data: data.clone(),
            clean_needed: false
        };
        Service {
            handler: handler,
            node_id: node_id,
            table: table,
            data: data
        }
    }

    /// Get an immutable reference to the node table.
    pub fn node_table(&self) -> RwLockReadGuard<TNodeTable> {
        self.table.read().unwrap()
    }
    /// Get a mutable reference to the node table.
    pub fn node_table_mut(&mut self) -> RwLockWriteGuard<TNodeTable> {
        self.table.write().unwrap()
    }
    /// Get the current node ID.
    pub fn node_id(&self) -> &TId {
        &self.node_id
    }
    /// Get an immutable reference to the data.
    pub fn stored_data(&self)
                       -> RwLockReadGuard<HashMap<TId, TData>> {
        self.data.read().unwrap()
    }
    /// Get an immutable reference to the data.
    pub fn stored_data_mut(&mut self)
                           -> RwLockWriteGuard<HashMap<TId, TData>> {
        self.data.write().unwrap()
    }
    /// Check if some buckets are full already.
    pub fn clean_needed(&self) -> bool {
        self.handler.clean_needed
    }

    /// Try to clean up the table by checking the oldest records.
    ///
    /// Should be called periodically, especially when clean_needed is true.
    pub fn clean_up<TCheck>(&mut self, mut check: TCheck)
        where TCheck: FnMut(&Node<TId, TAddr>) -> bool {
        {
            let mut node_table = self.node_table_mut();
            let oldest = node_table.pop_oldest();
            for node in oldest {
                if check(&node) {
                    node_table.update(&node);
                }
            }
        }
        self.handler.clean_needed = false;
    }
}

impl<TId, TAddr, TNodeTable, TData> Handler<TId, TAddr, TNodeTable, TData>
    where TId: GenericId,
          TNodeTable: GenericNodeTable<TId, TAddr>,
          TData: Send + Sync + Clone {
    /// Process the ping request.
    ///
    /// Essentially remembers the incoming node and returns true.
    pub fn on_ping(&mut self, sender: &Node<TId, TAddr>) -> bool {
        self.update(sender);
        true
    }
    /// Process the find request.
    pub fn on_find_node(&mut self, sender: &Node<TId, TAddr>, id: &TId) -> Vec<Node<TId, TAddr>> {
        let res = self.table.read().unwrap().find(&id, MAX_NODE_COUNT);
        self.update(sender);
        res
    }
    /// Find a value or the closes nodes.
    pub fn on_find_value(&mut self, sender: &Node<TId, TAddr>, id: &TId)
                         -> FindResult<TId, TAddr, TData> {
        self.update(sender);
        let data = self.data.read().unwrap();
        let table = self.table.read().unwrap();
        let res = match data.get(&id) {
            Some(value) => FindResult::Value(value.clone()),
            None => FindResult::ClosestNodes(table.find(&id, MAX_NODE_COUNT))
        };
        res
    }

    fn update(&mut self, node: &Node<TId, TAddr>) {
        if node.id == self.node_id {
            return;
        }

        if !self.table.write().unwrap().update(&node) {
            self.clean_needed = true;
        }
    }
}