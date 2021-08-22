fn main() -> Result<(),std::io::Error> {
	let status = match cargo::build(std::env::args().skip(1))? {
		Ok(status) => status,
		Err(cargo::Diagnostic{message: _, spans, rendered: Some(rendered), ..}, ..) => {
			eprint!("{}", rendered);
			for span in spans {
				if std::path::Path::new(&span.file_name).exists() { println!("{}:{}:{}", span.file_name, span.line_start, span.column_start); }
			}
			-1
		}
		_ => unimplemented!(),
	};
	std::process::exit(status);
}
