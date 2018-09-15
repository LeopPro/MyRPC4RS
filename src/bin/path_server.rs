#[macro_use]
extern crate myrpc4rs;
#[macro_use]
extern crate serde_derive;

use myrpc4rs::server::MyRPCServer;
use myrpc4rs::serialization::Serializer;
use std::collections::HashSet;
use std::cmp::Ordering;
use std::usize;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Node {
    index: usize,
    path_set: HashSet<Path>,
}

impl Node {
    #[allow(dead_code)]
    fn new(index: usize) -> Self {
        Self {
            index,
            path_set: HashSet::new(),
        }
    }

    #[allow(dead_code)]
    fn add_path(&mut self, next: Path) {
        assert_eq!(next.from, self.index);
        self.path_set.insert(next);
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
    #[allow(dead_code)]
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


fn find_shortest_path(nodes: Vec<Node>, from: usize, to: usize)
                      -> (Option<usize>, Vec<usize>) {
    let mut shortest_time = Vec::with_capacity(nodes.len());
    let mut shortest_path = Vec::with_capacity(nodes.len());
    let mut shortest_set = Vec::with_capacity(nodes.len());
    for _i in 0..nodes.len() {
        shortest_time.push(None);
        shortest_path.push(None);
        shortest_set.push(false);
    }
    shortest_time[from] = Some(0usize);
    shortest_path[from] = Some(from);
    shortest_set[from] = true;
    let mut current_node = &nodes[from];

    loop {
        // 到 to，没有最小的可以换
        for path in current_node.get_path_set() {
            let new_time = path.time() + shortest_time[current_node.index].unwrap();
            if shortest_time[path.to].is_none() {
                shortest_time[path.to] = Some(new_time);
                shortest_path[path.to] = Some(current_node.index);
            } else {
                if new_time < shortest_time[path.to].unwrap() {
                    shortest_time[path.to] = Some(new_time);
                    shortest_path[path.to] = Some(current_node.index);
                }
            }
        }
        let mut min_index = 0usize;
        let mut min_time = usize::MAX;
        let mut set_num = 0;
        for i in 0..nodes.len() {
            if !shortest_set[i] {
                set_num += 1;
            }
            if !shortest_set[i] && shortest_time[i].is_some() && shortest_time[i].unwrap() < min_time {
                min_time = shortest_time[i].unwrap();
                min_index = i;
            }
        }
        if set_num == 0 {
            return (shortest_time[to], get_shortest_path(from, to, &shortest_path));
        }
        current_node = &nodes[min_index];
        shortest_set[min_index] = true;
        if current_node.index == to {
            return (shortest_time[to], get_shortest_path(from, to, &shortest_path));
        }
    }
}

fn get_shortest_path(from: usize, to: usize, shortest_path: &Vec<Option<usize>>) -> Vec<usize> {
    let mut result = Vec::new();
    let mut current = match shortest_path[to] {
        Some(v) => v,
        None => return vec![],
    };
    result.push(to);
    loop {
        result.push(current);
        current = shortest_path[current].unwrap();
        if current == from {
            result.push(from);
            result.reverse();
            return result;
        }
    }
}

fn main() {
    let mut myrpc = MyRPCServer::new("127.0.0.1:6181".parse().unwrap());
    myrpc_function!(myrpc,find_shortest_path,nodes<Vec<Node>>,from<usize>,to<usize>,{
        find_shortest_path(nodes, from, to)
    });
    myrpc.start_server();
}


#[cfg(test)]
mod tests {
    use Path;
    use Node;
    use find_shortest_path;

    #[test]
    fn find_shortest_path_test() {
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
        let result = find_shortest_path(nodes, 0, 5);
        println!("{:?}", result);
    }

    #[test]
    fn path_test() {
        let path1 = Path::new(1, 2, 10, 10);
        let path2 = Path::new(1, 2, 20, 10);
        assert!(path1 < path2);
    }
}