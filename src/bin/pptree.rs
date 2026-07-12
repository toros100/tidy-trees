use clap::Parser;
use std::fs::File;

use tidy_trees::pretty_print;
use tidy_trees::tree::{Tree, TreeStructure};

/// prints a tree from a JSON shape
/// Tree = { "content": string, "children"?: Tree[] }
#[derive(clap::Parser)]
#[command(group(clap::ArgGroup::new("input").required(true).args(["file", "json"])))]
struct Args {
    /// path to JSON file
    file: Option<std::path::PathBuf>,

    /// inlined JSON string (single quoted)
    #[arg(short, long)]
    json: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let t: Tree<String> = {
        match args {
            Args { file: Some(p), .. } => {
                let file = File::open(p)?;
                serde_json::from_reader(file)?
            }
            Args { json: Some(s), .. } => serde_json::from_str(&s)?,
            _ => unreachable!("excluded by clap ArgGroup"),
        }
    };

    let mut ts = TreeStructure::default();
    let td = ts.load_data(t);
    pretty_print::print_tree(td);
    Ok(())
}
