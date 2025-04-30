use std::io;
use std::io::Write as _;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File};

use clap::Parser;
use serde::Deserialize;
use serde_json::{Deserializer, Number, Value};

/// Takes an `.ndjson` file as input, the list of urls and
/// outputs the input with the ID of the post instead of the URLs.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Url4Post2Ndjson {
    /// The `posts.ndjson` file.
    #[arg(long)]
    posts: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let Url4Post2Ndjson { posts } = Url4Post2Ndjson::parse();

    let posts_file = File::open(posts)?;
    let mut url_to_posts = HashMap::<_, Vec<_>>::new();

    eprintln!("Preparing the url meta ids table...");
    for (i, result) in Deserializer::from_reader(posts_file).into_iter().enumerate() {
        let JustIdAndUrl { id: post_id, url: url_id } = result?;

        if i % 1000 == 0 {
            eprintln!("{i} posts seen so far");
        }

        url_to_posts.entry(url_id).or_default().push(post_id);
    }

    // Now we stream the url metas and modify the ids to the post ones
    let mut stdout = io::stdout();
    for result in Deserializer::from_reader(io::stdin()).into_iter() {
        let mut url_indexed: Value = result?;
        let url_indexed_id = url_indexed["id"].as_u64().unwrap() as u32;
        let post_ids_opt = url_to_posts.get(&url_indexed_id).unwrap();

        for post_id in post_ids_opt {
            url_indexed["id"] = Value::Number(Number::from(*post_id));
            serde_json::to_writer(&mut stdout, &url_indexed)?;
            writeln!(&mut stdout)?;
        }
    }

    Ok(())
}

#[derive(Deserialize)]
struct JustIdAndUrl {
    id: u32,
    url: u32,
}
