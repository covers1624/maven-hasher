use clap::{app_from_crate, Arg};
use std::fs::File;
use std::path::Path;
use walkdir::WalkDir;
use task_queue::TaskQueue;
use ring::digest::{Context, Digest, SHA256, Algorithm, SHA1_FOR_LEGACY_USE_ONLY, SHA512};
use std::io::{Read, Write};
use std::io::Result;
use data_encoding::HEXLOWER;

static HASH_EXTS: [&str; 4] = [
    "md5",
    "sha1",
    "sha256",
    "sha512"
];

fn main() {
    let matches = app_from_crate!()
        .arg(
            Arg::new("repo")
                .short('r')
                .long("repo")
                .value_name("FOLDER")
                .about("The folder on disk representing the maven repository.")
                .required(true)
                .takes_value(true)
        )
        .arg(
            Arg::new("threads")
                .short('t')
                .long("threads")
                .value_name("THREADS")
                .about("The number of threads to use when processing files.")
                .takes_value(true)
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                // .multiple_occurrences(true)
                .about("Tells you what its hashing.")
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                // .multiple_occurrences(true)
                .about("Doesn't hash things, just tells you what it will do. Implies verbose.")
        )
        .get_matches();
    let repo = Path::new(matches.value_of("repo").unwrap());

    let threads;
    if matches.is_present("threads") {
        threads = matches.value_of("threads").unwrap().parse::<usize>().unwrap()
    } else {
        threads = num_cpus::get()
    }

    let dry_run = matches.is_present("dry-run");
    let verbose = dry_run || matches.is_present("verbose");

    let mut task_queue = TaskQueue::with_threads(1, threads).unwrap();

    for entry in WalkDir::new(repo)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok()) {
        task_queue.enqueue(move || {
            let f_path = entry.path();
            let f_name = entry.file_name().to_string_lossy();
            if !HASH_EXTS.iter().any(|e| f_name.ends_with(*e)) && entry.file_type().is_file() {
                for hash_ext in HASH_EXTS.iter() {
                    let hash_path = f_path.clone().with_extension(f_path.extension().unwrap().to_string_lossy().to_string() + "." + hash_ext);
                    if !hash_path.exists() {
                        let reader = File::open(f_path).unwrap();
                        let hash;
                        if verbose {
                            println!("Computing {} for {:?}", hash_ext, f_path);
                        }
                        if !dry_run {
                            if *hash_ext == "md5" {
                                hash = format!("{:x}", md5_digest(reader).unwrap());
                            } else if *hash_ext == "sha1" {
                                hash = HEXLOWER.encode(sha_digest(reader, &SHA1_FOR_LEGACY_USE_ONLY).unwrap().as_ref())
                            } else if *hash_ext == "sha256" {
                                hash = HEXLOWER.encode(sha_digest(reader, &SHA256).unwrap().as_ref())
                            } else if *hash_ext == "sha512" {
                                hash = HEXLOWER.encode(sha_digest(reader, &SHA512).unwrap().as_ref())
                            } else {
                                panic!("Unknown hash format! {}", hash_ext)
                            }
                            let mut hash_file = File::create(hash_path).unwrap();
                            hash_file.write_all(hash.as_bytes()).unwrap();
                        }
                    }
                }
            }
        }).unwrap()
    }
    task_queue.stop_wait()
}

fn sha_digest<R: Read>(mut reader: R, algorithm: &'static Algorithm) -> Result<Digest> {
    let mut context = Context::new(algorithm);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count])
    }
    Ok(context.finish())
}

fn md5_digest<R: Read>(mut reader: R) -> Result<md5::Digest> {
    let mut context = md5::Context::new();
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.consume(&buffer[..count])
    }
    Ok(context.compute())
}
