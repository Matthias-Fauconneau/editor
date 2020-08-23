use {fehler::throws, anyhow::Error};
use std::io::{Read, Write};
use serde::{Serialize,de::DeserializeOwned};
use bincode::deserialize;
pub use bincode::serialize;

pub trait Server {
	const ID: &'static str;
	type Item : Serialize+DeserializeOwned;
	//#[throws] fn new() -> Self where Self: Sized;
	#[throws] fn reply(&mut self, item: Self::Item) -> Vec<u8>;
}

use std::os::unix::net::UnixStream;
#[throws] fn connect<S:Server>() -> UnixStream {
	let path = std::path::Path::new("/run/user").join(unsafe{sc::syscall!(GETUID)}.to_string()).join(S::ID);
	UnixStream::connect(&path).or_else(|_| {
		if path.exists() { std::fs::remove_file(&path)?; }
		let mut inotify = inotify::Inotify::init()?;
    inotify.add_watch(path.parent().unwrap(), inotify::WatchMask::CREATE)?;
    std::process::Command::new("server").spawn()?;
		/*let server = S::new().unwrap();
		if let Ok(fork::Fork::Child) = fork::daemon(true, true) { // slow link
			std::panic::set_hook(Box::new(|info| { // Block unwind
			 let msg = match info.payload().downcast_ref::<&'static str>() {
						Some(s) => *s,
						None => match info.payload().downcast_ref::<String>() {
								Some(s) => &s[..],
								None => "Box<Any>",
						},
				};
				eprintln!("{}: {}", info.location().unwrap(), msg);
				std::process::abort()
			}));
			panic!("{:?}", (||->anyhow::Result<()>{ for client in std::os::unix::net::UnixListener::bind(&path)?.incoming() {
				let mut client = client?;
				let reply = server.reply(bincode::deserialize_from(std::io::Read::by_ref(&mut client))?)?;
				client.write_all(&reply)?;
			} Ok(()) })());
		}*/
		let mut buffer = [0u8; 256];
    loop {
        let mut events = inotify.read_events_blocking(&mut buffer)?;
        if events.find(|e| e.name == path.file_name()).is_some() { break; }
		}
		UnixStream::connect(path)
	})?
}

pub trait Request {
	type Server: Server;
	type Reply: Serialize+DeserializeOwned;
	#[throws] fn reply(self, server: &mut Self::Server) -> Self::Reply;
}

#[throws] pub fn request<'t, R: Request>(request: <R::Server as Server>::Item) -> R::Reply {
	let mut server = connect::<R::Server>()?;
	server.write(&serialize(&request)?)?;
	let mut reply = Vec::new();
	let size = server.read_to_end(&mut reply)?;
	deserialize(&reply[0..size])?
}

#[throws] pub fn spawn<S:Server>(mut server: S) {
	let path = std::path::Path::new("/run/user").join(unsafe{sc::syscall!(GETUID)}.to_string()).join(S::ID);
	for client in std::os::unix::net::UnixListener::bind(&path)?.incoming() {
		let mut client = client?;
		let reply = server.reply(bincode::deserialize_from(std::io::Read::by_ref(&mut client))?)?;
		client.write_all(&reply)?;
	}
}
