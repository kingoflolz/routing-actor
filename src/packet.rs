use actix::*;
use nc::NC;
use node::Node;

pub trait PacketData {
    fn process(packet: &Packet<Self>, node: &mut Node) where Self: Sized;
    fn process_search(packet: &SearchPacket<Self>, node: &mut Node) where Self: Sized;
}

#[derive(Message)]
pub struct Packet<T: PacketData> {
    des: u64,
    route: Vec<u64>,
    //list of hops
    pub data: T
}

#[derive(Message)]
pub struct SearchPacket<T: PacketData> {
    des: u64,
    nc: NC,
    data: T
}
