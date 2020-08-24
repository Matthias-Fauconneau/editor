use std::{ops::Range, path::{Path, PathBuf}};
use serde::{Serialize, Deserialize};
#[cfg(not(feature="ide"))] mod ide; // syntax_highlighting/tags
pub use ide::{HighlightTag, HighlightModifier, Highlight};

#[derive(Serialize,Deserialize)] pub struct HighlightedRange {
  pub range: Range<u32>,
  pub highlight: Highlight,
}

#[derive(Serialize,Deserialize,Debug)] pub struct NavigationTarget {
	pub path: PathBuf,
	pub range: Range<u32>,
}

pub trait Rust {
	#[throws] fn highlight(&mut self, path: &Path) -> Vec<HighlightedRange>;
	#[throws] fn definition(&self, path: &Path, offset: u32) -> Option<NavigationTarget>;
}

#[derive(Serialize,Deserialize)] pub struct HighlightFile { path: PathBuf }
impl ipc::Request for HighlightFile {
	type Server = Box<dyn Rust>;
	type Reply = Vec<HighlightedRange>;
	#[throws] fn reply(self, server: &mut Self::Server) -> Self::Reply { server.highlight(&self.path)? }
}

#[derive(Serialize,Deserialize)] pub struct Definition { path: PathBuf, offset: u32 }
impl ipc::Request for Definition {
	type Server = Box<dyn Rust>;
	type Reply = Option<NavigationTarget>;
	#[throws] fn reply(self, server: &mut Self::Server) -> Self::Reply { server.definition(&self.path, self.offset)? }
}

#[derive(Serialize,Deserialize)] pub enum Item {
	HighlightFile(HighlightFile),
	Definition(Definition),
}

use {fehler::throws, anyhow::Error};

impl ipc::Server for Box<dyn Rust> {
	const ID : &'static str = "rust";
	type Item = Item;
	#[throws] fn reply(&mut self, item: Item) -> Vec<u8> { use {ipc::Request, Item::*}; match item {
			HighlightFile(r) => ipc::serialize(&r.reply(self)?)?,
			Definition(r) => ipc::serialize	(&r.reply(self)?)?,
	}}
}

fn abs(path: &Path) -> PathBuf { if path.is_relative() { std::env::current_dir().unwrap().join(path) } else { path.to_owned() } }

#[throws] pub fn highlight(path: &Path) -> Vec<HighlightedRange> {
	ipc::request::<HighlightFile>(Item::HighlightFile(HighlightFile{path: abs(path)}))?
}

#[throws] pub fn definition(path: &Path, offset: usize) -> Option<NavigationTarget> {
	ipc::request::<Definition>(Item::Definition(Definition{path: abs(path), offset: offset as u32}))?
}
