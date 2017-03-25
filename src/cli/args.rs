use ::prelude::*;
use super::*;

use std::process::exit;


pub enum Arguments {
    Init {
        repo_path: String,
        bundle_size: usize,
        chunker: ChunkerType,
        compression: Option<Compression>,
        encryption: bool,
        hash: HashMethod,
        remote_path: String
    },
    Backup {
        repo_path: String,
        backup_name: String,
        src_path: String,
        full: bool,
        reference: Option<String>,
        same_device: bool,
        excludes: Vec<String>,
        excludes_from: Option<String>
    },
    Restore {
        repo_path: String,
        backup_name: String,
        inode: Option<String>,
        dst_path: String
    },
    Remove {
        repo_path: String,
        backup_name: String,
        inode: Option<String>
    },
    Prune {
        repo_path: String,
        prefix: String,
        daily: Option<usize>,
        weekly: Option<usize>,
        monthly: Option<usize>,
        yearly: Option<usize>,
        force: bool
    },
    Vacuum {
        repo_path: String,
        ratio: f32,
        force: bool
    },
    Check {
        repo_path: String,
        backup_name: Option<String>,
        inode: Option<String>,
        full: bool
    },
    List {
        repo_path: String,
        backup_name: Option<String>,
        inode: Option<String>
    },
    Info {
        repo_path: String,
        backup_name: Option<String>,
        inode: Option<String>
    },
    Analyze {
        repo_path: String
    },
    BundleList {
        repo_path: String
    },
    BundleInfo {
        repo_path: String,
        bundle_id: BundleId
    },
    Import {
        repo_path: String,
        remote_path: String,
        key_files: Vec<String>
    },
    Configure {
        repo_path: String,
        bundle_size: Option<usize>,
        chunker: Option<ChunkerType>,
        compression: Option<Option<Compression>>,
        encryption: Option<Option<PublicKey>>,
        hash: Option<HashMethod>
    },
    GenKey {
        file: Option<String>
    },
    AddKey {
        repo_path: String,
        file: Option<String>,
        set_default: bool
    },
    AlgoTest {
        file: String,
        bundle_size: usize,
        chunker: ChunkerType,
        compression: Option<Compression>,
        encrypt: bool,
        hash: HashMethod
    }
}


pub fn split_repo_path(repo_path: &str) -> (&str, Option<&str>, Option<&str>) {
    let mut parts = repo_path.splitn(3, "::");
    let repo = parts.next().unwrap();
    let backup = parts.next();
    let inode = parts.next();
    (repo, backup, inode)
}

fn parse_num(num: &str, name: &str) -> u64 {
    if let Ok(num) = num.parse::<u64>() {
        num
    } else {
        error!("{} must be a number, was '{}'", name, num);
        exit(1);
    }
}

fn parse_chunker(val: &str) -> ChunkerType {
    if let Ok(chunker) = ChunkerType::from_string(val) {
        chunker
    } else {
        error!("Invalid chunker method/size: {}", val);
        exit(1);
    }
}

fn parse_compression(val: &str) -> Option<Compression> {
    if val == "none" {
        return None
    }
    if let Ok(compression) = Compression::from_string(val) {
        Some(compression)
    } else {
        error!("Invalid compression method/level: {}", val);
        exit(1);
    }
}

fn parse_public_key(val: &str) -> PublicKey {
    let bytes = match parse_hex(val) {
        Ok(bytes) => bytes,
        Err(_) => {
            error!("Invalid key: {}", val);
            exit(1);
        }
    };
    if let Some(key) = PublicKey::from_slice(&bytes) {
        key
    } else {
        error!("Invalid key: {}", val);
        exit(1);
    }
}

fn parse_hash(val: &str) -> HashMethod {
    if let Ok(hash) = HashMethod::from(val) {
        hash
    } else {
        error!("Invalid hash method: {}", val);
        exit(1);
    }
}

fn parse_bundle_id(val: &str) -> BundleId {
    if let Ok(hash) = Hash::from_string(val) {
        BundleId(hash)
    } else {
        error!("Invalid bundle id: {}", val);
        exit(1);
    }
}

