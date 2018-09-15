use futures::{Future, Stream};
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use std::net::SocketAddr;
use futures::Async;
use tokio_core::net::TcpStream;
use bytes::BytesMut;
use std::io;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};
use futures::future;
use std::cell::RefCell;
use server::Processes;
use std::rc::Rc;
use common::Request;
use serialization::Serializer;
use common::Response;
use error::Error;
use futures::Poll;
use bytes::BufMut;
use tokio_io::AsyncRead;
use tokio_io::AsyncWrite;
use bytes::IntoBuf;
use serialization::BincodeSerializer;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::collections::HashMap;
use byteorder::WriteBytesExt;
use std::u32;


pub struct Server {
    socket_addr: SocketAddr,
}

impl Server {
    pub fn new(socket_addr: SocketAddr) -> Self {
        Self {
            socket_addr,
        }
    }
    pub fn start(&mut self, processes: Rc<Processes>) {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let listener = TcpListener::bind(&self.socket_addr, &handle).unwrap();
//        let process_cell = RefCell::new(process);
        let server = listener.incoming().for_each(|(socket, _)| {
            let packages = ServerPackages::new(socket, Rc::clone(&processes));
//            let pro = process_cell.borrow_mut();
            let package_handler = packages.into_future()
                .then(|_| {
                    future::ok(())
                });
            handle.spawn(package_handler);
            Ok(())
        });
        core.run(server).unwrap();
    }
}

pub struct Client {
    socket_addr: SocketAddr,
    serializer: Rc<BincodeSerializer>,
}

impl Client {
    pub fn new(socket_addr: SocketAddr, serializer: Rc<BincodeSerializer>) -> Self {
        Self {
            socket_addr,
            serializer,
        }
    }
    pub fn start(&self, receiver: &Arc<Receiver<(Request, Box<FnMut(&Rc<BincodeSerializer>, &Response) + Send>)>>) {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let socket = TcpStream::connect(&self.socket_addr, &handle).and_then(|socket| {
            let mut packages = ClientPackages::new(socket, Arc::clone(receiver), Rc::clone(&self.serializer));
            loop {
                match packages.poll() {
                    Ok(result) => match result {
                        Async::NotReady => continue,
                        Async::Ready(_) => break,
                    },
                    Err(e) => { println!("ERR:{:?}", e) }
                }
                break;
            }
            Ok(())
        });
        core.run(socket).unwrap();
    }
}

pub struct ServerPackages {
    socket: TcpStream,
    read_buffer: BytesMut,
    write_buffer: RefCell<BytesMut>,
    processes: Rc<Processes>,
}

impl Stream for ServerPackages {
    type Item = BytesMut;
    type Error = io::Error;

    fn poll(&mut self) -> Result<Async<Option<<Self as Stream>::Item>>, <Self as Stream>::Error> {
//        let _ = self.poll_flush()?;
        loop {
            let sock_closed = self.fill_read_buf()?.is_ready();
            if self.read_buffer.len() < 4 {
                if sock_closed {
                    return Ok(Async::Ready(None));
                } else {
                    return Ok(Async::NotReady);
                }
            }
            let mut package_length = [0; 4];
            for i in 0..4 {
                package_length[i] = self.read_buffer[i];
            }
            let mut rdr = Cursor::new(package_length);
            let package_length = rdr.read_u32::<BigEndian>().unwrap() as usize;
            if self.read_buffer.len() < 4 + package_length {
                return Ok(Async::NotReady);
            }
            let mut package = self.read_buffer.split_to(4 + package_length);
            let package = package.split_off(4);
            {
                let response = self.process(&package);
                let serializer = self.get_processes().get_serializer();
                let result = match serializer.serialize(&response) {
                    Ok(bytes) => {
                        bytes
                    }
                    Err(_) => { /*不应该发生*/panic!("should not happend") }
                };
                let mut length = vec![];
                length.write_u32::<BigEndian>(result.len() as u32).unwrap();
                self.write(&length);
                self.write(&result);
            }
            let _ = self.poll_flush()?;
        }
    }
}

impl ServerPackages {
    fn new(socket: TcpStream, processes: Rc<Processes>) -> Self {
        Self {
            socket,
            read_buffer: BytesMut::new(),
            write_buffer: RefCell::new(BytesMut::new()),
            processes,
        }
    }
    fn fill_read_buf(&mut self) -> Result<Async<()>, io::Error> {
        loop {
            self.read_buffer.reserve(1024);
            let n = try_ready!(self.socket.read_buf(&mut self.read_buffer));
            if n == 0 {
                return Ok(Async::Ready(()));
            }
        }
    }

