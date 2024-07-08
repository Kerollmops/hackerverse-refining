use std::fs::File;
use std::io;
use std::io::Write as _;
use std::num::NonZeroU32;
use std::path::PathBuf;

use clap::Parser;
use hackernews_embs2ndjson::MatLEView;
use memmap2::Mmap;
use serde_json::{Deserializer, Number, Value};

/// Takes an `.ndjson` file as input, the matrix and outputs the input
/// with IDs corresponding to the posts instead of the URLs.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct GenUrlPostMatrix {
    /// The one-dimension `.mat` file copiling relations between the URLs and the posts.
    ///
    /// It is generated from the `posts.ndjson` file with the `gen-url-post-matrix` tool.
    #[arg(long)]
    url_post_matrix: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let GenUrlPostMatrix { url_post_matrix } = GenUrlPostMatrix::parse();

    let url_post_matrix_file = File::open(url_post_matrix)?;
    let url_post_matrix = unsafe { Mmap::map(&url_post_matrix_file)? };
    let url_post_matrix = MatLEView::<1, Option<NonZeroU32>>::new(&url_post_matrix);

    // Now we stream the url metas and modify the ids to the post ones
    let mut stdout = io::stdout();
    for result in Deserializer::from_reader(io::stdin()).into_iter() {
        let mut url_indexed: Value = result?;

        let url_indexed_id = url_indexed["id"].as_u64().unwrap() as usize;
        let post_id_opt = url_post_matrix.get(url_indexed_id).transpose().unwrap();
        match post_id_opt.and_then(|&[x]| x) {
            Some(post_id) => url_indexed["id"] = Value::Number(Number::from(post_id.get())),
            None => continue,
        }

        serde_json::to_writer(&mut stdout, &url_indexed)?;
        writeln!(&mut stdout)?;
    }

    Ok(())
}
