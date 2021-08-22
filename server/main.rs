#![feature(default_free_fn)]
use {std::default::default, fehler::throws, anyhow::Error, std::path::{Path, PathBuf}};

// Direct Serde on the foreign types would avoid boilerplate
trait From<T> { fn from(T) -> Self; }
trait Into<T:From<Self>> { fn into(self) -> T { T::from(self) } }
impl From<&text_size::TextRange> for rust::TextRange { fn from(range: &text_size::TextRange) -> Self { Self{start: range.start().into(), end: range.end().into()} } }
impl From<TextEdit> for rust::TextEdit { fn from(text_edit: &text_edit::TextEdit) -> Self { text_edit.iter().map(|text_edit::Indel{insert,delete} (insert, delete.into())).collect() } }
impl From<rust::FilePosition> for ide::FilePosition { fn from(rust::FilePosition{file_id, offset}: rust::FilePosition) -> Self { Self{file_id: vfs::FileId(file_id), offset: offset.into()} } }

struct Analyzer {
		host: ide::AnalysisHost,
		vfs: vfs::Vfs
}

impl Analyzer {
#[throws] fn new() -> Self {
	use rust_analyzer::cli::load_cargo::*;
	let (host, vfs, _proc_macro) = load_workspace_at(&std::env::current_dir()?, &default(),
		&LoadCargoConfig{load_out_dirs_from_check: false, with_proc_macro: true, prefill_caches: false}, &|_| {})?;
	Analyzer{host, vfs}
}
#[throws] fn file_id(&self, path: &Path) -> vfs::FileId {
	let current_path = std::env::current_dir().unwrap().join(path);
	let path = if path.is_relative() { current_path.as_path() } else { path };
	use std::convert::TryFrom;
	self.vfs.file_id(paths::AbsPath::try_from(path).unwrap().into())
}
#[throws] fn path(&self, id: &vfs::FileId) -> PathBuf { Path::new(self.vfs.file_path(*id).as_path().unwrap().as_ref().to_str().unwrap()).to_owned() }
}

impl rust::Rust for Analyzer {
	#[throws] fn file_id(&mut self, path: Path) -> rust::FileId { self.file_id(path)?.0 }
	#[throws] fn highlight(&mut self, file_id: rust::FileId) -> Vec<rust::HighlightedRange> {
		let file_id = vfs::FileId(file_id);
		let mut change = ide::Change::new();
		change.change_file(file_id, Some(std::sync::Arc::new(std::str::from_utf8(&std::fs::read(path)?)?.to_owned())));
		self.host.apply_change(change);
		self.host.analysis().highlight(file_id)?
		//trace::timeout_(100, || self.host.analysis().highlight(file_id), format!("Timeout: {}", path.display().to_string()))??
			.into_iter().map(|ide::HlRange{range, highlight, ..}| rust::HighlightedRange{range: from(&range), highlight}).collect::<Vec<_>>()
	}
	#[throws] fn definition(&self, position: rust::FilePosition) -> Option<rust::NavigationTarget> {
		self.host.analysis().goto_definition(position.into())?
		.map(|v| v.info.first().map(|ide::NavigationTarget{file_id, full_range, ..}| rust::NavigationTarget{path: self.path(file_id).unwrap(), range: from(full_range)})).flatten()
	}
	#[throws] fn diagnostics(&self, file_id: rust::FileId) -> Vec<rust::Diagnostic> {
		self.host.analysis().diagnostics(&default(), ide::AssistResolveStrategy::None, vfs::FileId(file_id))?
			.into_iter().map(|ide::Diagnostic{message, range, ..}| rust::Diagnostic{message, range: from(&range)}).collect::<Vec<_>>()
	}
	#[throws] fn on_typed_char(&self, position: rust::FilePosition, typed_char: char) -> Option<TextEdit> {
		if char=='\n' { self.host.analysis().on_enter(position.into())?.into() }
		else { panic!() }
	}
}

#[throws] fn main() {
	#[cfg(feature="trace")] trace::rstack_self();
	ipc::spawn::<Box<dyn rust::Rust>>(Box::new(Analyzer::new()?))?
}
