use std::{path::{PathBuf, Path}, fs::File, io::{Write, Read}};

use anyhow::{Error, Context};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use markdown::Block;
use syntect::parsing::SyntaxSet;

mod server;
mod parser;

const CONTENT_DIR: &str = "content";
const PUBLISH_DIR: &str = "publish";
const SRC_DIR: &str = "src";
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
        title: String
    },
    Publish {
        #[arg(long, default_value = PUBLISH_DIR)]
        publish_dir: String
    },
    Serve,
    Markdown
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    match args.cmd {
        Command::Init { content_dir } => init(content_dir),
        Command::New { title } => new(title),
        Command::Publish { publish_dir } => publish(&publish_dir),
        Command::Serve => serve(),
        Command::Markdown => markdown()
    }
}

#[derive(Debug)]
pub struct Post {
    pub title: String,
    pub published: DateTime::<Utc>,
    pub content: String
}

pub fn init(content_dir: String) -> Result<(), Error> {
    if Path::new(METADATA_FILE).exists() {
        return Err(Error::msg("cms is already initialized"))
    }

    let mut file = File::create(METADATA_FILE).context("creating .cms file")?;
    file.write_all(content_dir.as_bytes()).context("writing .cms file")?;

    let src_dir = PathBuf::from(&content_dir).join("src");
    std::fs::create_dir_all(&src_dir).context("creating content directory")?;
    println!("initialized cms at {}", &content_dir);
    Ok(())
}

fn new(title: String) -> Result<(), Error> {
    let content_dir = get_content_dir().context("Could not get content directory")?;
    let src_dir = content_dir.join(SRC_DIR);
    let new_file_path = src_dir.join(&title);
    if new_file_path.exists() {
        return Err(Error::msg(format!("post titled '{}' already exists", &title)))
    }

    let mut file = File::create(&new_file_path.with_extension("md")).context("Failed to create a new post file")?;
    let content = format!("------------------\ntitle: {}\ndate published: {}\n------------------\n", &title, chrono::offset::Utc::now().format("%d/%m/%Y %H:%M"));
    file.write_all(content.as_bytes()).context("writing file metadata")?;
    println!("created empty post titled {}", &new_file_path.to_str().or(Some(&title)).unwrap());
    Ok(())
}

fn publish(publish_dir: &str) -> Result<(), Error> {
    let content_dir = get_content_dir().context("Could not get content directory")?;
    let src_dir = content_dir.join(SRC_DIR);
    let target_dir = content_dir.join(publish_dir);
    if target_dir.exists() {
        std::fs::remove_dir_all(&target_dir).context("Failed cleaning the publish directory")?;
    }

    std::fs::create_dir(&target_dir).context("failed creating the publish directory")?;

    let mut published = 0;
    for entry in src_dir.read_dir().context("failed reading the source directory")? {
        let entry = entry.context("failed to read source directory entry")?;
        let mut source = File::open(&entry.path()).context("failed opening the source md file")?;

        let post = parser::parse(&mut source)?;

        if post.content.len() == 0 {
            let warn = format!("post '{}' is empty", &entry.file_name().to_str().unwrap_or("unknown"));
            println!("{}", warn.yellow());
        }

        let html = markdown::to_html(&post.content);
        let mut target = File::create(target_dir.join(entry.file_name()).with_extension("html")).context("failed creating target html file")?;
        target.write_all(html.as_bytes()).context("failed writing html string to target html file")?;
        published += 1;
    }

    println!("published {} files", published);
    Ok(())
}

fn markdown() -> Result<(), Error> {
    let content_dir = get_content_dir().context("Could not get content directory")?;
    let src_dir = content_dir.join(SRC_DIR);
    for entry in src_dir.read_dir().context("failed reading the source directory")? {
        let entry = entry.context("failed to read source directory entry")?;
        let mut source = File::open(&entry.path()).context("failed opening the source md file")?;

        let post = parser::parse(&mut source)?;
        let md = markdown::tokenize(&post.content);
        println!("{} blocks:", &entry.file_name().to_str().unwrap_or("unknown"));
        for block in md {
            println!("\t{:?}", block);
        }
    }

    Ok(())
}

fn serve() -> Result<(), Error> {
    server::run_server("");
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

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::path::Path;
    use test_context::{TestContext, test_context};
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;

    struct DirectoryContext {
        directory: String
    }

    impl TestContext for DirectoryContext {
        fn setup() -> Self {
            let directory = thread_rng().sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from)
                .collect();
            println!("{}\n", &directory);
            std::fs::create_dir(&directory).unwrap();
            std::env::set_current_dir(&directory).unwrap();
            Self { directory }
        }

        fn teardown(self) {
            std::env::set_current_dir("..").unwrap();
            std::fs::remove_dir_all(&self.directory).unwrap();
        }
    }

    #[test_context(DirectoryContext)]
    #[test]
    fn init(_: &mut DirectoryContext) {
        crate::init("init".to_string()).expect("should be able to init");
        assert_eq!(Path::new("init/src").exists(), true, "failed to create content dir");
        assert_eq!(Path::new(".cms").exists(), true, "failed to create .cms file");
    }

    #[test_context(DirectoryContext)]
    #[test]
    fn new_empty_post(_: &mut DirectoryContext) {
        crate::init("init".to_string()).expect("failed to init");
        crate::new("hello-world".to_string()).expect("failed to new");

        let path = Path::new("init/src/hello-world.md");
        assert_eq!(path.exists(), true);

        let mut file = std::fs::File::open(path).unwrap();
        let post = crate::parser::parse(&mut file).unwrap();

        assert_eq!(post.title.as_str(), "hello-world");
        assert_eq!(post.published > chrono::Utc::now() - chrono::TimeDelta::try_minutes(1).unwrap(), true);
        assert_eq!(post.content.is_empty(), true);
    }

    #[test_context(DirectoryContext)]
    #[test]
    fn publish_empty_post(_: &mut DirectoryContext) {
        crate::init("init".to_string()).expect("failed to init");
        crate::new("hello-world".to_string()).expect("failed to new");
        crate::publish(crate::PUBLISH_DIR).expect("failed to publish");
        
        let path = Path::new("init/publish/hello-world.html");
        assert_eq!(path.exists(), true);
        let mut file = std::fs::File::open(path).unwrap();
        let mut buf = Vec::new();
        let len = file.read_to_end(&mut buf).unwrap();
        assert_eq!(len, 1);
        assert_eq!(buf[0], b'\n');
    }

    #[test_context(DirectoryContext)]
    #[test]
    fn publish_html_conversion(_: &mut DirectoryContext) {
        crate::init("init".to_string()).expect("failed to init");
        crate::new("hello-world".to_string()).expect("failed to new");
        let path = Path::new("init/src/hello-world.md");
        let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
        file.write_all(b"\n# h1 title\na paragraph").expect("failed to write to file");
        crate::publish(crate::PUBLISH_DIR).expect("failed to publish");

        let path = Path::new("init/publish/hello-world.html");
        assert_eq!(path.exists(), true);
        let mut buf = Vec::new();
        _ = std::fs::File::open(path).unwrap().read_to_end(&mut buf);
        let str = std::str::from_utf8(&buf).unwrap();

        assert_eq!(str.trim(), "<h1 id='h1_title'>h1 title</h1>\n\n<p>a paragraph</p>");
    }
}
