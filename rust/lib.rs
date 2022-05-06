use std::path::{Path, PathBuf};
fn abs(path: &Path) -> PathBuf { if path.is_relative() { std::env::current_dir().unwrap().join(path) } else { path.to_owned() } }

pub use {text_size::{TextSize, TextRange}, text_edit::TextEdit, vfs::FileId, base_db::FilePosition, types::{SymbolKind, HlTag, HlMod, Highlight}};

use serde::{Serialize, Deserialize};

#[derive(Serialize,Deserialize)] pub struct HlRange {
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

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Rust {
	fn get_file_id(&self, path: &Path) -> Result<Option<FileId>>;
	fn highlight(&mut self, file_id: FileId) -> Result<Box<[HlRange]>>;
	fn diagnostics(&self, file_id: FileId) -> Result<Box<[Diagnostic]>>;
	fn definition(&self, position: FilePosition) -> Result<Option<NavigationTarget>>;
	fn on_char_typed(&self, position: FilePosition, char_typed: char) -> Result<Option<TextEdit>>;
}

use ipc::Request;

#[derive(Serialize,Deserialize)] pub struct GetFileId { path: PathBuf }
impl Request for GetFileId {
	type Server = Box<dyn Rust>;
	type Reply = Option<FileId>;
	fn reply(self, server: &mut Self::Server) -> Result<Self::Reply> { server.get_file_id(&self.path) }
}

#[derive(Serialize,Deserialize)] pub struct HighlightFile { file_id: FileId }
impl Request for HighlightFile {
	type Server = Box<dyn Rust>;
	type Reply = Box<[HlRange]>;
	fn reply(self, server: &mut Self::Server) -> Result<Self::Reply> { server.highlight(self.file_id) }
}

#[derive(Serialize,Deserialize)] pub struct Diagnostics { file_id: FileId }
impl Request for Diagnostics {
	type Server = Box<dyn Rust>;
	type Reply = Box<[Diagnostic]>;
	fn reply(self, server: &mut Self::Server) -> Result<Self::Reply> { server.diagnostics(self.file_id) }
}

#[derive(Serialize,Deserialize)] pub struct Definition { position: FilePosition }
impl Request for Definition {
	type Server = Box<dyn Rust>;
	type Reply = Option<NavigationTarget>;
	fn reply(self, server: &mut Self::Server) -> Result<Self::Reply> { server.definition(self.position) }
}

#[derive(Serialize,Deserialize)] pub struct OnCharTyped { position: FilePosition, char_typed: char }
impl Request for OnCharTyped {
	type Server = Box<dyn Rust>;
	type Reply = Option<TextEdit>;
	fn reply(self, server: &mut Self::Server) -> Result<Self::Reply> { server.on_char_typed(self.position, self.char_typed) }
}


#[derive(Serialize,Deserialize)] pub enum Item {
	GetFileId(GetFileId),
	HighlightFile(HighlightFile),
	Diagnostics(Diagnostics),
	Definition(Definition),
	OnCharTyped(OnCharTyped)
}

impl ipc::Server for Box<dyn Rust> {
	const ID : &'static str = "rust";
	type Item = Item;
	fn reply(&mut self, item: Item) -> Box<[u8]> {
		fn serialize<T:Serialize>(r : Result<T>) -> Box<[u8]> { ipc::serialize(&r.map_err(|e| e.to_string())).unwrap().into_boxed_slice() }
		use Item::*;
		match item {
			GetFileId(r) => serialize(r.reply(self)),
			HighlightFile(r) => serialize(r.reply(self)),
			Diagnostics(r) => serialize(r.reply(self)),
			Definition(r) => serialize(r.reply(self)),
			OnCharTyped(r) => serialize(r.reply(self)),
		}
	}
}

use ipc::request;
pub fn file_id(path: &Path) -> Result<Option<FileId>> { request::<GetFileId>(Item::GetFileId(GetFileId{path: abs(path)})) }
pub fn highlight(file_id: FileId) -> Result<Box<[HlRange]>> { request::<HighlightFile>(Item::HighlightFile(HighlightFile{file_id})) }
pub fn diagnostics(file_id: FileId) -> Result<Box<[Diagnostic]>> { request::<Diagnostics>(Item::Diagnostics(Diagnostics{file_id})) }
pub fn definition(position: FilePosition) -> Result<Option<NavigationTarget>> { request::<Definition>(Item::Definition(Definition{position})) }
pub fn on_char_typed(position: FilePosition, char_typed: char) -> Result<Option<TextEdit>> { request::<OnCharTyped>(Item::OnCharTyped(OnCharTyped{position, char_typed})) }
