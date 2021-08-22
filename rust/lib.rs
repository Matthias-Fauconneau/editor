use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
#[derive(Clone,Copy,Serialize,Deserialize,Debug)] pub struct TextRange { pub start: u32, pub end: u32 }
pub use types::{SymbolKind, HlTag as HighlightTag, HlMod as HighlightModifier, Highlight};

pub type FileId = u32;
#[derive(Serialize,Deserialize)] pub struct FilePosition { pub file_id: FileId, pub offset: u32 }
pub type TextEdit = Vec<(String, TextRange)>;

#[derive(Serialize,Deserialize)] pub struct HighlightedRange {
  pub range: TextRange,
  pub highlight: Highlight,
  // binding_hash
}

#[derive(Serialize,Deserialize,Debug)] pub struct NavigationTarget {
	pub path: PathBuf,
	pub range: TextRange,
	// full_range, name, kind, container_name, description, docs
}

#[derive(Serialize,Deserialize,Debug)] pub struct Diagnostic {
	pub message: String,
	pub range: TextRange,
	// severity, fix
}

pub trait Rust {
	#[throws] fn get_file_id(&self, path: &Path) -> Option<FileId>;
	#[throws] fn highlight(&mut self, file_id: FileId) -> Vec<HighlightedRange>;
	#[throws] fn diagnostics(&self, file_id: FileId) -> Vec<Diagnostic>;
	#[throws] fn definition(&self, position: FilePosition) -> Option<NavigationTarget>;
	#[throws] fn on_char_typed(&self, position: FilePosition, char_typed: char) -> Option<TextEdit>;
}

#[derive(Serialize,Deserialize)] pub struct GetFileId { path: PathBuf }
impl ipc::Request for GetFileId {
	type Server = Box<dyn Rust>;
	type Reply = Option<FileId>;
	#[throws] fn reply(self, server: &mut Self::Server) -> Self::Reply { server.get_file_id(&self.path)? }
}

#[derive(Serialize,Deserialize)] pub struct HighlightFile { file_id: FileId }
impl ipc::Request for HighlightFile {
	type Server = Box<dyn Rust>;
	type Reply = Vec<HighlightedRange>;
	#[throws] fn reply(self, server: &mut Self::Server) -> Self::Reply { server.highlight(self.file_id)? }
}

#[derive(Serialize,Deserialize)] pub struct Diagnostics { file_id: FileId }
impl ipc::Request for Diagnostics {
	type Server = Box<dyn Rust>;
	type Reply = Vec<Diagnostic>;
	#[throws] fn reply(self, server: &mut Self::Server) -> Self::Reply { server.diagnostics(self.file_id)? }
}

#[derive(Serialize,Deserialize)] pub struct Definition { position: FilePosition }
impl ipc::Request for Definition {
	type Server = Box<dyn Rust>;
	type Reply = Option<NavigationTarget>;
	#[throws] fn reply(self, server: &mut Self::Server) -> Self::Reply { server.definition(self.position)? }
}

#[derive(Serialize,Deserialize)] pub struct OnCharTyped { position: FilePosition, char_typed: char }
impl ipc::Request for OnCharTyped {
	type Server = Box<dyn Rust>;
	type Reply = Option<TextEdit>;
	#[throws] fn reply(self, server: &mut Self::Server) -> Self::Reply { server.on_char_typed(self.position, self.char_typed)? }
}

#[derive(Serialize,Deserialize)] pub enum Item {
	GetFileId(GetFileId),
	HighlightFile(HighlightFile),
	Diagnostics(Diagnostics),
	Definition(Definition),
	OnCharTyped(OnCharTyped)
}

use {fehler::throws, error::Error};

impl ipc::Server for Box<dyn Rust> {
	const ID : &'static str = "rust";
	type Item = Item;
	#[throws] fn reply(&mut self, item: Item) -> Vec<u8> {
		#[throws] fn serialize<T:Serialize>(r : Result<T, Error>) -> Vec<u8> { ipc::serialize(&r.map_err(|e| e.to_string()))? }
		use {ipc::Request, Item::*};
		match item {
			GetFileId(r) => serialize(r.reply(self)),
			HighlightFile(r) => serialize(r.reply(self)),
			Diagnostics(r) => serialize(r.reply(self)),
			Definition(r) => serialize(r.reply(self)),
			OnCharTyped(r) => serialize(r.reply(self)),
		}?
	}
}

#[throws] pub fn file_id(path: &Path) -> Option<FileId> {
	fn abs(path: &Path) -> PathBuf { if path.is_relative() { std::env::current_dir().unwrap().join(path) } else { path.to_owned() } }
	ipc::request::<GetFileId>(Item::GetFileId(GetFileId{path: abs(path)}))?
}

#[throws] pub fn highlight(file_id: FileId) -> Vec<HighlightedRange> {
	ipc::request::<HighlightFile>(Item::HighlightFile(HighlightFile{file_id}))?
}

#[throws] pub fn diagnostics(file_id: FileId) -> Vec<Diagnostic> {
	ipc::request::<Diagnostics>(Item::Diagnostics(Diagnostics{file_id}))?
}

#[throws] pub fn definition(position: FilePosition) -> Option<NavigationTarget> {
	ipc::request::<Definition>(Item::Definition(Definition{position}))?
}

#[throws] pub fn on_char_typed(position: FilePosition, char_typed: char) -> Option<TextEdit> {
	ipc::request::<OnCharTyped>(Item::OnCharTyped(OnCharTyped{position, char_typed}))?
}