    fn get_processes(&self) -> &Rc<Processes> {
        &self.processes
    }

    fn write(&self, line: &[u8]) {
        let mut write_buffer = self.write_buffer.borrow_mut();
        write_buffer.reserve(line.len());
        write_buffer.put(line);
    }

    fn poll_flush(&mut self) -> Poll<(), io::Error> {
        let mut write_buffer = self.write_buffer.borrow_mut();
        while !write_buffer.is_empty() {
            let n = try_ready!(self.socket.write_buf(&mut (&*write_buffer).into_buf()));
            let _ = write_buffer.split_to(n);
        }
        Ok(Async::Ready(()))
    }

    fn process(&self, package: &BytesMut) -> Response {
        let processes = self.get_processes();
        let serializer = processes.get_serializer();
        let request: Request = match serializer.deserialize(&package[..]) {
            Ok(t) => t,
            Err(_) => { return Response::err_unknow_request(Error::ParamDeserializeFail); }
        };
        match processes.execute_function(&request.name, &request.params) {
            Ok(result) => Response::from(request, result),
            Err(err) => Response::err(request, err),
        }
    }
}


pub struct ClientPackages {
    socket: TcpStream,
    read_buffer: BytesMut,
    write_buffer: RefCell<BytesMut>,
    receiver: Arc<Receiver<(Request, Box<FnMut(&Rc<BincodeSerializer>, &Response) + Send>)>>,
    request_map: HashMap<u32, (Request, Box<FnMut(&Rc<BincodeSerializer>, &Response) + Send>)>,
    serializer: Rc<BincodeSerializer>,
    stoping: bool,
}

impl Stream for ClientPackages {
    type Item = BytesMut;
    type Error = io::Error;

    fn poll(&mut self) -> Result<Async<Option<<Self as Stream>::Item>>, <Self as Stream>::Error> {
//        let _ = self.poll_flush()?;
        if self.receive().is_none() {
            self.stoping = true;
            return Ok(Async::NotReady);
        }
        if self.stoping && self.request_map.len() == 0 {
            return Ok(Async::Ready(None));
        }
        let sock_closed = self.fill_read_buf()?.is_ready();
        if self.read_buffer.len() < 4 {
            if sock_closed {
                return Ok(Async::Ready(None));
            } else {
                return Ok(Async::NotReady);
            }
        }
        let mut package_length = [0; 4];
        for i in 0..4 {
            package_length[i] = self.read_buffer[i];
        }
        let mut rdr = Cursor::new(package_length);
        let package_length = rdr.read_u32::<BigEndian>().unwrap() as usize;
        if self.read_buffer.len() < 4 + package_length {
            return Ok(Async::NotReady);
        }
        let mut package = self.read_buffer.split_to(4 + package_length);
        let package = package.split_off(4);
        self.process(&package);
        return Ok(Async::NotReady);
    }
}

impl ClientPackages {
    fn new(socket: TcpStream,
           receiver: Arc<Receiver<(Request, Box<FnMut(&Rc<BincodeSerializer>, &Response) + Send>)>>,
           serializer: Rc<BincodeSerializer>) -> Self {
        Self {
            socket,
            read_buffer: BytesMut::new(),
            write_buffer: RefCell::new(BytesMut::new()),
            receiver,
            request_map: HashMap::new(),
            serializer,
            stoping: false,
        }
    }
    fn fill_read_buf(&mut self) -> Result<Async<()>, io::Error> {
        loop {
            self.read_buffer.reserve(1024);
            let n = try_ready!(self.socket.read_buf(&mut self.read_buffer));
            if n == 0 {
                return Ok(Async::Ready(()));
            }
        }
    }

    fn write(&self, line: &[u8]) {
        let mut write_buffer = self.write_buffer.borrow_mut();
        write_buffer.reserve(line.len());
        write_buffer.put(line);
    }

    fn poll_flush(&mut self) -> Poll<(), io::Error> {
        let mut write_buffer = self.write_buffer.borrow_mut();
        while !write_buffer.is_empty() {
            let n = try_ready!(self.socket.write_buf(&mut (&*write_buffer).into_buf()));
            assert!(n > 0);
            let _ = write_buffer.split_to(n);
        }
        Ok(Async::Ready(()))
    }

