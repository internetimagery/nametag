use std::path::PathBuf;

use structopt::StructOpt;

use nametag::NameTag;



#[derive(Debug, StructOpt)]
#[structopt(about = "Work with tags on files, using a defined format.")]
enum Cli {
    Add {
        #[structopt(short)]
        tags: Vec<String>,
        #[structopt(parse(from_os_str))]
        paths: Vec<PathBuf>,
    },
    Remove {
    },
    Query {
    },
}



fn main() {
    match Cli::from_args() {
        Cli::Add{tags, paths} => {
            let nametags = paths.iter().map(|path| NameTag::new(path));
            println!(">tags: {:?}, paths {:?}", tags, paths);
        },
        Cli::Remove{} => {},
        Cli::Query{} => {},
    }
}
