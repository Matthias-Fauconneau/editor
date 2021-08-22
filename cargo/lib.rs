#![feature(associated_type_bounds)]
pub use cargo_metadata::diagnostic::{Diagnostic, DiagnosticSpan as Span};

use std::process::{Command, Stdio};
#[fehler::throws(std::io::Error)] pub fn build(args: impl IntoIterator<Item:AsRef<std::ffi::OsStr>>) -> Result<i32, Diagnostic> {
	let mut child = Command::new("cargo").arg("build").args(args).arg("--message-format=json").stdout(Stdio::piped()).spawn()?;
	use cargo_metadata::{Message, CompilerMessage};
	for msg in Message::parse_stream(std::io::BufReader::new(child.stdout.take().unwrap())) { match msg? {
		Message::CompilerMessage(CompilerMessage{message, ..}) => {
			let _ = child.kill(); // Kill on first warning/error to save energy/heat
			if message.message == "aborting due to previous error" { continue; }
			return Err(message);
		},
		_=>{},
	}}
	Ok(child.wait()?.code().unwrap_or(-1))
}
