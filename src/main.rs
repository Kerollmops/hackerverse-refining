use std::fs::File;
use std::io::{self, stdout, Write};
use std::marker::PhantomData;
use std::mem;
use std::path::PathBuf;

use bytemuck::{AnyBitPattern, PodCastError};
use clap::Parser;
use memmap2::Mmap;
use serde::ser::Serialize;
use serde_json::ser::{CompactFormatter, Serializer};
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

struct MatLEView<'m, const DIM: usize, T> {
    bytes: &'m [u8],
    _marker: PhantomData<T>,
}

impl<const DIM: usize, T: AnyBitPattern> MatLEView<'_, DIM, T> {
    pub fn new(bytes: &[u8]) -> MatLEView<DIM, T> {
        assert!(bytes.len() % DIM == 0);
        MatLEView { bytes, _marker: PhantomData }
    }

    pub fn get(&self, index: usize) -> Option<Result<&[T; DIM], PodCastError>> {
        let tsize = mem::size_of::<T>();
        if (index * DIM + DIM) * tsize < self.bytes.len() {
            let start = index * DIM;
            let bytes = &self.bytes[start * tsize..(start + DIM) * tsize];
            match bytemuck::try_cast_slice::<u8, T>(bytes) {
                Ok(slice) => Some(Ok(slice.try_into().unwrap())),
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        }
    }
}

struct SmallFloatsFormatter {
    float_precision: usize,
    compact: serde_json::ser::CompactFormatter,
}

impl SmallFloatsFormatter {
    fn with_float_precision(float_precision: usize) -> Self {
        SmallFloatsFormatter { float_precision, compact: CompactFormatter }
    }
}

impl serde_json::ser::Formatter for SmallFloatsFormatter {
    fn write_null<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_null(writer)
    }

    fn write_bool<W>(&mut self, writer: &mut W, value: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_bool(writer, value)
    }

    fn write_i8<W>(&mut self, writer: &mut W, value: i8) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_i8(writer, value)
    }

    fn write_i16<W>(&mut self, writer: &mut W, value: i16) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_i16(writer, value)
    }

    fn write_i32<W>(&mut self, writer: &mut W, value: i32) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_i32(writer, value)
    }

    fn write_i64<W>(&mut self, writer: &mut W, value: i64) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_i64(writer, value)
    }

    fn write_i128<W>(&mut self, writer: &mut W, value: i128) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_i128(writer, value)
    }

    fn write_u8<W>(&mut self, writer: &mut W, value: u8) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_u8(writer, value)
    }

    fn write_u16<W>(&mut self, writer: &mut W, value: u16) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_u16(writer, value)
    }

    fn write_u32<W>(&mut self, writer: &mut W, value: u32) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_u32(writer, value)
    }

    fn write_u64<W>(&mut self, writer: &mut W, value: u64) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_u64(writer, value)
    }

    fn write_u128<W>(&mut self, writer: &mut W, value: u128) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_u128(writer, value)
    }

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

    fn write_number_str<W>(&mut self, writer: &mut W, value: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_number_str(writer, value)
    }

    fn begin_string<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.begin_string(writer)
    }

    fn end_string<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.end_string(writer)
    }

    fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_string_fragment(writer, fragment)
    }

    fn write_char_escape<W>(
        &mut self,
        writer: &mut W,
        char_escape: serde_json::ser::CharEscape,
    ) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_char_escape(writer, char_escape)
    }

    fn write_byte_array<W>(&mut self, writer: &mut W, value: &[u8]) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_byte_array(writer, value)
    }

    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.begin_array(writer)
    }

    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.end_array(writer)
    }

    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.begin_array_value(writer, first)
    }

    fn end_array_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.end_array_value(writer)
    }

    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.begin_object(writer)
    }

    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.end_object(writer)
    }

    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.begin_object_key(writer, first)
    }

    fn end_object_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.end_object_key(writer)
    }

    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.begin_object_value(writer)
    }

    fn end_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.end_object_value(writer)
    }

    fn write_raw_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.compact.write_raw_fragment(writer, fragment)
    }
}
