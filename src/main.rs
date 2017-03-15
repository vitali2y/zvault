extern crate serde;
extern crate rmp_serde;
#[macro_use] extern crate serde_utils;
extern crate squash_sys as squash;
extern crate mmap;
extern crate blake2_rfc as blake2;
extern crate murmurhash3;
extern crate serde_yaml;
#[macro_use] extern crate quick_error;
extern crate docopt;
extern crate rustc_serialize;

mod errors;
mod util;
mod bundle;
mod index;
mod chunker;
mod repository;
mod algotest;

use chunker::ChunkerType;
use repository::{Repository, Config, Mode, Inode};
use util::{ChecksumType, Compression, HashMethod, to_file_size};

use std::fs::File;
use std::io::Read;
use std::time;

use docopt::Docopt;


static USAGE: &'static str = "
Usage:
    zvault init <repo>
    zvault info <repo>
    zvault bundles <repo>
    zvault check [--full] <repo>
    zvault algotest <path>
    zvault test <repo> <path>
    zvault stat <path>
    zvault put <repo> <path>

Options:
    --full                     Whether to verify the repository by loading all bundles
    --bundle-size SIZE         The target size of a full bundle in MiB [default: 25]
    --chunker METHOD           The chunking algorithm to use [default: fastcdc]
    --chunk-size SIZE          The target average chunk size in KiB [default: 8]
    --compression COMPRESSION  The compression to use [default: brotli/3]
";


#[derive(RustcDecodable, Debug)]
struct Args {
    cmd_init: bool,
    cmd_info: bool,
    cmd_algotest: bool,
    cmd_test: bool,
    cmd_stat: bool,
    cmd_check: bool,
    cmd_bundles: bool,
    cmd_put: bool,
    arg_repo: Option<String>,
    arg_path: Option<String>,
    flag_full: bool,
    flag_bundle_size: usize,
    flag_chunker: String,
    flag_chunk_size: usize,
    flag_compression: String
}


fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    //println!("{:?}", args);

    if args.cmd_algotest {
        algotest::run(&args.arg_path.unwrap());
        return
    }

    if args.cmd_init {
        let chunker = ChunkerType::from(&args.flag_chunker, args.flag_chunk_size*1024, 0).expect("No such chunk algorithm");
        let compression = if args.flag_compression == "none" {
            None
        } else {
            Some(Compression::from_string(&args.flag_compression).expect("Failed to parse compression"))
        };
        Repository::create(&args.arg_repo.unwrap(), Config {
            bundle_size: args.flag_bundle_size*1024*1024,
            checksum: ChecksumType::Blake2_256,
            chunker: chunker,
            compression: compression,
            hash: HashMethod::Blake2
        }).unwrap();
        return
    }

    if args.cmd_stat {
        println!("{:?}", Inode::get_from(&args.arg_path.unwrap()).unwrap());
        return
    }

    let mut repo = Repository::open(&args.arg_repo.unwrap()).unwrap();

    if args.cmd_check {
        repo.check(args.flag_full).unwrap();
        return
    }

    if args.cmd_info {
        let info = repo.info();
        println!("Bundles: {}", info.bundle_count);
        println!("Total size: {}", to_file_size(info.encoded_data_size));
        println!("Uncompressed size: {}", to_file_size(info.raw_data_size));
        println!("Compression ratio: {:.1}", info.compression_ratio * 100.0);
        println!("Chunk count: {}", info.chunk_count);
        println!("Average chunk size: {}", to_file_size(info.avg_chunk_size as u64));
        return
    }

    if args.cmd_bundles {
        for bundle in repo.list_bundles() {
            println!("Bundle {}", bundle.id);
            println!("  - Chunks: {}", bundle.chunk_count);
            println!("  - Size: {}", to_file_size(bundle.encoded_size as u64));
            println!("  - Data size: {}", to_file_size(bundle.raw_size as u64));
            let ratio = bundle.encoded_size as f32 / bundle.raw_size as f32;
            let compression = if let Some(ref c) = bundle.compression {
                c.to_string()
            } else {
                "none".to_string()
            };
            println!("  - Compression: {}, ratio: {:.1}%", compression, ratio * 100.0);
            println!();
        }
        return
    }

    if args.cmd_put {
        let chunks = repo.put_inode(&args.arg_path.unwrap()).unwrap();
        println!("done. {} chunks, total size: {}", chunks.len(), to_file_size(chunks.iter().map(|&(_,s)| s).sum::<usize>() as u64));
        return
    }

    if args.cmd_test {
        print!("Integrity check before...");
        repo.check(true).unwrap();
        println!(" done.");

        let file_path = args.arg_path.unwrap();
        print!("Reading file {}...", file_path);
        let mut data = Vec::new();
        let mut file = File::open(file_path).unwrap();
        file.read_to_end(&mut data).unwrap();
        println!(" done. {} bytes", data.len());

        print!("Adding data to repository...");
        let start = time::Instant::now();
        let chunks = repo.put_data(Mode::Content, &data).unwrap();
        repo.flush().unwrap();
        let elapsed = start.elapsed();
        let duration = elapsed.as_secs() as f64 * 1.0 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
        let write_speed = data.len() as f64 / duration;
        println!(" done. {} chunks, {:.1} MB/s", chunks.len(), write_speed / 1_000_000.0);

        println!("Integrity check after...");
        repo.check(true).unwrap();
        println!(" done.");

        print!("Reading data from repository...");
        let start = time::Instant::now();
        let data2 = repo.get_data(&chunks).unwrap();
        let elapsed = start.elapsed();
        let duration = elapsed.as_secs() as f64 * 1.0 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
        let read_speed = data.len() as f64 / duration;
        assert_eq!(data.len(), data2.len());
        println!(" done. {:.1} MB/s", read_speed / 1_000_000.0);
    }
}