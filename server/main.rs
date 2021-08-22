#![feature(default_free_fn)]
use {std::default::default, fehler::throws, error::{Error, Result, error, Ok}, std::path::{Path, PathBuf}};

// Direct Serde on the foreign types would avoid boilerplate
trait Convert<T> { fn cvt(self) -> T; }
//impl<S: Convert<T>, T> Convert<Option<T>> for Option<S> { fn cvt(self) -> Option<T> { self.map(|t| t.cvt()) } }
impl Convert<rust::TextRange> for text_size::TextRange { fn cvt(self) -> rust::TextRange { rust::TextRange{start: self.start().into(), end: self.end().into()} } }
impl Convert<rust::TextEdit> for ide::TextEdit { fn cvt(self) -> rust::TextEdit { self.into_iter().map(|ide::Indel{insert,delete}| (insert, delete.cvt())).collect() } }
impl Convert<ide::FilePosition> for rust::FilePosition { fn cvt(self) -> ide::FilePosition { ide::FilePosition{file_id: vfs::FileId(self.file_id), offset: self.offset.try_into().unwrap()} } }
impl Convert<Result<vfs::VfsPath>> for &Path { #[throws] fn cvt(self) -> vfs::VfsPath {
	let current_path = std::env::current_dir()?.join(self);
	let path = if self.is_relative() { current_path.as_path() } else { self };
	use std::convert::TryFrom;
	vfs::AbsPathBuf::try_from(PathBuf::from(path)).map_err(|path| error!(path.display().to_string()))?.into()
}}

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
#[throws] fn path(&self, id: &vfs::FileId) -> PathBuf { Path::new(self.vfs.file_path(*id).as_path().ok()?.as_ref().to_str().ok()?).to_owned() }
}

impl rust::Rust for Analyzer {
	#[throws] fn get_file_id(&self, path: &Path) -> Option<rust::FileId> { self.vfs.file_id(&path.cvt()?).map(|file_id| file_id.0) }
	#[throws] fn highlight(&mut self, file_id: rust::FileId) -> Vec<rust::HighlightedRange> {
		let file_id = vfs::FileId(file_id);
		let mut change = ide::Change::new();
		self.vfs.set_file_contents(self.vfs.file_path(file_id), Some(std::fs::read(self.vfs.file_path(file_id).as_path().ok()?)?));
		change.change_file(file_id, Some(std::sync::Arc::new(std::str::from_utf8(&self.vfs.file_contents(file_id))?.to_owned())));
		self.host.apply_change(change);
		self.host.analysis().highlight(file_id)?
		//trace::timeout_(100, || self.host.analysis().highlight(file_id), format!("Timeout: {}", path.display().to_string()))??
			.into_iter().map(|ide::HlRange{range, highlight, ..}| rust::HighlightedRange{range: range.cvt(), highlight}).collect::<Vec<_>>()
	}
	#[throws] fn definition(&self, position: rust::FilePosition) -> Option<rust::NavigationTarget> {
		self.host.analysis().goto_definition(position.cvt())?
		.map(|v| v.info.first().map(|ide::NavigationTarget{file_id, full_range, ..}| rust::NavigationTarget{path: self.path(file_id).unwrap(), range: full_range.cvt()})).flatten()
	}
	#[throws] fn diagnostics(&self, file_id: rust::FileId) -> Vec<rust::Diagnostic> {
		self.host.analysis().diagnostics(&default(), ide::AssistResolveStrategy::None, vfs::FileId(file_id))?
			.into_iter().map(|ide::Diagnostic{message, range, ..}| rust::Diagnostic{message, range: range.cvt()}).collect::<Vec<_>>()
	}
	#[throws] fn on_char_typed(&self, position: rust::FilePosition, char_typed: char) -> Option<rust::TextEdit> {
		if char_typed=='\n' { self.host.analysis().on_enter(position.cvt())?.map(|edit| edit.cvt()) }
		else { panic!() }
	}
}

#[throws] fn main() {
	#[cfg(feature="trace")] trace::rstack_self();
	ipc::spawn::<Box<dyn rust::Rust>>(Box::new(Analyzer::new()?))?
}
