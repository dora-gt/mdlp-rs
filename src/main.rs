use clap::App;
use std::io::Read;
use std::process::{Child, Command, Stdio};

fn main() {
    let args = App::new("mdlp-rs")
        .version("0.1.0")
        .author("Taiga Nakayama <dora@dora-gt.jp>")
        .about("Extract source code from markdown files.")
        .args_from_usage(
            "
            -i, --input=[INPUT] 'files to process'
        ",
        )
        .get_matches();

    let input = args.value_of("input");
    let input = match input {
        Some(arg) => arg,
        None => ".",
    };
    let files = get_files(input);
    for file in &files {
        println!("target file: {}", file);
    }
}

fn get_files(directory: &str) -> Vec<String> {
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
    let output = find_process.wait_with_output().expect("Could not launch find");
    let stdout = String::from_utf8(output.stdout).expect("IO error");
    let mut files = Vec::<String>::new();
    for line in stdout.trim().lines() {
        files.push(line.to_string());
    }
    return files;
}

struct Mdlprs {
    outputdir: String,
}