    fn process(&mut self, package: &BytesMut) {
        let serializer = &self.serializer;
        let response: Response = match serializer.deserialize(&package[..]) {
            Ok(t) => t,
            Err(_) => { panic!("deserialize fail"); }
        };
        if response.id == u32::MAX {
            println!("ERR:服务器反序列化失败");
            return;
        }
        {
            let (_request, callback) = self.request_map.get_mut(&response.id).unwrap();
            callback(serializer, &response);
        }
        self.request_map.remove(&response.id);
    }

    fn receive(&mut self) -> Option<()> {
        loop {
            if let Ok(result) = self.receiver.try_recv() {
                if result.0.id == u32::MAX && &result.0.name == "stop" {
                    return None;
                }
                let bytes = self.serializer.serialize(&result.0).unwrap();
                self.request_map.insert(result.0.id, result);
                let mut length = vec![];
                length.write_u32::<BigEndian>(bytes.len() as u32).unwrap();
                self.write(&length);
                self.write(&bytes);
                self.poll_flush().unwrap();
            } else {
                return Some(());
            }
        };
    }
}

trait PackageHandler {
    fn handle(&self, package: BytesMut);
}


#[cfg(test)]
mod tests {
    use net::Server;
    use std::net::TcpStream;
    use std::thread;
    use std::io::Write;
    use std::io::Read;
    use std::net::SocketAddr;
    use byteorder::BigEndian;
    use byteorder::WriteBytesExt;
    use serialization::BincodeSerializer;
    use common::Request;
    use serialization::Serializer;
    use server::Processes;
    use std::rc::Rc;
    use net::Client;
    use std::sync::mpsc;
    use std::sync::mpsc::Sender;
    use common::Response;
    use std::sync::Arc;
    use std::u32;
    use std::time::Duration;

    //    #[test]
    #[allow(dead_code)]
    fn start_server() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let mut server = Server::new(addr);
        let processes = Processes::new(Rc::new(BincodeSerializer::new()));
        processes.insert_function(String::from("print"), |serializer, params| {
            let mut param1: u32 = serializer.deserialize(&params[0]).unwrap();
            param1 += 666;
            println!("{}", param1);
            Ok(vec![6, 6, 6])
        });
        server.start(Rc::new(processes));
    }

//    #[test]
    #[allow(dead_code)]
    fn client() {
        let request = Request {
            id: 1,
            name: String::from("print"),
            params: vec![BincodeSerializer::new().serialize(&123).unwrap(),
                         BincodeSerializer::new().serialize(&456).unwrap()],
        };
        let send = BincodeSerializer::new().serialize(&request).unwrap();

        let mut wtr = vec![];
        wtr.write_u32::<BigEndian>(send.len() as u32).unwrap();
        println!("{:?}", wtr);
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let mut stream = TcpStream::connect(&addr).unwrap();
        loop {
            let msg = wtr.as_ref();
            thread::sleep(Duration::from_secs(1));
            let _ = stream.write(&msg).unwrap();
            thread::sleep(Duration::from_secs(1));
            let msg = send.as_ref();
            let _ = stream.write(&msg).unwrap();
            thread::sleep(Duration::from_secs(1));
            let mut buffer = [0; 10];

            stream.read(&mut buffer).unwrap();
            println!("{:?}", buffer);
        }
    }

    //    #[test]
    #[allow(dead_code)]
    fn client_test() {
        let channel = mpsc::channel();
        let sender: Sender<(Request, Box<FnMut(&Rc<BincodeSerializer>, &Response) + Send>)> = channel.0;
        let receiver = channel.1;
        let req = Request {
            id: 2,
            name: String::from("print"),
            params: vec![BincodeSerializer::new().serialize(&123).unwrap(),
                         BincodeSerializer::new().serialize(&456).unwrap()],
        };
        let callback = |_: &Rc<BincodeSerializer>, r: &Response| { println!("{:?}", r) };
        let handler = thread::spawn(move || {
            let addr = "127.0.0.1:8080".parse().unwrap();
            let client = Client::new(addr, Rc::new(BincodeSerializer::new()));
            client.start(&Arc::new(receiver));
        });
        sender.send((req, Box::new(callback))).unwrap();
        thread::sleep(Duration::from_secs(1));

        let req = Request {
            id: u32::MAX,
            name: String::from("stop"),
            params: vec![BincodeSerializer::new().serialize(&123).unwrap(),
                         BincodeSerializer::new().serialize(&456).unwrap()],
        };
        sender.send((req, Box::new(callback))).unwrap();
        handler.join().unwrap();
    }
}