#[macro_use]
extern crate lazy_static;

use clap::App;
use regex::Regex;
use std::fs::File;
use std::io::{Read, Write};
use std::io::{self, BufRead, BufReader, BufWriter };
use std::process::{Child, Command, Stdio};
use std::collections::HashMap;
use std::path::Path;

lazy_static! {
    static ref REGEX_SOURCE_FILE_NAME: Regex =
        { Regex::new(r"(?P<source_file_name>(\./)*[^\./]+\..+)\.md$").unwrap() };
}

fn main() {
    let args = App::new("mdlp-rs")
        .version("0.1.0")
        .author("Taiga Nakayama <dora@dora-gt.jp>")
        .about("Extract source code from markdown files.")
        .args_from_usage(
            "
            -i, --input=[INPUT]   'files to process'
            -o, --output=[OUTPUT] 'output directory'
            -v                    'be verbose mode, outputs log'
        ",
        )
        .get_matches();

    // decide input files
    let input_files = args.value_of("input");
    let input_files = match input_files {
        Some(arg) => arg,
        None => ".",
    };
    let input_files = get_files_of(input_files);

    // decide output directory
    let output_directory = args.value_of("output");
    let output_directory = match output_directory {
        Some(arg) => arg,
        None => ".",
    };

    // decide verbose mode or not
    let verbose_mode = args.is_present("v");

    // extract source codes
    let mut mdlprs = Mdlprs::new(input_files, output_directory.to_string(), verbose_mode);
    mdlprs.output_sources();
}

fn get_files_of(directory: &str) -> Vec<String> {
    let mut find_process = Command::new("find")
        .arg(directory)
        .arg("-name")
        .arg("*.md")
        .arg("-type")
        .arg("f")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");
    let output = find_process
        .wait_with_output()
        .expect("Could not launch find");
    let stdout = String::from_utf8(output.stdout).expect("IO error");
    let mut files = Vec::<String>::new();
    for line in stdout.trim().lines() {
        files.push(line.to_string());
    }
    return files;
}

struct Mdlprs {
    files: Vec<String>,
    output_directory: String,
    verbose_mode: bool,
}

#[derive(PartialEq)]
enum MdlprsState {
    None,
    Markdown,
    Program,
}

impl Mdlprs {
    pub fn new(files: Vec<String>, output_directory: String, verbose_mode: bool) -> Mdlprs {
        Mdlprs {
            files,
            output_directory,
            verbose_mode,
        }
    }

    pub fn output_sources(&mut self) {
        for file in &self.files {
            self.log(&format!("Processing: {}", file));
            let source_file_name = self.get_source_file_name_of(file);
            let file_to_read = File::open(file).unwrap();
            let reader = BufReader::new(file_to_read);
            let mut state = MdlprsState::None;
            let mut output_file_name;
            let mut programs_map: HashMap<String, Vec<String>> = HashMap::new();

            for line in reader.lines() {
                if let Err(_) = line {
                    break;
                }

                let line = line.unwrap();
                if line.starts_with("```") {
                    if state == MdlprsState::Program {
                        panic!("Program section should not start twice!");
                    }
                    output_file_name = self.get_output_file_name_of(&line);
                    if source_file_name.as_ref().is_none() && output_file_name.as_ref().is_none() {
                        self.log("Can not process todo");
                        return;
                    }
                    if output_file_name.as_ref().is_none() {
                        output_file_name = Some(source_file_name.as_ref().unwrap().clone());
                    }
                    if !programs_map.contains_key(output_file_name.as_ref().unwrap()) {
                        programs_map.insert(output_file_name.as_ref().unwrap().clone(), Vec::new());
                    }
                    state = MdlprsState::Program;
                } else if state == MdlprsState::Program {
                    let key = output_file_name.as_ref().unwrap().clone();
                    programs_map.get_mut(&key).unwrap().push(line);
                }
            }

            for (key, value) in programs_map {
                let file_path = Path::new(&self.output_directory).join(key);
                let file_to_write = File::create(file_path.as_path().to_str().unwrap());
                let mut writer = BufWriter::new(file_to_write.unwrap());
                for line in value {
                    writer.write(line.as_bytes());
                }
            }
        }
    }

    /// If the file contains source file name like: "foo.rs.md", returns Some of "foo.rs".
    /// If not, returns None.
    fn get_source_file_name_of(&self, file: &str) -> Option<String> {
        let captures = REGEX_SOURCE_FILE_NAME.captures(file);
        match captures {
            Some(value) => Some(value.name("source_file_name").unwrap().as_str().to_string()),
            None => None,
        }
    }

    /// Returns the file name specified in the first line of a code block like \`\`\`rust main.rs
    /// If the line doesn't contain file name, it returns None
    fn get_output_file_name_of(&self, md_line: &str) -> Option<String> {
        if md_line.contains(" ") {
            Some(md_line.split(" ").collect::<Vec<&str>>()[1].to_string())
        } else {
            None
        }
    }

    #[inline]
    fn log(&self, log: &str) {
        if !self.verbose_mode {
            return;
        }
        println!("{}", log);
    }
}
