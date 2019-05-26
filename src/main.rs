#![feature(duration_float)]

#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate failure;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::time::Duration;

use failure::Error;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
	println!("{:?}", args);
	ensure!(args.len() > 1, "usage: {:?} [input.srt] > [output.srt]", args[0]);
	
	let filename = &args[1];
	let mut file = File::open(filename)?;

	let mut buf_reader = BufReader::new(file);
	let mut srt_file = SRTFile::new(buf_reader)?;
	println!("debug {:?}", srt_file);

	for caption in srt_file.captions.iter_mut() {
		let duration = caption.endTime - caption.startTime;
		if duration < Duration::new(0,0) {
			bail!("Caption number {:?} has a negative duration", caption.number);
		}

		let word_count: Vec<_> = caption.text.split(|c| c == '\n' || c == ' ').collect();

		let words_per_second = word_count.len() as f64 / duration.as_secs_f64();
		println!("Duration {:?}, word_count {:?}, wordsPerSecond {:?}", duration, word_count, words_per_second);
	}
	Ok(())
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
enum SRTFileError {
	#[fail(display = "Timestamp Parsing Error")]
	TimestampParsingError,
	#[fail(display = "Caption Builder Error")]
	CaptionBuilderError
}

#[derive(Debug)]
struct SRTFile {
	captions: Vec<Caption>
}

impl SRTFile {
	fn new<T: Read>(buf: BufReader<T>) -> Result<Self, Error> {
		let mut lines = buf.lines();
		let mut ret = SRTFile{
			captions: vec![],
		};
		let mut caption = CaptionBuilder::default();
		let mut line_no = 0;
		while let Some(Ok(line)) = lines.next() {
			println!("line: {:?}", line);
			if line == "" {
				ret.captions.push(caption.build().map_err(|_| SRTFileError::CaptionBuilderError)?);
				println!("new caption: {:?}", ret.captions[ret.captions.len()-1]);
				caption = CaptionBuilder::default();
				line_no = 0;
				continue;
			}
			if line_no == 0 {
				caption.number(line.parse()?);
			}
			if line_no == 1 {
				let mut parts = line.split("-->").map(|part| {
					let mut time = part.trim().split(|c| c == ':' || c == ',');
					let mut n = || time.next().ok_or(SRTFileError::TimestampParsingError);
					let res: Result<_, failure::Error> = Ok(Duration::new(
						(n()?.parse::<u64>()? * 60 * 60) +
						(n()?.parse::<u64>()? * 60) +
						(n()?.parse::<u64>()?),
						(n()?.parse::<u32>()?) * 1000000 // ms to ns
					));
					res
				});
				caption.startTime(parts.next().unwrap()?);
				caption.endTime(parts.next().unwrap()?);
			}
			if line_no > 1 {
				//println!("endTime: {:?}", caption.endTime);
				caption.text(match caption.text.clone() {
					Some(text) => text + "\n" + &line,
					None => line
				});
			}

			line_no+=1;
		}
		Ok(ret)	
	}
	fn encode() -> Result<String, Error> {
		return Ok("".to_string());
	}
}

#[derive(Builder, Debug)]
struct Caption {
	number: u64,
	startTime: Duration,
	endTime: Duration,
	text: String
}
