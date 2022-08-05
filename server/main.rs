use fehler::throws; type Error = Box<dyn std::error::Error>; use std::path::{Path, PathBuf};

#[throws] fn vfs(path: &Path) -> vfs::VfsPath {
	use std::convert::TryFrom;
	vfs::AbsPathBuf::try_from(if path.is_relative() { std::env::current_dir()?.join(path) } else { path.into() }).map_err(|path| anyhow::Error::msg(path.display().to_string()))?.into()
}

struct Analyzer {
		host: ide::AnalysisHost,
		vfs: vfs::Vfs
}

impl Analyzer {
#[throws] fn new() -> Self {
	use rust_analyzer::cli::load_cargo::*;
	let (host, vfs, _proc_macro) = load_workspace_at(&std::env::current_dir()?, &Default::default(),
		&LoadCargoConfig{load_out_dirs_from_check: false, with_proc_macro: true, prefill_caches: false}, &|_| {})?;
	Analyzer{host, vfs}
}
#[throws] fn path(&self, id: &vfs::FileId) -> PathBuf { Path::new(self.vfs.file_path(*id).as_path().unwrap().as_ref().to_str().unwrap()).to_owned() }
}

impl rust::Rust for Analyzer {
	#[throws] fn get_file_id(&self, path: &Path) -> Option<rust::FileId> { self.vfs.file_id(&vfs(path)?) }
	#[throws] fn highlight(&mut self, file_id: rust::FileId) -> Box<[rust::HlRange]> {
		let mut change = ide::Change::new();
		self.vfs.set_file_contents(self.vfs.file_path(file_id), Some(std::fs::read(self.vfs.file_path(file_id).as_path().unwrap())?));
		change.change_file(file_id, Some(std::sync::Arc::new(std::str::from_utf8(&self.vfs.file_contents(file_id))?.to_owned())));
		self.host.apply_change(change);
		self.host.analysis().highlight(file_id)?
			.into_iter().map(|ide::HlRange{range, highlight, ..}| rust::HlRange{range, highlight}).collect()
	}
	#[throws] fn definition(&self, position: rust::FilePosition) -> Option<rust::NavigationTarget> {
		self.host.analysis().goto_definition(position)?
		.map(|v| v.info.first().map(|ide::NavigationTarget{file_id, full_range, ..}| rust::NavigationTarget{path: self.path(file_id).unwrap(), range: *full_range})).flatten()
	}
	#[throws] fn diagnostics(&self, file_id: rust::FileId) -> Box<[rust::Diagnostic]> {
		self.host.analysis().diagnostics(&ide::DiagnosticsConfig{proc_macros_enabled:true, proc_attr_macros_enabled:true, ..Default::default()}, ide::AssistResolveStrategy::None, file_id)?
			.into_iter().map(|ide::Diagnostic{message, range, ..}| rust::Diagnostic{message, range}).collect()
	}
	#[throws] fn on_char_typed(&self, position: rust::FilePosition, char_typed: char) -> Option<rust::TextEdit> {
		if char_typed=='\n' { self.host.analysis().on_enter(position)? }
		else { panic!() }
	}
}

#[throws] fn main() {
	#[cfg(feature="trace")] trace::rstack_self();
	ipc::spawn::<Box<dyn rust::Rust>>(Box::new(Analyzer::new()?))?
}
