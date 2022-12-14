use aes::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use clap::{ArgGroup, Parser};
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::process::exit;

const KEY: [u8; 32] = [
    0x05, 0xF7, 0x8F, 0x59, 0x53, 0xBF, 0xAB, 0xB1, 0xBE, 0x2D, 0x84, 0x0C, 0xB0, 0xE1, 0x3D, 0xB6,
    0x5F, 0x7F, 0xA5, 0xEF, 0xC9, 0x7F, 0xBA, 0xA6, 0x30, 0x0C, 0xA4, 0xE5, 0x33, 0xFA, 0x71, 0x7C,
];
const IV: [u8; 16] = [
    0xe9, 0x3e, 0xcc, 0x7c, 0x6e, 0x39, 0x4f, 0x3d, 0x43, 0x22, 0x82, 0xef, 0x18, 0x5c, 0xc3, 0xb1,
];

#[derive(Parser, Debug)]
#[clap(version, name = "undsm")]
#[clap(group(
ArgGroup::new("tokens")
    .required(true)
    .args(& ["unpack", "pack"]) // conflict
))]
struct Options {
    /// Unpack a .dsm file
    #[arg(short = 'u', long = "unpack")]
    unpack: bool,
    /// Pack a file to .dsm
    #[arg(short = 'p', long = "pack")]
    pack: bool,
    /// Force writing to output file
    #[arg(short = 'f', long = "force", default_value_t = false)]
    force: bool,
    /// Input file
    #[arg(short = 'i', long = "input")]
    input: PathBuf,
    /// Output file
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,
}

type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;

fn unpack(input: &PathBuf, output: &PathBuf) {
    println!(
        "Unpacking {} to {}",
        input.to_string_lossy(),
        output.to_string_lossy()
    );
    let input_file = File::open(input).unwrap();
    let buf_reader = BufReader::new(input_file);
    let base64_encoded = String::from_utf8_lossy(
        &buf_reader
            .bytes()
            .map(|b| b.unwrap())
            .filter(|b| {
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=".contains(b)
            })
            .collect::<Vec<u8>>(),
    )
    .to_string();
    let mut vec = base64::decode(&base64_encoded).unwrap();
    let aes = Aes256CbcDec::new_from_slices(&KEY, &IV).unwrap();
    let result = aes.decrypt_padded_mut::<Pkcs7>(&mut *vec).unwrap();
    let output_file = File::create(output).unwrap();
    let mut buf_writer = BufWriter::new(output_file);
    buf_writer.write_all(result).unwrap();
}

fn pack(input: &PathBuf, output: &PathBuf) {
    println!(
        "Packing {} to {}",
        input.to_string_lossy(),
        output.to_string_lossy()
    );
    let input_file = File::open(input).unwrap();
    let buf_reader = BufReader::new(input_file);
    let vec = buf_reader.bytes().map(|b| b.unwrap()).collect::<Vec<u8>>();
    let aes = Aes256CbcEnc::new_from_slices(&KEY, &IV).unwrap();
    let result = aes.encrypt_padded_vec_mut::<Pkcs7>(&*vec);
    let result_base64 = base64::encode(&result);
    let output_file = File::create(output).unwrap();
    let mut buf_writer = BufWriter::new(output_file);
    buf_writer.write_all(&*result_base64.as_bytes()).unwrap();
}

fn main() {
    let args = Options::parse();

    if !args.input.exists() {
        eprintln!("Input file does not exist");
        exit(1);
    }

    if !args.input.is_file() {
        eprintln!("Input file is not a file");
        exit(1);
    }

    let output = args.output.unwrap_or_else(|| {
        args.input.parent().unwrap().join(format!(
            "{}-{suffix}.txt",
            args.input.file_stem().unwrap().to_string_lossy(),
            suffix = if args.unpack { "unpack" } else { "pack" }
        ))
    });

    if output.exists() {
        if !args.force {
            eprintln!("Output file already exists, use -f to overwrite");
            exit(1);
        } else {
            fs::remove_file(&output).unwrap();
        }
    }

    if args.unpack {
        unpack(&args.input, &output);
    } else {
        pack(&args.input, &output);
    }
}
