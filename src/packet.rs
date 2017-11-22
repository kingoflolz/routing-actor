use actix::*;
use nc::NC;
use node::Node;

pub trait PacketData {
    fn process(packet: &Packet<Self>, node: &mut Node) where Self: Sized + Clone + Send;
}

pub trait SearchPacketData {
    fn process(packet: &SearchPacket<Self>, node: &mut Node) where Self: Sized + Clone + Send;
}


#[derive(Message, Clone)]
pub struct Packet<T: PacketData + Clone + Send> {
    pub des: u64,
    pub route: Vec<u64>,
    //list of hops
    pub data: T
}

#[derive(Message, Clone)]
pub struct SearchPacket<T: SearchPacketData + Clone> {
    des: u64,
    nc: NC,
    data: T
}
