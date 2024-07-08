use std::io::{self, BufWriter, Write};

use clap::Parser;
use serde::Deserialize;
use serde_json::Deserializer;

/// Generates a one-dimension matrix of the post ids (in little endian).
/// The index corresponds to the URL id.
/// Zero (0) means no corresponding post is associated to this URL.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct GenUrlPostMatrix {}

fn main() -> anyhow::Result<()> {
    let GenUrlPostMatrix {} = Parser::parse();

    // First we create the matrix to associate post ids to the corresponding url ids
    let stdin = io::stdin();
    eprintln!("Preparing the url meta ids table...");
    let mut post_ids = Vec::<u32>::new();
    for (i, result) in Deserializer::from_reader(stdin).into_iter().enumerate() {
        let JustIdAndUrl { id: post_id, url: url_id } = result?;

        if i % 1000 == 0 {
            eprintln!("{i} posts seen so far");
        }

        let url_meta_id = url_id as usize;
        post_ids.resize(url_meta_id + 1, 0);
        post_ids[url_meta_id] = post_id as u32;
    }

    let mut writer = BufWriter::new(io::stdout());
    post_ids.iter_mut().for_each(|x| *x = x.to_le());
    let bytes = bytemuck::cast_slice(&post_ids);
    writer.write_all(bytes)?;

    Ok(())
}

#[derive(Deserialize)]
struct JustIdAndUrl {
    id: u64,
    url: u64,
}
