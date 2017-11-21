use actix::*;
use actix::dev::AsyncContextApi;

use tokio_core::reactor::Timeout;
use std::time::Duration;
use futures::future::Future;

use rand::thread_rng;
use rand::distributions::{Weighted, WeightedChoice, Sample, Range};

use petgraph::stable_graph::StableDiGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;

use spade::HasPosition;
use spade::rtree::RTree;

use node::{Node, HelloNode};
use connection::Connection;

struct GraphNode {
    address: Option<SyncAddress<Node>>,
    thread: usize,
}

pub struct World {
    graph: StableDiGraph<GraphNode, Connection>,
    rtrees: Vec<RTree<MapNode>>,
    threads: Vec<SyncAddress<Arbiter>>,
    active: usize,
    pending: usize,
    adding: bool,
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


impl Default for World {
    fn default() -> World {
        let mut threads: Vec<SyncAddress<Arbiter>> = Vec::new();
        for i in 0..4 {
            threads.push(Arbiter::new(format!("Core {}", i)))
        }
        World::new(&threads)
    }
}

impl World {
    pub fn new(threads: &Vec<SyncAddress<Arbiter>>) -> World {
        World {
            graph: StableDiGraph::new(),
            rtrees: Vec::new(),
            threads: threads.clone(),
            active: 0,
            pending: 0,
            adding: true,
        }
    }

    fn activate_node(&mut self, i: NodeIndex) {
        let core = &self.threads[self.graph[i].thread];
        core.send::<msgs::Execute>(msgs::Execute::new(move || {
            Node::new(i).start::<Address<Node>>();
            Ok(())
        }))
    }

    // add 5% of new nodes per epoch
    fn add_nodes(&mut self) -> bool {
        for i in self.active..self.active + (1 + (self.active / 20)) {
            if i < self.graph.node_count() {
                self.activate_node(NodeIndex::new(i));
                self.pending += 1;
            } else {
                return false
            }
        }
        return true
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

        let mut area = Range::new(-1e8, 1e8);

        // number of upstream providers
        let mut items = vec!(
            Weighted { weight: 10, item: 1 },
            Weighted { weight: 2, item: 2 },
            Weighted { weight: 1, item: 3 });
        let mut wc = WeightedChoice::new(&mut items);

        // number of fully connected core nodes
        let core = 8;

        // 1 million nodes (approx)
        // how deep the hierarchy is
        let levels = 6;
        // number of nodes under another node, on average
        let spread = [1, 20, 20, 5, 6, 10];
        // number of connections on the same level
        let conn = [core, 2, 2, 2, 2, 2];

        // // 10k nodes (approx)
        // let levels = 4;
        // // number of nodes under another node, on average
        // let spread = [1, 20, 5, 10];
        // // number of connections on the same level
        // let conn = [core, 2, 2, 2];

        let mut num_nodes = core;
        // add core nodes

        for level in 0..levels {
            println!("Starting graph generation... (level {})", level);
            num_nodes *= spread[level];
            self.rtrees.push(RTree::new());

            // add nodes
            for i in 0..num_nodes {
                let p = [area.sample(&mut rng), area.sample(&mut rng)];

                let graph_index = self.graph.add_node(GraphNode { address: None, thread: i % 4 });

                self.rtrees[level].insert(MapNode { position: p, graph_index });
            }

            for i in self.rtrees[level].iter() {
                // create same level connections
                for j in self.rtrees[level].nearest_n_neighbors(&i.position, conn[level]) {
                    if i.graph_index != j.graph_index {
                        let connection = connection(&i.position, &j.position, 0);
                        self.graph.add_edge(i.graph_index, j.graph_index, connection.clone());
                        self.graph.add_edge(j.graph_index, i.graph_index, connection.clone());
                    }
                }

                // create upstream connections
                if level > 0 {
                    let upstreams = wc.sample(&mut rng);
                    for j in self.rtrees[level - 1].nearest_n_neighbors(&i.position, upstreams) {
                        let connection = connection(&i.position, &j.position, 0);
                        self.graph.add_edge(i.graph_index, j.graph_index, connection.clone());
                        self.graph.add_edge(j.graph_index, i.graph_index, connection.clone());
                        // take on the core of upstream
                        self.graph[i.graph_index].thread = self.graph[j.graph_index].thread;
                    }
                }
            }
        }

        println!("Completed graph generation...");

        println!("{} nodes Added", self.graph.node_count());

        // self.add_nodes();

        // Arbiter::system().send(msgs::SystemExit(0));
    }
}

impl Supervised for World {}

impl SystemService for World {
    fn service_started(&mut self, ctx: &mut Context<Self>) {
        println!("Service started");
    }
}

// sent by node to world to notify that it has been initialised
pub struct HelloWorld {
    pub addr: SyncAddress<Node>,
    pub graph_index: NodeIndex,
}

impl ResponseType for HelloWorld {
    type Item = ();
    type Error = ();
}

impl Handler<HelloWorld> for World {
    fn handle(&mut self, msg: HelloWorld, ctx: &mut Context<Self>) -> Response<Self, HelloWorld> {
        self.graph[msg.graph_index].address = Some(msg.addr.clone());
        self.active += 1;
        self.pending -= 1;
        for i in self.graph.edges(msg.graph_index) {
            if let Some(ref addr) = self.graph[i.target()].address {
                addr.send(HelloNode { addr: msg.addr.clone(), reply: false, connection: i.weight().clone() })
            }
        }
        Self::reply(())
    }
}

pub struct Wake;

impl ResponseType for Wake {
    type Item = ();
    type Error = ();
}

impl Handler<Wake> for World {
    // runs every ms
    fn handle(&mut self, _msg: Wake, ctx: &mut Context<Self>) -> Response<Self, Wake> {
        let addr: Address<_> = ctx.address();
        ctx.notify(Wake, Duration::new(0, 100_000));

        if self.pending == 0 && self.adding {
            self.adding = self.add_nodes();
            println!("added more nodes, total: {}", self.active);
        }

        if !self.adding {
            Arbiter::system().send(msgs::SystemExit(0));
        }

        Self::reply(())
    }
}

pub struct AddThread {
    pub thread: SyncAddress<Arbiter>,
}

impl ResponseType for AddThread {
    type Item = ();
    type Error = ();
}

impl Handler<AddThread> for World {
    fn handle(&mut self, msg: AddThread, ctx: &mut Context<Self>) -> Response<Self, AddThread> {
        self.threads.push(msg.thread);
        Self::reply(())
    }
}