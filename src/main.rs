use clap::Parser;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
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
#[clap(version, name = "unhaha")]
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
    output: PathBuf,
}

fn decrypt_aes_cbc(key: &[u8], iv: &[u8; 16], data: &mut [u8]) {
    let mut ase256cbc = crypto2::blockmode::Aes256Cbc::new(key);
    ase256cbc.decrypt(iv, data)
}

fn encrypt_aes_cbc(key: &[u8], iv: &[u8; 16], data: &mut [u8]) {
    let mut ase256cbc = crypto2::blockmode::Aes256Cbc::new(key);
    ase256cbc.encrypt(iv, data)
}

fn unpack(input: &PathBuf, output: &PathBuf) {
    println!(
        "Unpacking {} to {}",
        input.to_string_lossy(),
        output.to_string_lossy()
    );
    let input_file = File::open(input).unwrap();
    let input_file_content_string = String::from_utf8_lossy(
        &input_file
            .bytes()
            .map(|b| b.unwrap())
            .filter(|b| {
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=".contains(b)
            })
            .collect::<Vec<u8>>(),
    )
    .to_string();
    let mut input_file_base64_array = base64::decode(&input_file_content_string).unwrap();
    decrypt_aes_cbc(&KEY, &IV, &mut input_file_base64_array);
    let mut output_file = File::create(output).unwrap();
    output_file.write_all(&*input_file_base64_array).unwrap();
}

fn pack(input: &PathBuf, output: &PathBuf) {
    println!(
        "Packing {} to {}",
        input.to_string_lossy(),
        output.to_string_lossy()
    );
    let input_file = File::open(input).unwrap();
    let mut input_file_array = input_file.bytes().map(|b| b.unwrap()).collect::<Vec<u8>>();
    encrypt_aes_cbc(&KEY, &IV, &mut input_file_array);
    let result_base64 = base64::encode(&input_file_array);
    let mut output_file = File::create(output).unwrap();
    output_file.write_all(&*result_base64.as_bytes()).unwrap();
}

fn main() {
    let args = Options::parse();

    if args.unpack == args.pack {
        println!("You must specify either -u or -p");
        exit(1);
    }

    if args.output.exists() {
        if !args.force {
            eprintln!("Output file already exists, use -f to overwrite");
            exit(1);
        } else {
            fs::remove_file(&args.output).unwrap();
        }
    }

    if !args.input.exists() {
        eprintln!("Input file does not exist");
        exit(1);
    }

    if !args.input.is_file() {
        eprintln!("Input file is not a file");
        exit(1);
    }

    if args.unpack {
        unpack(&args.input, &args.output);
    } else {
        pack(&args.input, &args.output);
    }
}
