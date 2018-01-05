// Copyright 2014 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

use rand;
use rand::Rng;

use std::hash::Hash;
use std::fmt::Debug;
use std::str::FromStr;
use std::net;

use futures::Future;

/// Generalization of num::BigUint, with hexadecimal encoding and decoding
pub trait GenericId: Hash + PartialEq + Eq + Ord + Clone + Send + Sync + Debug {
    fn bitxor(&self, other: &Self) -> Self;
    fn is_zero(&self) -> bool;
    fn bits(&self) -> usize;
    /// num::bigint::RandBigInt::gen_biguint
    fn gen(bit_size: usize) -> Self;
}

impl GenericId for u64 {
    fn bitxor(&self, other: &u64) -> u64 {
        self ^ other
    }
    fn is_zero(&self) -> bool {
        *self == 0
    }
    fn bits(&self) -> usize {
        (64 - self.leading_zeros()) as usize
    }
    fn gen(bit_size: usize) -> u64 {
        assert!(bit_size <= 64);
        if bit_size == 64 {
            rand::thread_rng().next_u64()
        } else {
            rand::thread_rng().gen_range(0, 1 << bit_size)
        }
    }
}

/// Trait representing table with known nodes.
///
/// Keeps some reasonable subset of known nodes passed to `update`.
pub trait GenericNodeTable: Send + Sync + Debug {
    /// Create new table
    fn new(node_id: u64) -> Self;
    /// Generate suitable random ID.
    fn random_id(&self) -> u64;
    /// Store or update node in the table.
    fn update(&mut self, node: &DHTNode) -> bool;
    /// Find given number of node, closest to given ID.
    fn find(&self, id: &u64, count: usize) -> Vec<DHTNode>;
    /// Pop expired or the oldest nodes from table for inspection.
    fn pop_oldest(&mut self) -> Vec<DHTNode>;
}

/// Structure representing a node in system.
///
/// Every node has an address (IP and port) and a numeric ID, which is
/// used to calculate metrics and look up data.
#[derive(Clone, Debug)]
pub struct DHTNode where {
    /// Network address of the node.
    pub route: Vec<u64>,
    /// ID of the node.
    pub id: u64
}

impl DHTNode {
    pub fn prepend(&mut self, e: &mut Vec<u64>) {
        self.route.append(e)
    }
}

/// Trait representing the API.
pub trait GenericAPI<TId, TAddr>
    where TId: GenericId {
    /// Value type.
    type TValue: Send + Sync + Clone;
    /// Ping a node.
    fn ping<F>(&mut self, node: &DHTNode) -> Future<Item=DHTNode, Error=()>;
    /// Return nodes clothest to the given id.
    fn find_node<F>(&mut self, id: &TId) -> Future<Item=Vec<DHTNode>, Error=()>;
    /// Find a value in the network.
    ///
    /// Either returns a value or several clothest nodes.
    fn find_value<F>(&mut self, id: &TId) -> Future<Item=(Option<Self::TValue>, Vec<DHTNode>), Error=()>;
    /// Store a value on a node.
    fn store(&mut self, node: &DHTNode, id: &TId, value: Self::TValue);
}