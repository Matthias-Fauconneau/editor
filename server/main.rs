use {fehler::throws, anyhow::Error};

struct Analyzer {
		host: ide::AnalysisHost,
		vfs: vfs::Vfs
}

impl Analyzer {
#[throws] fn new() -> Self { let (host, vfs) = rust_analyzer::cli::load_cargo(&std::env::current_dir()?, false, false)?; Analyzer{host, vfs} }
#[throws] fn file_id(&self, path: &std::path::Path) -> ide::FileId {
	let current_path = std::env::current_dir().unwrap().join(path);
	let path = if path.is_relative() { current_path.as_path() } else { path };
	use std::convert::TryFrom;
	let (file_id, _) = self.vfs.iter().find(|&(_, p)| p.as_path().unwrap() == <&paths::AbsPath>::try_from(path).unwrap()).ok_or(Error::msg(path.to_str().unwrap().to_owned()))?;
	file_id
}
}

impl rust::Rust for Analyzer {
	#[throws] fn highlight(&mut self, path: &std::path::Path) -> Vec<rust::HighlightedRange> {
		let file_id = self.file_id(path)?;
		let mut change = ide::AnalysisChange::new(); // todo: inotify
		change.change_file(file_id, Some(std::sync::Arc::new(std::str::from_utf8(&std::fs::read(path)?)?.to_owned())));
		self.host.apply_change(change);
		self.host.analysis().highlight(file_id)?
			.into_iter().map(|ide::HighlightedRange{range, highlight, ..}| rust::HighlightedRange{range: range.start().into()..range.end().into(), highlight}).collect::<Vec<_>>()
	}
}

#[throws] fn main() { ipc::spawn::<Box<dyn rust::Rust>>(Box::new(Analyzer::new()?))? }
