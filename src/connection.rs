struct Quality {
    latency: f32,
    bandwidth: f32,
    packet_loss: f32,
}

pub struct Connection {
    quality: Quality
}