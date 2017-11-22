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

impl GenericId for Vec<u8> {
    fn bitxor(&self, other: &Vec<u8>) -> Vec<u8> {
        self.iter().zip(other.iter()).map(|(digit1, digit2)| digit1 ^ digit2).collect()
    }
    fn is_zero(&self) -> bool {
        self.iter().all(|digit| *digit == 0)
    }
    fn bits(&self) -> usize {
        let mut bits = self.len() * 8;
        for digit in self {
            if *digit == 0 {
                bits -= 8;
            } else {
                return bits - digit.leading_zeros() as usize;
            }
        }
        assert!(bits == 0);
        0
    }
    fn gen(bit_size: usize) -> Vec<u8> {
        let nb_full_digits = bit_size / 8;
        let nb_bits_partial_digit = bit_size % 8;
        let mut rng = rand::thread_rng();
        if nb_bits_partial_digit == 0 {
            let mut res = vec![0u8; nb_full_digits];
            rng.fill_bytes(&mut res);
            res
        } else {
            let mut res = vec![0u8; nb_full_digits + 1];
            let first_digit = rng.gen_range(0, 1 << (nb_bits_partial_digit - 1));
            res[0] = first_digit;
            rng.fill_bytes(&mut res[1..nb_full_digits + 1]);
            res
        }
    }
}

/// Trait representing table with known nodes.
///
/// Keeps some reasonable subset of known nodes passed to `update`.
pub trait GenericNodeTable<TId, TAddr>: Send + Sync
    where TId: GenericId {
    /// Generate suitable random ID.
    fn random_id(&self) -> TId;
    /// Store or update node in the table.
    fn update(&mut self, node: &Node<TId, TAddr>) -> bool;
    /// Find given number of node, closest to given ID.
    fn find(&self, id: &TId, count: usize) -> Vec<Node<TId, TAddr>>;
    /// Pop expired or the oldest nodes from table for inspection.
    fn pop_oldest(&mut self) -> Vec<Node<TId, TAddr>>;
}

/// Structure representing a node in system.
///
/// Every node has an address (IP and port) and a numeric ID, which is
/// used to calculate metrics and look up data.
#[derive(Clone, Debug)]
pub struct Node<TId, TAddr> {
    /// Network address of the node.
    pub address: TAddr,
    /// ID of the node.
    pub id: TId
}

/// Trait representing the API.
pub trait GenericAPI<TId, TAddr>
    where TId: GenericId {
    /// Value type.
    type TValue: Send + Sync + Clone;
    /// Ping a node.
    fn ping<F>(&mut self, node: &Node<TId, TAddr>, callback: F)
        where F: FnOnce(&Node<TId, TAddr>, bool);
    /// Return nodes clothest to the given id.
    fn find_node<F>(&mut self, id: &TId, callback: F)
        where F: FnOnce(Vec<Node<TId, TAddr>>);
    /// Find a value in the network.
    ///
    /// Either returns a value or several clothest nodes.
    fn find_value<F>(&mut self, id: &TId, callback: F)
        where F: FnOnce(Option<Self::TValue>, Vec<Node<TId, TAddr>>);
    /// Store a value on a node.
    fn store(&mut self, node: &Node<TId, TAddr>, id: &TId, value: Self::TValue);
}