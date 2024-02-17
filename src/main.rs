use clap::{Parser, Subcommand};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::fs::{create_dir_all, remove_file, rename, File, OpenOptions};
use std::io::{stdout, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStringExt;
use std::path::Path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command()]
    Add,
    #[command()]
    List,
}

fn main() {
    let config = dirs::config_dir().unwrap();
    create_dir_all(config.join("cd_history")).unwrap();
    let db_rev = config.join("cd_history/log_rev");
    let c = Cli::parse();
    let mut s = env::current_dir().unwrap().into_os_string().into_vec();
    s.push(b'\n');
    match c.command {
        Commands::Add => {
            let mut f_rev = OpenOptions::new()
                .append(true)
                .create(true)
                .open(db_rev)
                .unwrap();
            f_rev.write_all(&s).unwrap();
        }
        Commands::List => {
            let db_next = config.join("cd_history/log_next");
            let db = config.join("cd_history/log");
            let mut paths = HashMap::with_capacity(100);
            let mut f_next = File::create(&db_next).unwrap();
            let mut out = stdout();
            if let Ok(f_rev) = File::open(&db_rev) {
                let mut f_rev = BufReader::new(f_rev);
                let mut f_rev_lines = Vec::with_capacity(10);
                loop {
                    let mut l = Vec::with_capacity(80);
                    let n = f_rev.read_until(b'\n', &mut l).unwrap();
                    if n == 0 {
                        break;
                    }
                    l.pop();
                    let l_s = OsString::from_vec(l);
                    let exists = Path::new(&l_s).is_dir();
                    l = l_s.into_vec();
                    l.push(b'\n');
                    if exists {
                        f_rev_lines.push(l);
                    }
                }
                for l in f_rev_lines.into_iter().rev() {
                    if let Entry::Vacant(e) = paths.entry(l) {
                        f_next.write_all(e.key()).unwrap();
                        out.write_all(e.key()).unwrap();
                        e.insert(());
                    }
                }
                remove_file(db_rev).unwrap();
            }
            if let Ok(f) = File::open(&db) {
                let mut f = BufReader::new(f);
                loop {
                    let mut l = Vec::with_capacity(80);
                    let n = f.read_until(b'\n', &mut l).unwrap();
                    if n == 0 {
                        break;
                    }
                    l.pop();
                    let l_s = OsString::from_vec(l);
                    let exists = Path::new(&l_s).is_dir();
                    l = l_s.into_vec();
                    l.push(b'\n');
                    if exists {
                        if let Entry::Vacant(e) = paths.entry(l) {
                            f_next.write_all(e.key()).unwrap();
                            out.write_all(e.key()).unwrap();
                            e.insert(());
                        }
                    }
                }
            }
            rename(db_next, db).unwrap();
        }
    }
}
