pub mod comparators;
pub mod codec;
#[cfg(feature = "net")]
use rust_sfp;
#[cfg(feature = "net")]
pub use rust_sfp::SocketAddr;
#[cfg(feature = "net")]
pub use codec::Package;
#[cfg(feature = "net")]
use std::{io, fmt};
#[cfg(feature = "net")]
use std::fmt::{Formatter, Debug};
#[cfg(feature = "net")]
use std::time::Duration;
#[cfg(feature = "net")]
use std::net::Shutdown;
#[cfg(feature = "net")]
use rust_sfp::{FrameWriter, FrameReader, ConnectionController};

#[cfg(feature = "net")]
#[derive(Debug)]
pub enum SendError{
    NetError(rust_sfp::WriteErr),
    CodecErr(codec::EncodeError),
}

#[cfg(feature = "net")]
#[derive(Debug)]
pub enum ReceiveError{
    NetError(io::Error),
    CodecErr(codec::DecodeError),
}


#[cfg(feature = "net")]
pub trait Sender{
    fn send(&mut self, package: Package) -> Result<(), SendError>;
    fn send_no_flush(&mut self, package: Package) -> Result<(), SendError>;
    fn flush(&mut self) -> io::Result<()>;
}

#[cfg(feature = "net")]
pub trait Receiver: Iterator{
    fn recv(&mut self) -> Result<Package, ReceiveError>;
}

#[cfg(feature = "net")]
pub trait Controller{
    fn local_addr(&self) -> io::Result<SocketAddr>;
    fn peer_addr(&self) -> io::Result<SocketAddr>;
    fn set_read_timeout(&self, t: Option<Duration>) -> io::Result<()>;
    fn set_write_timeout(&self, t: Option<Duration>) -> io::Result<()>;
    fn shutdown(&self, t: Shutdown) -> io::Result<()>;
}

#[cfg(feature = "net")]
impl fmt::Display for SendError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self{
            SendError::NetError(err) => {
                std::fmt::Display::fmt(&err, f)
            }
            SendError::CodecErr(err) => {
                std::fmt::Display::fmt(&err, f)
            }
        }
    }
}

#[cfg(feature = "net")]
impl fmt::Display for ReceiveError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self{
            ReceiveError::NetError(err) => {
                std::fmt::Display::fmt(&err, f)
            }
            ReceiveError::CodecErr(err) => {
                std::fmt::Display::fmt(&err, f)
            }
        }
    }
}

#[cfg(feature = "net")]
#[derive(Debug)]
pub struct Connection{
    stream: rust_sfp::Connection
}

#[cfg(feature = "net")]
impl Connection{
    pub fn connect(s: &SocketAddr) -> io::Result<Self> {
        match rust_sfp::Connection::connect(s){
            Ok(stream) => { Ok(Self{stream}) }
            Err(err) => { Err(err) }
        }
    }
}

#[cfg(feature = "net")]
impl Sender for Connection{
    fn send(&mut self, package: Package) -> Result<(), SendError> {
        if let Err(err) = self.send_no_flush(package){ return Err(err) }
        if let Err(err) = self.flush(){
            return Err(SendError::NetError(rust_sfp::WriteErr::I0(err)))
        }
        Ok(())
    }

    fn send_no_flush(&mut self, package: Package) -> Result<(), SendError> {
        let mut bytes = match package.encode(){
            Ok(bytes) => { bytes }
            Err(err) => {
                return Err(SendError::CodecErr(err))
            }
        };
        match self.stream.write_frame(&mut bytes){
            Ok(_) => { Ok(()) }
            Err(err) => {
                Err(SendError::NetError(err))
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

#[cfg(feature = "net")]
impl Receiver for Connection{
    fn recv(&mut self) -> Result<Package, ReceiveError> {
        let bytes = match self.stream.read_frame(){
            Ok(bytes) => { bytes }
            Err(err) => {
                return Err(ReceiveError::NetError(err))
            }
        };
        match Package::decode(bytes){
            Ok(package) => { Ok(package) }
            Err(err) => { Err(ReceiveError::CodecErr(err)) }
        }
    }
}

#[cfg(feature = "net")]
impl Iterator for Connection{
    type Item = Package;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.recv(){
                Ok(package) => { return Some(package) }
                Err(err) => {
                    match err{
                        ReceiveError::NetError(_) => { return None }
                        ReceiveError::CodecErr(_) => { continue }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "net")]
impl Controller for Connection{
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.stream.local_addr()
    }

    fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.stream.peer_addr()
    }

    fn set_read_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.stream.set_read_timeout(t)
    }

    fn set_write_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.stream.set_write_timeout(t)
    }

    fn shutdown(&self, t: Shutdown) -> io::Result<()> {
        self.stream.shutdown(t)
    }
}

#[cfg(feature = "net")]
impl From<rust_sfp::Connection> for Connection{
    fn from(stream: rust_sfp::Connection) -> Self {
        Self{stream}
    }
}

#[cfg(feature = "net")]
pub struct ConnectionSender{
    conn: Connection
}

#[cfg(feature = "net")]
impl From<Connection> for ConnectionSender{
    fn from(conn: Connection) -> Self {
        Self{conn}
    }
}

#[cfg(feature = "net")]
impl Sender for ConnectionSender{
    fn send(&mut self, package: Package) -> Result<(), SendError> {
        self.conn.send(package)
    }

    fn send_no_flush(&mut self, package: Package) -> Result<(), SendError> {
        self.conn.send_no_flush(package)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.conn.flush()
    }
}

#[cfg(feature = "net")]
impl Controller for ConnectionSender{
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.conn.local_addr()
    }

    fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.conn.peer_addr()
    }

    fn set_read_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.conn.set_read_timeout(t)
    }

    fn set_write_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.conn.set_write_timeout(t)
    }

    fn shutdown(&self, t: Shutdown) -> io::Result<()> {
        self.conn.shutdown(t)
    }
}

#[cfg(feature = "net")]
pub struct ConnectionReceiver{
    conn: Connection
}

#[cfg(feature = "net")]
impl From<Connection> for ConnectionReceiver{
    fn from(conn: Connection) -> Self {
        Self{conn}
    }
}

#[cfg(feature = "net")]
impl Receiver for ConnectionReceiver{
    fn recv(&mut self) -> Result<Package, ReceiveError> {
        self.conn.recv()
    }
}

#[cfg(feature = "net")]
impl Iterator for ConnectionReceiver{
    type Item = Package;

    fn next(&mut self) -> Option<Self::Item> {
        self.conn.next()
    }
}


#[cfg(feature = "net")]
pub struct Server{
    server: rust_sfp::Server
}

#[cfg(feature = "net")]
impl Server{
    pub fn bind(s: &SocketAddr) -> io::Result<Self> {
        Ok(Self{server: rust_sfp::Server::bind(s)?})
    }
    pub fn bind_reuse(s: &SocketAddr, _mode: Option<u32>) -> io::Result<Self> {
        Ok(Self{server: rust_sfp::Server::bind_reuse(s, _mode)?})
    }
    pub fn accept(&self) -> io::Result<(Connection,SocketAddr)> {
        let (stream, addr) = self.server.accept()?;
        Ok((Connection::from(stream), addr))
    }
}
