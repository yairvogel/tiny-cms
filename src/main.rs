use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Error;
use clap::{Parser, Subcommand};
use colored::Colorize;
use markdown;

const CONTENT_DIR: &str = "content";
const SRC_DIR: &str = "src";
const PUBLISH_DIR: &str = "publish";
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
    },
    Publish
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    match args.cmd {
        Command::Init { content_dir } => init(content_dir),
        Command::New { name } => new(name),
        Command::Publish => publish()
    }
}

fn init(content_dir: String) -> Result<(), Error> {
    if Path::new(METADATA_FILE).exists() {
        return Err(Error::msg("cms is already initialized"))
    }

    let mut file = File::create(METADATA_FILE).context("creating .cms file")?;
    file.write_all(content_dir.as_bytes()).context("writing .cms file")?;

    let src_dir = PathBuf::from(&content_dir).join("src");
    fs::create_dir_all(&src_dir).context("creating content directory")?;
    println!("initialized cms at {}", &content_dir);
    Ok(())
}

fn new(name: String) -> Result<(), Error> {
    let content_dir = get_content_dir().context("Could not get content directory")?;
    let src_dir = content_dir.join(SRC_DIR);
    let new_file_path = src_dir.join(&name);
    if new_file_path.exists() {
        return Err(Error::msg(format!("post name '{}' already exists", &name)))
    }

    let mut file = File::create(&new_file_path.with_extension("md")).context("Failed to create a new post file")?;
    let content = format!("------------------\nname: {}\ndate published: {}\n------------------\n", &name, chrono::offset::Utc::now().format("%d/%m/%Y %H:%M"));
    file.write_all(content.as_bytes()).context("writing file metadata")?;
    println!("created empty post named at {}", &new_file_path.to_str().or(Some(&name)).unwrap());
    Ok(())
}

fn publish() -> Result<(), Error> {
    let content_dir = get_content_dir().context("Could not get content directory")?;
    let src_dir = content_dir.join(SRC_DIR);
    let target_dir = content_dir.join(PUBLISH_DIR);
    if target_dir.exists() {
        fs::remove_dir_all(&target_dir).context("Failed cleaning the publish directory")?;
    }

    fs::create_dir(&target_dir).context("failed creating the publish directory")?;

    let mut published = 0;
    for entry in src_dir.read_dir().context("failed reading the source directory")? {
        let entry = entry.context("failed to read source directory entry")?;
        let mut source = File::open(&entry.path()).context("failed opening the source md file")?;

        let mut md_content = String::new();
        source.read_to_string(&mut md_content).context("failed reading the markdown file to string")?;

        // searching for the second line of dashes, pos will mark the point where the actual
        // content starts
        let mut pos = md_content.find("---").ok_or(Error::msg("couldn't find metadata section for markdown file"))?;
        pos += md_content[pos+1..].find("\n").ok_or(Error::msg("couldn't find metadata section for markdown file"))?;
        pos += md_content[pos+1..].find("---").ok_or(Error::msg("couldn't find metadata section for markdown file"))?;
        pos += md_content[pos+1..].find("\n").ok_or(Error::msg("couldn't find metadata section for markdown file"))?;
        pos += 2;

        let only_md = &md_content[pos..];
        if only_md.len() == 0 {
            let warn = format!("post '{}' is empty", &entry.file_name().to_str().unwrap_or("unknown"));
            println!("{}", warn.yellow());
        }

        let html = markdown::to_html(only_md);
        let mut target = File::create(target_dir.join(entry.file_name()).with_extension("html")).context("failed creating target html file")?;
        target.write_all(html.as_bytes()).context("failed writing html string to target html file")?;
        published += 1;
    }

    println!("published {} files", published);
    Ok(())
}

fn get_content_dir() -> Result<PathBuf, Error> {
    if !Path::new(METADATA_FILE).exists() {
        return Err(Error::msg("cms is not initialized, please call `cms init` to initzlize cms in the repository"))
    }

    let mut file = File::open(METADATA_FILE).context("open metadata file")?;
    let mut content_dir = String::new();
    file.read_to_string(&mut content_dir).context("read metadata context")?;
    Ok(PathBuf::from(content_dir))
}
