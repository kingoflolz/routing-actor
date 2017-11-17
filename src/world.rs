use actix::{msgs, Actor, Address, Arbiter, Context, System, Handler, Response, ResponseType};

use petgraph::Graph;
use petgraph::graph::NodeIndex;

use spade::HasPosition;
use spade::rtree::RTree;

use node::Node;
use connection::Connection;


pub struct World {
    graph: Graph<Node, Connection>,
    rtree: RTree<MapNode>,
}

#[derive(Clone, Debug)]
pub struct MapNode {
    position: [f32; 2],
    graph_index: NodeIndex,
}

impl HasPosition for MapNode {
    type Point = [f32; 2];
    fn position(&self) -> [f32; 2] {
        self.position
    }
}

impl World {
    pub fn new() -> World {
        World {
            graph: Graph::new(),
            rtree: RTree::new()
        }
    }
}

impl Actor for World {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("World started");
    }
}