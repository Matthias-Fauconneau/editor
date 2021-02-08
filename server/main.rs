#![feature(default_free_fn)]
use std::{default::default, path::{Path, PathBuf}};
use {fehler::throws, anyhow::Error};

fn from(range: &text_size::TextRange) -> rust::TextRange { rust::TextRange{start: range.start().into(), end: range.end().into()} } // serde

struct Analyzer {
		host: ide::AnalysisHost,
		vfs: vfs::Vfs
}

impl Analyzer {
#[throws] fn new() -> Self { let (host, vfs) = rust_analyzer::cli::load_cargo(&std::env::current_dir()?, false, false)?; Analyzer{host, vfs} }
#[throws] fn file_id(&self, path: &Path) -> vfs::FileId {
	let current_path = std::env::current_dir().unwrap().join(path);
	let path = if path.is_relative() { current_path.as_path() } else { path };
	use std::convert::TryFrom;
	let (file_id, _) = self.vfs.iter().find(|&(_, p)| p.as_path().unwrap() == <&paths::AbsPath>::try_from(path).unwrap()).ok_or(Error::msg(path.to_str().unwrap().to_owned()))?;
	file_id
}
#[throws] fn path(&self, id: &vfs::FileId) -> PathBuf { Path::new(self.vfs.file_path(*id).as_path().unwrap().to_str().unwrap()).to_owned() }
}

impl rust::Rust for Analyzer {
	#[throws] fn highlight(&mut self, path: &Path) -> Vec<rust::HighlightedRange> {
		let file_id = self.file_id(path)?;
		let mut change = ide::Change::new();
		change.change_file(file_id, Some(std::sync::Arc::new(std::str::from_utf8(&std::fs::read(path)?)?.to_owned())));
		self.host.apply_change(change);
		self.host.analysis().highlight(file_id)?
		//trace::timeout_(100, || self.host.analysis().highlight(file_id), format!("Timeout: {}", path.display().to_string()))??
			.into_iter().map(|ide::HlRange{range, highlight, ..}| rust::HighlightedRange{range: from(&range), highlight}).collect::<Vec<_>>()
	}
	#[throws] fn definition(&self, path: &Path, offset: u32) -> Option<rust::NavigationTarget> {
		self.host.analysis().goto_definition(ide::FilePosition{file_id: self.file_id(path)?, offset: offset.into()})?
		.map(|v| v.info.first().map(|ide::NavigationTarget{file_id, full_range, ..}| rust::NavigationTarget{path: self.path(file_id).unwrap(), range: from(full_range)})).flatten()
	}
	#[throws] fn diagnostics(&self, path: &Path) -> Vec<rust::Diagnostic> {
		self.host.analysis().diagnostics(&default(), self.file_id(path)?)?
			.into_iter().map(|ide::Diagnostic{message, range, ..}| rust::Diagnostic{message, range: from(&range)}).collect::<Vec<_>>()
	}
}

#[throws] fn main() {
	#[cfg(feature="trace")] trace::rstack_self();
	ipc::spawn::<Box<dyn rust::Rust>>(Box::new(Analyzer::new()?))?
}