#[allow(unknown_lints,cyclomatic_complexity)]
pub fn parse() -> Arguments {
    let args = clap_app!(zvault =>
        (version: crate_version!())
        (author: crate_authors!(",\n"))
        (about: crate_description!())
        (@setting SubcommandRequiredElseHelp)
        (@setting GlobalVersion)
        (@setting VersionlessSubcommands)
        (@setting UnifiedHelpMessage)
        (@subcommand init =>
            (about: "initializes a new repository")
            (@arg bundle_size: --bundlesize +takes_value "maximal bundle size in MiB [default: 25]")
            (@arg chunker: --chunker +takes_value "chunker algorithm [default: fastcdc/8]")
            (@arg compression: --compression -c +takes_value "compression to use [default: brotli/3]")
            (@arg encryption: --encryption -e "generate a keypair and enable encryption")
            (@arg hash: --hash +takes_value "hash method to use [default: blake2]")
            (@arg remote: --remote -r +takes_value +required "path to the mounted remote storage")
            (@arg REPO: +required "path of the repository")
        )
        (@subcommand backup =>
            (about: "creates a new backup")
            (@arg full: --full "create a full backup")
            (@arg reference: --ref +takes_value "the reference backup to use for partial backup")
            (@arg same_device: --xdev -x "do not cross filesystem boundaries")
            (@arg exclude: --exclude -e ... +takes_value "exclude this path or file")
            (@arg excludes_from: --excludesfrom +takes_value "read the list of exludes from this file")
            (@arg SRC: +required "source path to backup")
            (@arg BACKUP: +required "repository::backup path")
        )
        (@subcommand restore =>
            (about: "restores a backup (or subpath)")
            (@arg BACKUP: +required "repository::backup[::subpath] path")
            (@arg DST: +required "destination path for backup")
        )
        (@subcommand remove =>
            (about: "removes a backup or a subpath")
            (@arg BACKUP: +required "repository::backup[::subpath] path")
        )
        (@subcommand prune =>
            (about: "removes backups based on age")
            (@arg prefix: --prefix +takes_value "only consider backups starting with this prefix")
            (@arg daily: --daily +takes_value "keep this number of daily backups")
            (@arg weekly: --weekly +takes_value "keep this number of weekly backups")
            (@arg monthly: --monthly +takes_value "keep this number of monthly backups")
            (@arg yearly: --yearly +takes_value  "keep this number of yearly backups")
            (@arg force: --force -f "actually run the prunce instead of simulating it")
            (@arg REPO: +required "path of the repository")
        )
        (@subcommand vacuum =>
            (about: "saves space by combining and recompressing bundles")
            (@arg ratio: --ratio -r +takes_value "ratio in % of unused space in a bundle to rewrite that bundle")
            (@arg force: --force -f "actually run the vacuum instead of simulating it")
            (@arg REPO: +required "path of the repository")
        )
        (@subcommand check =>
            (about: "checks the repository, a backup or a backup subpath")
            (@arg full: --full "also check file contents")
            (@arg PATH: +required "repository[::backup] path")
        )
        (@subcommand list =>
            (about: "lists backups or backup contents")
            (@arg PATH: +required "repository[::backup[::subpath]] path")
        )
        (@subcommand bundlelist =>
            (about: "lists bundles in a repository")
            (@arg REPO: +required "path of the repository")
        )
        (@subcommand bundleinfo =>
            (about: "lists bundles in a repository")
            (@arg REPO: +required "path of the repository")
            (@arg BUNDLE: +required "the bundle id")
        )
        (@subcommand import =>
            (about: "reconstruct a repository from the remote files")
            (@arg key: --key -k ... +takes_value "a file with a needed to read the bundles")
            (@arg REMOTE: +required "remote repository path")
            (@arg REPO: +required "path of the local repository to create")
        )
        (@subcommand info =>
            (about: "displays information on a repository, a backup or a path in a backup")
            (@arg PATH: +required "repository[::backup[::subpath]] path")
        )
        (@subcommand analyze =>
            (about: "analyze the used and reclaimable space of bundles")
            (@arg REPO: +required "repository path")
        )
        (@subcommand configure =>
            (about: "changes the configuration")
            (@arg REPO: +required "path of the repository")
            (@arg bundle_size: --bundlesize +takes_value "maximal bundle size in MiB [default: 25]")
            (@arg chunker: --chunker +takes_value "chunker algorithm [default: fastcdc/16]")
            (@arg compression: --compression -c +takes_value "compression to use [default: brotli/3]")
            (@arg encryption: --encryption -e +takes_value "the public key to use for encryption")
            (@arg hash: --hash +takes_value "hash method to use [default: blake2]")
        )
        (@subcommand genkey =>
            (about: "generates a new key pair")
            (@arg FILE: +takes_value "the destination file for the keypair")
        )
        (@subcommand addkey =>
            (about: "adds a key to the respository")
            (@arg REPO: +required "path of the repository")
            (@arg generate: --generate -g "generate a new key")
            (@arg set_default: --default -d "set this key as default")
            (@arg FILE: +takes_value "the file containing the keypair")
        )
        (@subcommand algotest =>
            (about: "test a specific algorithm combination")
            (@arg bundle_size: --bundlesize +takes_value "maximal bundle size in MiB [default: 25]")
            (@arg chunker: --chunker +takes_value "chunker algorithm [default: fastcdc/16]")
            (@arg compression: --compression -c +takes_value "compression to use [default: brotli/3]")
            (@arg encrypt: --encrypt -e "enable encryption")
            (@arg hash: --hash +takes_value "hash method to use [default: blake2]")
            (@arg FILE: +required "the file to test the algorithms with")
        )
    ).get_matches();
    if let Some(args) = args.subcommand_matches("init") {
        let (repository, backup, inode) = split_repo_path(args.value_of("REPO").unwrap());
        if backup.is_some() || inode.is_some() {
            println!("No backups or subpaths may be given here");
            exit(1);
        }
        return Arguments::Init {
            bundle_size: (parse_num(args.value_of("bundle_size").unwrap_or(&DEFAULT_BUNDLE_SIZE.to_string()), "Bundle size") * 1024 * 1024) as usize,
            chunker: parse_chunker(args.value_of("chunker").unwrap_or(DEFAULT_CHUNKER)),
            compression: parse_compression(args.value_of("compression").unwrap_or(DEFAULT_COMPRESSION)),
            encryption: args.is_present("encryption"),
            hash: parse_hash(args.value_of("hash").unwrap_or(DEFAULT_HASH)),
            repo_path: repository.to_string(),
            remote_path: args.value_of("remote").unwrap().to_string()
        }
    }
    if let Some(args) = args.subcommand_matches("backup") {
        let (repository, backup, inode) = split_repo_path(args.value_of("BACKUP").unwrap());
        if backup.is_none() {
            println!("A backup must be specified");
            exit(1);
        }
        if inode.is_some() {
            println!("No subpaths may be given here");
            exit(1);
        }
        return Arguments::Backup {
            repo_path: repository.to_string(),
            backup_name: backup.unwrap().to_string(),
            full: args.is_present("full"),
            same_device: args.is_present("same_device"),
            excludes: args.values_of("exclude").map(|v| v.map(|k| k.to_string()).collect()).unwrap_or_else(|| vec![]),
            excludes_from: args.value_of("excludes_from").map(|v| v.to_string()),
            src_path: args.value_of("SRC").unwrap().to_string(),
            reference: args.value_of("reference").map(|v| v.to_string())
        }
    }
    if let Some(args) = args.subcommand_matches("restore") {
        let (repository, backup, inode) = split_repo_path(args.value_of("BACKUP").unwrap());
        if backup.is_none() {
            println!("A backup must be specified");
            exit(1);
        }
        return Arguments::Restore {
            repo_path: repository.to_string(),
            backup_name: backup.unwrap().to_string(),
            inode: inode.map(|v| v.to_string()),
            dst_path: args.value_of("DST").unwrap().to_string()
        }
    }
    if let Some(args) = args.subcommand_matches("remove") {
        let (repository, backup, inode) = split_repo_path(args.value_of("BACKUP").unwrap());
        if backup.is_none() {
            println!("A backup must be specified");
            exit(1);
        }
        return Arguments::Remove {
            repo_path: repository.to_string(),
            backup_name: backup.unwrap().to_string(),
            inode: inode.map(|v| v.to_string())
        }
    }
    if let Some(args) = args.subcommand_matches("prune") {
        let (repository, backup, inode) = split_repo_path(args.value_of("REPO").unwrap());
        if backup.is_some() || inode.is_some() {
            println!("No backups or subpaths may be given here");
            exit(1);
        }
        return Arguments::Prune {
            repo_path: repository.to_string(),
            prefix: args.value_of("prefix").unwrap_or("").to_string(),
            force: args.is_present("force"),
            daily: args.value_of("daily").map(|v| parse_num(v, "daily backups") as usize),
            weekly: args.value_of("weekly").map(|v| parse_num(v, "weekly backups") as usize),
            monthly: args.value_of("monthly").map(|v| parse_num(v, "monthly backups") as usize),
            yearly: args.value_of("yearly").map(|v| parse_num(v, "yearly backups") as usize),
        }
    }
    if let Some(args) = args.subcommand_matches("vacuum") {
        let (repository, backup, inode) = split_repo_path(args.value_of("REPO").unwrap());
        if backup.is_some() || inode.is_some() {
            println!("No backups or subpaths may be given here");
            exit(1);
        }
        return Arguments::Vacuum {
            repo_path: repository.to_string(),
            force: args.is_present("force"),
            ratio: parse_num(args.value_of("ratio").unwrap_or(&DEFAULT_VACUUM_RATIO.to_string()), "ratio") as f32 / 100.0
        }
    }
    if let Some(args) = args.subcommand_matches("check") {
        let (repository, backup, inode) = split_repo_path(args.value_of("PATH").unwrap());
        return Arguments::Check {
            repo_path: repository.to_string(),
            backup_name: backup.map(|v| v.to_string()),
            inode: inode.map(|v| v.to_string()),
            full: args.is_present("full")
        }
    }
    if let Some(args) = args.subcommand_matches("list") {
        let (repository, backup, inode) = split_repo_path(args.value_of("PATH").unwrap());
        return Arguments::List {
            repo_path: repository.to_string(),
            backup_name: backup.map(|v| v.to_string()),
            inode: inode.map(|v| v.to_string())
        }
    }
    if let Some(args) = args.subcommand_matches("bundlelist") {
        let (repository, backup, inode) = split_repo_path(args.value_of("REPO").unwrap());
        if backup.is_some() || inode.is_some() {
            println!("No backups or subpaths may be given here");
            exit(1);
        }
        return Arguments::BundleList {
            repo_path: repository.to_string(),
        }
    }
    if let Some(args) = args.subcommand_matches("bundleinfo") {
        let (repository, backup, inode) = split_repo_path(args.value_of("REPO").unwrap());
        if backup.is_some() || inode.is_some() {
            println!("No backups or subpaths may be given here");
            exit(1);
        }
        return Arguments::BundleInfo {
            repo_path: repository.to_string(),
            bundle_id: parse_bundle_id(args.value_of("BUNDLE").unwrap())
        }
    }
    if let Some(args) = args.subcommand_matches("info") {
        let (repository, backup, inode) = split_repo_path(args.value_of("PATH").unwrap());
        return Arguments::Info {
            repo_path: repository.to_string(),
            backup_name: backup.map(|v| v.to_string()),
            inode: inode.map(|v| v.to_string())
        }
    }
    if let Some(args) = args.subcommand_matches("analyze") {
        let (repository, backup, inode) = split_repo_path(args.value_of("REPO").unwrap());
        if backup.is_some() || inode.is_some() {
            println!("No backups or subpaths may be given here");
            exit(1);
        }
        return Arguments::Analyze {
            repo_path: repository.to_string()
        }
    }
    if let Some(args) = args.subcommand_matches("import") {
        let (repository, backup, inode) = split_repo_path(args.value_of("REPO").unwrap());
        if backup.is_some() || inode.is_some() {
            println!("No backups or subpaths may be given here");
            exit(1);
        }
        return Arguments::Import {
            repo_path: repository.to_string(),
            remote_path: args.value_of("REMOTE").unwrap().to_string(),
            key_files: args.values_of("key").map(|v| v.map(|k| k.to_string()).collect()).unwrap_or_else(|| vec![])
        }
    }
    if let Some(args) = args.subcommand_matches("configure") {
        let (repository, backup, inode) = split_repo_path(args.value_of("REPO").unwrap());
        if backup.is_some() || inode.is_some() {
            println!("No backups or subpaths may be given here");
            exit(1);
        }
        return Arguments::Configure {
            bundle_size: args.value_of("bundle_size").map(|v| (parse_num(v, "Bundle size") * 1024 * 1024) as usize),
            chunker: args.value_of("chunker").map(|v| parse_chunker(v)),
            compression: args.value_of("compression").map(|v| parse_compression(v)),
            encryption: args.value_of("encryption").map(|v| {
                if v == "none" {
                    None
                } else {
                    Some(parse_public_key(v))
                }
            }),
            hash: args.value_of("hash").map(|v| parse_hash(v)),
            repo_path: repository.to_string(),
        }
    }
    if let Some(args) = args.subcommand_matches("genkey") {
        return Arguments::GenKey {
            file: args.value_of("FILE").map(|v| v.to_string())
        }
    }
    if let Some(args) = args.subcommand_matches("addkey") {
        let (repository, backup, inode) = split_repo_path(args.value_of("REPO").unwrap());
        if backup.is_some() || inode.is_some() {
            println!("No backups or subpaths may be given here");
            exit(1);
        }
        let generate = args.is_present("generate");
        if !generate && !args.is_present("FILE") {
            println!("Without --generate, a file containing the key pair must be given");
            exit(1);
        }
        if generate && args.is_present("FILE") {
            println!("With --generate, no file may be given");
            exit(1);
        }
        return Arguments::AddKey {
            repo_path: repository.to_string(),
            set_default: args.is_present("set_default"),
            file: args.value_of("FILE").map(|v| v.to_string())
        }
    }
    if let Some(args) = args.subcommand_matches("algotest") {
        return Arguments::AlgoTest {
            bundle_size: (parse_num(args.value_of("bundle_size").unwrap_or(&DEFAULT_BUNDLE_SIZE.to_string()), "Bundle size") * 1024 * 1024) as usize,
            chunker: parse_chunker(args.value_of("chunker").unwrap_or(DEFAULT_CHUNKER)),
            compression: parse_compression(args.value_of("compression").unwrap_or(DEFAULT_COMPRESSION)),
            encrypt: args.is_present("encrypt"),
            hash: parse_hash(args.value_of("hash").unwrap_or(DEFAULT_HASH)),
            file: args.value_of("FILE").unwrap().to_string(),
        }
    }
    error!("No subcommand given");
    exit(1);
}
