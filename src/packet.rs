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
    pub des: u64,
    pub route: Vec<u64>,
    //list of hops
    pub data: T
}

impl<T: PacketData + Clone + Send + ResponseType> ResponseType for Packet<T> where T::Error: Debug, T::Item: Send, T::Error: Send {
    type Item = T::Item;
    type Error = T::Error;
}

#[derive(Clone)]
pub struct SearchPacket<T: SearchPacketData + Clone> {
    des: u64,
    nc: NC,
    data: T
}

impl<T: SearchPacketData + Clone + Send + ResponseType> ResponseType for SearchPacket<T> {
    type Item = T::Item;
    type Error = T::Error;
}