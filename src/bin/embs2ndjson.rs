use std::fs::File;
use std::io::{self, stdout, Write};
use std::path::PathBuf;

use clap::Parser;
use hackerverse_refining::MatLEView;
use memmap2::Mmap;
use serde::ser::Serialize;
use serde_json::ser::Serializer;
use serde_json::{Number, Value};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct HackernewsEmbs2Ndjson {
    /// The `.mat` file corresponding to the ids of the embeddings
    embs_ids_path: PathBuf,
    /// The `.mat` file with the embeddings
    embs_data_path: PathBuf,

    /// The float precision
    #[arg(long, default_value_t = 2)]
    float_precision: usize,
}

fn main() -> anyhow::Result<()> {
    let HackernewsEmbs2Ndjson { embs_ids_path, embs_data_path, float_precision } =
        HackernewsEmbs2Ndjson::parse();

    let embs_ids_file = File::open(embs_ids_path)?;
    let embs_ids = unsafe { Mmap::map(&embs_ids_file)? };
    let embs_ids = MatLEView::<1, u32>::new(&embs_ids);

    let embs_data_file = File::open(embs_data_path)?;
    let embs_data = unsafe { Mmap::map(&embs_data_file)? };
    let embs_data = MatLEView::<512, f32>::new(&embs_data);

    // Prepare the object with the fields
    let mut object = serde_json::Map::new();
    object.insert("id".to_string(), Value::Null);
    object.insert("embeddings".to_string(), Value::Null);

    let mut stdout = stdout();

    for i in 0.. {
        match (embs_ids.get(i), embs_data.get(i)) {
            (Some(Ok([id])), Some(Ok(emb))) => {
                *object.get_mut("id").unwrap() = Value::Number(Number::from(*id));
                *object.get_mut("embeddings").unwrap() = Value::Array(
                    emb.iter()
                        .map(|&f| Number::from_f64(f as f64).unwrap())
                        .map(Into::into)
                        .collect(),
                );

                let mut serializer = Serializer::with_formatter(
                    &mut stdout,
                    SmallFloatsFormatter::with_float_precision(float_precision),
                );
                object.serialize(&mut serializer)?;
                stdout.write_all(b"\n")?;
            }
            (None, None) => break,
            (Some(Err(e)), _) | (_, Some(Err(e))) => panic!("{e:?}"),
            (_, _) => anyhow::bail!("number of values in the ids and data is not consistent"),
        }
    }

    Ok(())
}

struct SmallFloatsFormatter {
    float_precision: usize,
}

impl SmallFloatsFormatter {
    fn with_float_precision(float_precision: usize) -> Self {
        SmallFloatsFormatter { float_precision }
    }
}

impl serde_json::ser::Formatter for SmallFloatsFormatter {
    fn write_f32<W>(&mut self, writer: &mut W, value: f32) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let precision = self.float_precision;
        write!(writer, "{value:.precision$}")
    }

    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let precision = self.float_precision;
        write!(writer, "{value:.precision$}")
    }
}
