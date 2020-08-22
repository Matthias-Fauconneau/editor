use {anyhow::Error, fehler::throws, rust_analyzer::cli::load_cargo};
pub use ide::{HighlightedRange, HighlightTag, HighlightModifier};

pub struct TextHighlight { pub text: std::sync::Arc<String>, pub highlight: Vec<HighlightedRange> }
#[throws] pub fn highlight() -> TextHighlight {
	//#[cfg(feature="profile")] env_logger::try_init()?;
	//#[cfg(feature="profile")] env_logger::Builder::new().filter(None, log::LevelFilter::Trace).format_level(false).format_timestamp(None).init();
	//#[cfg(feature="profile")] profile::init();
	let (host, vfs) = load_cargo(&std::env::current_dir()?, false, false)?;
	use std::convert::TryFrom;
	let (file_id, _) = vfs.iter().find(|&(_, path)| path.as_path().unwrap() == <&paths::AbsPath>::try_from(std::env::current_dir().unwrap().join("src/main.rs").as_path()).unwrap()).unwrap();
	let analysis = host.analysis();
	TextHighlight{text: analysis.file_text(file_id)?, highlight: analysis.highlight(file_id)?}
}
