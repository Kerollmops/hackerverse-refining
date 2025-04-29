use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Write as _;
use std::path::PathBuf;

use clap::Parser;
use memmap2::Mmap;
use serde::Deserialize;
use serde_json::{Deserializer, Value};

/// Takes an `.ndjson` file as input, the list of usernames and
/// outputs the input with IDs corresponding to the username instead
/// of the author ID.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Author4Post2Ndjson {
    /// The `users.ndjson` file.
    #[arg(long)]
    usernames: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let Author4Post2Ndjson { usernames } = Author4Post2Ndjson::parse();

    let usernames_file = File::open(usernames)?;
    let usernames_map = unsafe { Mmap::map(&usernames_file)? };
    let mut usernames = HashMap::new();
    for result in Deserializer::from_slice(&usernames_map[..]).into_iter() {
        let JustIdAndUsername { id, username } = result?;
        usernames.insert(id, username);
    }

    // Now we stream the url metas and modify the ids to the post ones
    let mut stdout = io::stdout();
    for result in Deserializer::from_reader(io::stdin()).into_iter() {
        let mut post: Value = result?;

        let post_id = post["id"].as_u64().unwrap();
        let author_id = post["author"].as_u64().unwrap() as u32;
        match usernames.get(&author_id) {
            Some(&username) => post["author"] = Value::String(username.to_owned()),
            None => eprintln!("Unknown author ID {author_id} in post ID {post_id}"),
        };

        serde_json::to_writer(&mut stdout, &post)?;
        writeln!(&mut stdout)?;
    }

    Ok(())
}

#[derive(Deserialize)]
struct JustIdAndUsername<'a> {
    id: u32,
    username: &'a str,
}
