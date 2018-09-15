#[macro_use]
extern crate myrpc4rs;
#[macro_use]
extern crate serde_derive;

use myrpc4rs::serialization::Serializer;
use myrpc4rs::client::MyRPCClient;
use myrpc4rs::error::Error;
use std::collections::HashSet;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Node {
    index: usize,
    path_set: HashSet<Path>,
}

impl Node {
    fn new(index: usize) -> Self {
        Self {
            index,
            path_set: HashSet::new(),
        }
    }
    fn add_path(&mut self, next: Path) {
        assert_eq!(next.from, self.index);
        self.path_set.insert(next);
    }
    fn get_shortest_patch(&self) -> &Path {
        let mut shortest_path = None;
        for path in &self.path_set {
            if shortest_path == None {
                shortest_path = Some(path);
                continue;
            }

            if (shortest_path.unwrap() > path) {
                shortest_path = Some(path);
            }
        };
        shortest_path.unwrap()
    }
    fn get_path_set(&self) -> &HashSet<Path> {
        &self.path_set
    }
}

#[derive(Hash, Eq, Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Path {
    from: usize,
    to: usize,
    length: usize,
    busyness: usize,
}

impl Path {
    fn new(from: usize, to: usize, length: usize, busyness: usize) -> Path {
        Path {
            from,
            to,
            length,
            busyness,
        }
    }

    fn time(&self) -> usize {
        self.length + self.busyness
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Path) -> Option<Ordering> {
        if self.time() == other.time() {
            return Some(Ordering::Equal);
        }
        if self.time() < other.time() {
            return Some(Ordering::Less);
        }
        if self.time() > other.time() {
            return Some(Ordering::Greater);
        }
        None
    }
}

fn main(){

    let mut nodes = Vec::new();
    for i in 0..6usize {
        nodes.push(Node::new(i));
    }
    nodes[0].add_path(Path::new(0, 2, 2, 8));
    nodes[0].add_path(Path::new(0, 4, 15, 15));
    nodes[0].add_path(Path::new(0, 5, 50, 50));
    nodes[1].add_path(Path::new(1, 2, 5, 0));
    nodes[2].add_path(Path::new(2, 3, 5, 45));
    nodes[3].add_path(Path::new(3, 5, 5, 5));
    nodes[4].add_path(Path::new(4, 3, 15, 5));
    nodes[4].add_path(Path::new(4, 5, 15, 45));
    let mut client = MyRPCClient::new("127.0.0.1:6181".parse().unwrap());

    let resp = myrpc_call_sync!(client,find_shortest_path,&nodes,&0usize,&5usize;<(Option<usize>, Vec<usize>)>);
    println!("{:?}", resp);
}