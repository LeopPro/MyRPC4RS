use futures::{Future, Stream};
use tokio_io::AsyncRead;
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

pub struct Server {
    socket_addr: SocketAddr,
//    process: &'static FnMut(&BytesMut, &Packages),
}

impl Server {
    pub fn new<F>(socket_addr: SocketAddr) -> Self {
        Self {
            socket_addr,
//            process: process,
        }
    }
    pub fn start<F: 'static>(&mut self, mut process: F)
        where F: FnOnce(&BytesMut, &Packages) {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let listener = TcpListener::bind(&self.socket_addr, &handle).unwrap();
        let process_cell = RefCell::new(process);
        let server = listener.incoming().for_each(move |(socket, _)| {
            let packages = Packages::new(socket);
            let pro = process_cell.borrow_mut();
            let package_handler = packages.into_future()
                .then(move |conn| {
                    match conn {
                        Ok((o_package, packages)) => {
                            match o_package {
                                Some(package) => {
                                    pro(&package, &packages);
                                }
                                None => { /*连接关闭*/ }
                            };
                        }
                        Err(_) => { /*连接错误*/ }
                    };
                    future::ok(())
                });
            handle.spawn(package_handler);
            Ok(())
        });
        core.run(server).unwrap();
    }
}

#[derive(Debug)]
pub struct Packages {
    socket: TcpStream,
    read_buffer: BytesMut,
    write_buffer: BytesMut,
}

impl Stream for Packages {
    type Item = BytesMut;
    type Error = io::Error;

    fn poll(&mut self) -> Result<Async<Option<<Self as Stream>::Item>>, <Self as Stream>::Error> {
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
        return Ok(Async::Ready(Some(package)));
    }
}

impl Packages {
    fn new(socket: TcpStream) -> Self {
        Self {
            socket,
            read_buffer: BytesMut::new(),
            write_buffer: BytesMut::new(),
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

    #[test]
    fn start_server() {
//        let addr = "127.0.0.1:8080".parse().unwrap();
//        let mut server = Server::new(addr, |package, packages| {});
//        server.start();
    }

    #[test]
    fn client() {
        let mut send = vec![11, 21, 31, 12 as u8];

        let mut wtr = vec![];
        wtr.write_u32::<BigEndian>(send.len() as u32).unwrap();
        println!("{:?}", wtr);
        let mut buf = vec![0u8; 1024];
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        loop {
            // 对比Listener，TcpStream就简单很多了
            // 本次模拟的是tcp短链接的过程，可以看作是一个典型的HTTP交互的基础IO模拟
            // 当然，这个通讯里面并没有HTTP协议 XD！
            let mut stream = TcpStream::connect(&addr).unwrap();
            let msg = wtr.as_ref();
            // 避免发送数据太快而刷屏
            thread::sleep_ms(100);
            let rcount = stream.write(&msg).unwrap();
            thread::sleep_ms(1000);
            let msg = send.as_ref();
            let rcount = stream.write(&msg).unwrap();
        }
    }
}