// Copyright 2014 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

//! DHT node table implementation based on Kademlia.
//!
//! See [original paper](http://pdos.csail.mit.edu/%7Epetar/papers/maymounkov-kademlia-lncs.pdf)
//! for details. The most essential difference is that when k-bucket is full,
//! no RPC call is done. It is up to upper-level code to ensure proper clean up
//! using `pop_oldest` call.

use std::cmp;
use std::fmt::Debug;
use std::collections::VecDeque;

use super::GenericId;
use super::GenericNodeTable;
use super::DHTNode;


// TODO(divius): make public?
static BUCKET_SIZE: usize = 32;
static DEFAULT_HASH_SIZE: usize = 64;


/// Kademlia node table.
///
/// Keeps nodes in a number of k-buckets (equal to bit size of ID in a system,
/// usually 160), where N-th k-bucket contains nodes with distance
/// from 2^N to 2^(N+1) from our node.
///
/// methods may panic if distance between two ids is greater than the
/// `hash_size`.
#[derive(Debug)]
pub struct KNodeTable<TId, TAddr> {
    this_id: TId,
    hash_size: usize,
    // TODO(divius): convert to more appropriate data structure
    buckets: Vec<KBucket<TId, TAddr>>,
}

/// K-bucket - structure for keeping last nodes in Kademlia.
#[derive(Debug)]
pub struct KBucket<TId, TAddr> {
    data: VecDeque<DHTNode<TId, TAddr>>,
    size: usize,
}


impl<TId, TAddr> KNodeTable<TId, TAddr>
    where TId: GenericId,
          TAddr: Clone + Debug {
    /// Create a new node table.
    ///
    /// `this_id` -- ID of the current node (used to calculate metrics).
    pub fn new(this_id: TId) -> KNodeTable<TId, TAddr> {
        KNodeTable::new_with_details(this_id, BUCKET_SIZE, DEFAULT_HASH_SIZE)
    }

    pub fn new_with_details(this_id: TId, bucket_size: usize,
                            hash_size: usize) -> KNodeTable<TId, TAddr> {
        KNodeTable {
            this_id: this_id,
            hash_size: hash_size,
            buckets: (0..hash_size).map(
                |_| KBucket::new(bucket_size)).collect(),
        }
    }

    pub fn buckets(&self) -> &Vec<KBucket<TId, TAddr>> {
        &self.buckets
    }

    #[inline]
    fn distance(id1: &TId, id2: &TId) -> TId {
        id1.bitxor(id2)
    }

    fn bucket_number(&self, id: &TId) -> usize {
        let diff = KNodeTable::<TId, TAddr>::distance(&self.this_id, id);
        debug_assert!(!diff.is_zero());
        let res = diff.bits() - 1;
        if res >= self.hash_size {
            panic!(format!("Distance between IDs {:?} and {:?} is {:?}, which is \
                    greater than the hash size ({:?})",
                           id, self.this_id, res, self.hash_size));
        }
        res
    }
}

impl<TId, TAddr> GenericNodeTable<TId, TAddr> for KNodeTable<TId, TAddr>
    where TId: GenericId,
          TAddr: Clone + Debug + Sync + Send {
    fn new(this_id: TId) -> KNodeTable<TId, TAddr> {
        KNodeTable::new_with_details(this_id, BUCKET_SIZE, DEFAULT_HASH_SIZE)
    }

    fn random_id(&self) -> TId {
        TId::gen(self.hash_size)
    }

    fn update(&mut self, node: &DHTNode<TId, TAddr>) -> bool {
        assert!(node.id != self.this_id);
        let bucket = self.bucket_number(&node.id);
        self.buckets[bucket].update(node)
    }

    fn find(&self, id: &TId, count: usize) -> Vec<DHTNode<TId, TAddr>> {
        debug_assert!(count > 0);
        assert!(*id != self.this_id);

        let mut data_copy: Vec<_> = self.buckets.iter().flat_map(|b| &b.data).map(|n| n.clone()).collect();
        data_copy.sort_by_key(|n| KNodeTable::<TId, TAddr>::distance(id, &n.id));
        data_copy[0..cmp::min(count, data_copy.len())].to_vec()
    }

    fn pop_oldest(&mut self) -> Vec<DHTNode<TId, TAddr>> {
        // For every full k-bucket, pop the last.
        // TODO(divius): TTL expiration?
        self.buckets.iter_mut()
            .filter(|b| { !b.data.is_empty() && b.size == b.data.len() })
            .map(|b| b.data.pop_front().unwrap())
            .collect()
    }
}

impl<TId, TAddr> KBucket<TId, TAddr>
    where TId: GenericId,
          TAddr: Clone + Debug {
    pub fn new(k: usize) -> KBucket<TId, TAddr> {
        assert!(k > 0);
        KBucket {
            data: VecDeque::new(),
            size: k
        }
    }

    pub fn update(&mut self, node: &DHTNode<TId, TAddr>) -> bool {
        if self.data.iter().any(|x| x.id == node.id) {
            self.update_position(node.clone());
            true
        } else if self.data.len() == self.size {
            false
        } else {
            self.data.push_back(node.clone());
            true
        }
    }

    pub fn find(&self, id: &TId, count: usize) -> Vec<DHTNode<TId, TAddr>> {
        let mut data_copy: Vec<_> = self.data.iter().map(|n| n.clone()).collect();
        data_copy.sort_by_key(|n| KNodeTable::<TId, TAddr>::distance(id, &n.id));
        data_copy[0..cmp::min(count, data_copy.len())].to_vec()
    }

    pub fn data(&self) -> &VecDeque<DHTNode<TId, TAddr>> {
        &self.data
    }
    pub fn size(&self) -> usize {
        self.size
    }

    fn update_position(&mut self, node: DHTNode<TId, TAddr>) {
        // TODO(divius): 1. optimize, 2. make it less ugly
        let mut new_data = VecDeque::with_capacity(self.data.len());
        new_data.extend(self.data.iter()
            .filter(|x| x.id != node.id)
            .map(|x| x.clone()));
        new_data.push_back(node.clone());
        self.data = new_data;
    }
}