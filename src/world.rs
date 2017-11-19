use actix::{msgs, Actor, Address, Arbiter, Context, Handler, Response, ResponseType, SyncAddress};
use actix::dev::AsyncContextApi;

use tokio_core::reactor::Timeout;
use std::time::Duration;
use futures::future::Future;

use rand::thread_rng;
use rand::distributions::{Weighted, WeightedChoice, Sample, Range};

use petgraph::stable_graph::StableDiGraph;
use petgraph::graph::NodeIndex;

use spade::HasPosition;
use spade::rtree::RTree;

use node::Node;
use connection::Connection;

struct GraphNode {
    address: Option<SyncAddress<Node>>,
    thread: SyncAddress<Arbiter>
}

pub struct World {
    graph: StableDiGraph<GraphNode, Connection>,
    rtrees: Vec<RTree<MapNode>>,
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
            graph: StableDiGraph::new(),
            rtrees: Vec::new(),
            threads: threads.clone(),
        }
    }
}

fn connection(a: &[f32; 2], b: &[f32; 2], level: usize) -> Connection {
    let l = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt() / 3e5;
    Connection { latency: l, packet_loss: 0.01, bandwidth: 10000. }
}

impl Actor for World {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("World started");

        let mut rng = thread_rng();

        let mut area = Range::new(-1e6, 1e6);
        let mut latency_jitter = Range::new(0., 10.);

        // number of upstream providers
        let mut items = vec!(
            Weighted { weight: 2, item: 1 },
            Weighted { weight: 10, item: 2 },
            Weighted { weight: 3, item: 3 });
        let mut wc = WeightedChoice::new(&mut items);

        // how deep the hierarchy is
        let levels = 6;

        // number of fully connected core nodes
        let core = 8;

        // number of nodes under another node, on average
        let spread = [1, 20, 20, 5, 6, 10];

        // number of connections on the same level
        let conn = [core - 1, 3, 3, 3, 3, 3];

        let mut num_nodes = core;
        // add core nodes
        for level in 0..levels {
            num_nodes *= spread[level];
            self.rtrees.push(RTree::new());

            // add nodes
            for i in 0..num_nodes {
                let p = [area.sample(&mut rng), area.sample(&mut rng)];

                let graph_index = self.graph.add_node(GraphNode { address: None, thread: self.threads[i % 4].clone() });

                self.rtrees[0].insert(MapNode { position: p, graph_index });
            }

            // core connections
            if level == 0 {
                for i in self.rtrees[level].iter() {
                    for j in self.rtrees[level].iter() {
                        if i.graph_index != j.graph_index {
                            let connection = connection(&i.position, &j.position, 0);
                            self.graph.add_edge(i.graph_index, j.graph_index, connection.clone());
                            self.graph.add_edge(j.graph_index, i.graph_index, connection.clone());
                        }
                    }
                }
            }
        }

        println!("Starting graph generation...");

        println!("Completed graph generation...");

        println!("Nodes Added");
        ctx.address_cell().sync_address().send(Wake);
        //Arbiter::system().send(msgs::SystemExit(0));
    }
}

struct Wake;

impl ResponseType for Wake {
    type Item = ();
    type Error = ();
}

impl Handler<Wake> for World {
    // runs every ms
    fn handle(&mut self, _msg: Wake, ctx: &mut Context<Self>) -> Response<Self, Wake> {
        let addr = ctx.address_cell().unsync_address();
        Arbiter::handle().spawn(Timeout::new(Duration::new(0, 1_000_000), Arbiter::handle()).unwrap()
            .then(move |_| {
                addr.send(Wake);
                Ok(())
            }));

        for i in 0..1_000 {
            self.threads[i % 4].send::<msgs::Execute>(msgs::Execute::new(move || {
                Node::new(NodeIndex::new(i)).start::<Address<Node>>();
                Ok(())
            }))
        }

        Self::reply(())
    }
}