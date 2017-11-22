use actix::*;

use nalgebra::VectorN;
use nalgebra::U10;

use rand::distributions::{IndependentSample, Range};
use rand::thread_rng;

use world::World;
use connection::Connection;

pub type NC = VectorN<f32, U10>;

#[derive(Clone, Debug)]
pub struct NCNodeData {
    pub outgoing_vec: NC,
    pub incoming_vec: NC,
    pub learn_rate: f32,
}

impl NCNodeData {
    pub fn new() -> NCNodeData {
        let mut rng = thread_rng();
        let between = Range::new(0., 1.);
        NCNodeData {
            outgoing_vec: <NC>::from_fn(|_, _| between.ind_sample(&mut rng)),
            incoming_vec: <NC>::from_fn(|_, _| between.ind_sample(&mut rng)),
            learn_rate: 0.05
        }
    }
}

pub fn calc_update(a: NC, b: NC, actual: f32, learn_rate: f32) -> (NC, NC) {
    let diff = actual - a.dot(&b);

    let mut a_d = b;

    let mut n = 0;

    let diff = (diff).min(2.).max(-2.);

    for i in a_d.iter_mut() {
        *i = *i * learn_rate * diff - b[n].abs() * 0.01 * learn_rate;
        n += 1;
    }

    let mut b_d = a;

    n = 0;
    for i in b_d.iter_mut() {
        *i = *i * learn_rate * diff - a[n].abs() * 0.01 * learn_rate;
        n += 1;
    }

    (a_d, b_d)
}

#[derive(Message)]
struct MeasureMetric {
    quality: Connection
}

impl World {
    fn send_nc(&mut self, ctx: &mut Context<Self>) {}
}