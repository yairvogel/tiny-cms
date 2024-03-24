use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use anyhow::Context;
use anyhow::Error;
use clap::{Parser, Subcommand};

const CONTENT_DIR: &str = "content";
const METADATA_FILE: &str = ".cms";


#[derive(Parser, Debug)]
struct Args
{
    #[command(subcommand)]
    cmd: Command
}

#[derive(Subcommand, Debug)]
enum Command {
    Init {
        #[arg(long, default_value = CONTENT_DIR)]
        content_dir: String
    },
    New {
        #[arg(short, long)]
        name: String
    }
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    match args.cmd {
        Command::Init { content_dir } => init(content_dir),
        Command::New { name } => new(name)
    }
}

fn init(content_dir: String) -> Result<(), Error> {
    if Path::new(METADATA_FILE).exists() {
        return Err(Error::msg("cms is already initialized"))
    }

    let mut file = File::create(METADATA_FILE).context("creating .cms file")?;
    file.write_all(content_dir.as_bytes()).context("writing .cms file")?;

    fs::create_dir(content_dir).context("creating content directory")?;
    Ok(())
}

fn new(name: String) -> Result<(), Error> {
    let content_dir = get_content_dir().context("get content directory")?;
    let new_file_path = Path::new(&content_dir).join(&name);
    if new_file_path.exists() {
        return Err(Error::msg(format!("post name '{}' already exists", &name)))
    }

    let mut file = File::create(new_file_path).context("creating new post file")?;
    let content = format!("------------------\nname: {}\ndate published: {}\n------------------\n", &name, chrono::offset::Utc::now().format("%d/%m/%Y %H:%M"));
    file.write_all(content.as_bytes()).context("writing file metadata")?;
    Ok(())
}

fn get_content_dir() -> Result<String, Error> {
    if !Path::new(METADATA_FILE).exists() {
        return Err(Error::msg("cms is not initialized, please call `cms init` to initzlize cms in the repository"))
    }

    let mut file = File::open(METADATA_FILE).context("open metadata file")?;
    let mut content_dir = String::new();
    file.read_to_string(&mut content_dir).context("read metadata context")?;
    Ok(content_dir)
}
