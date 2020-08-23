use serde::{Serialize, Deserialize};
#[cfg(not(feature="ide"))] mod ide; // syntax_highlighting/tags (3s link)
pub use ide::{HighlightTag, HighlightModifier, Highlight};

#[derive(Serialize,Deserialize)] pub struct HighlightedRange {
    pub range: std::ops::Range<u32>,
    pub highlight: Highlight,
}

pub trait Rust {
	#[throws] fn highlight(&mut self, path: &std::path::Path) -> Vec<HighlightedRange>;
}

#[derive(Serialize,Deserialize)] pub struct HighlightFile { path: std::path::PathBuf }
impl ipc::Request for HighlightFile {
	type Server = Box<dyn Rust>;
	type Reply = Vec<HighlightedRange>;
	#[throws] fn reply(self, server: &mut Self::Server) -> Self::Reply { server.highlight(&self.path)? }
}

#[derive(Serialize,Deserialize)] pub enum Item { HighlightFile(HighlightFile) }

use {fehler::throws, anyhow::Error};

impl ipc::Server for Box<dyn Rust> {
	const ID : &'static str = "rust";
	type Item = Item;
	//#[throws] fn new() -> Self { let (host, vfs) = rust_analyzer::cli::load_cargo(&std::env::current_dir()?, false, false)?; box Analyzer{host, vfs} }
	#[throws] fn reply(&mut self, item: Item) -> Vec<u8> { use {ipc::Request, Item::*}; match item {
			HighlightFile(r) => ipc::serialize(&r.reply(self)?)?,
	}}
}

#[throws] pub fn highlight(path: &std::path::Path) -> Vec<HighlightedRange> {
	let path = if path.is_relative() { std::env::current_dir().unwrap().join(path) } else { path.to_owned() };
	ipc::request::<HighlightFile>(Item::HighlightFile(HighlightFile{path}))?
}
