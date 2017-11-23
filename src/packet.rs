use actix::*;
use nc::NC;
use node::Node;

use std::fmt::Debug;

pub trait PacketData {
    fn process(packet: &Packet<Self>, node: &mut Node) -> Response<Node, Packet<Self>> where Self: Sized + Clone + Send + ResponseType, <Self as ResponseType>::Item: Send, <Self as ResponseType>::Error: Send;
}

pub trait SearchPacketData {
    fn process(packet: &SearchPacket<Self>, node: &mut Node) -> Response<Node, SearchPacket<Self>> where Self: Sized + Clone + Send + ResponseType;
}


#[derive(Clone)]
pub struct Packet<T: PacketData + Clone + Send + ResponseType> {
    pub from: u64,
    pub des: u64,
    pub route: Vec<u64>,
    //list of hops
    pub data: T
}

pub struct PacketRouteData {
    pub from: u64,
    pub des: u64,
    pub route: Vec<u64>,
}

impl<T: PacketData + Clone + Send + ResponseType> Packet<T> {
    pub fn reverse(&self) -> PacketRouteData {
        let mut r = self.clone();
        r.route.reverse();
        PacketRouteData {
            from: r.des,
            des: r.from,
            route: r.route
        }
    }

    pub fn new(r: PacketRouteData, d: T) -> Packet<T> {
        Packet { data: d, route: r.route, des: r.des, from: r.from }
    }
}

impl<T: PacketData + Clone + Send + ResponseType> ResponseType for Packet<T> where T::Error: Debug, T::Item: Send, T::Error: Send {
    type Item = T::Item;
    type Error = T::Error;
}

#[derive(Clone)]
pub struct SearchPacket<T: SearchPacketData + Clone> {
    pub from: u64,
    des: u64,
    nc: NC,
    data: T
}

impl<T: SearchPacketData + Clone + Send + ResponseType> ResponseType for SearchPacket<T> {
    type Item = T::Item;
    type Error = T::Error;
}