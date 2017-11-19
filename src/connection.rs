#[derive(Clone, Debug)]
pub struct Connection {
    pub latency: f32,
    pub bandwidth: f32,
    pub packet_loss: f32,
}