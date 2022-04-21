extern crate log;
extern crate simplelog;
extern crate yui;

use std::error::Error;
use std::fs::File;

use log::LevelFilter;
use simplelog::{Config, WriteLogger};

use yui::console;
use yui::sparks::selection_editor::SelectionEditorSpark;

fn main() -> Result<(), Box<dyn Error>> {
	WriteLogger::init(
		LevelFilter::Info,
		Config::default(),
		File::create("selection.log").expect("log file"),
	).expect("result");
	log::info!("Table");

	let spark = SelectionEditorSpark {
		selected: 0,
		choices: vec![33, 34, 35, 36, 37],
	};
	console::run_spark(spark);
	Ok(())
}
