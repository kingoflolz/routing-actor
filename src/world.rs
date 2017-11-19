use actix::{msgs, Actor, Address, Arbiter, Context, System, Handler, Response, ResponseType, SyncAddress};

use petgraph::Graph;
use petgraph::graph::NodeIndex;

use spade::HasPosition;
use spade::rtree::RTree;

use node::Node;
use connection::Connection;


pub struct World {
    graph: Graph<Node, Connection>,
    rtree: RTree<MapNode>,
    threads: Vec<SyncAddress<Arbiter>>
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
    pub fn new(threads: &Vec<SyncAddress<Arbiter>>) -> World {
        World {
            graph: Graph::new(),
            rtree: RTree::new(),
            threads: threads.clone(),
        }
    }
}

impl Actor for World {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("World started");

        for i in 0..10_000_000 {
            self.threads[i % 4].send::<msgs::Execute>(msgs::Execute::new(move || {
                Node::new(NodeIndex::new(i)).start::<Address<Node>>();
                Ok(())
            }))
        }

        println!("Added nodes");
        Arbiter::system().send(msgs::SystemExit(0));
    }
}
