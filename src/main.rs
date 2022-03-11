use std::fs;
use std::fs::{Metadata, DirEntry};
use std::io;
use bytesize::ByteSize;
use guard::guard;

mod cleaner;

use cleaner::*;

fn main() -> io::Result<()> {
    let dry_run = true;
    let mut sum = 0;
    let cleaners: Vec<Box<dyn Cleaner>> = vec!(Box::new(RustCleaner {}), Box::new(NodeCleaner {}));
    let path = dirs::home_dir().unwrap().join("Developer");

    println!("Working on {:?}", path);
    
    for entry in fs::read_dir(path)? {
        // Skip whatever we don't have permission to access.
        guard!(let Ok(entry) = entry else { continue });
         // If we can't access metadata, skip it.
        guard!(let Ok(metadata) = fs::metadata(&entry.path()) else { continue });

        if is_candidate(&entry, &metadata) {
            for cleaner in &cleaners {
                // Provide a report on what space can be saved.
                let cleanable = cleaner.cleanable(&entry);
                match  cleanable {
                    Some(size) => {
                        println!("Found a {} project in {:?}, can save {:?}", cleaner.name(), entry.path(), ByteSize::b(size));
                        sum += size;
                    },
                    None => ()
                }

                if cleanable.is_some() {
                    if !dry_run {
                        cleaner.clean(&entry)?;
                    }
                }
            }
        }
    }

    if dry_run {
        println!("Total space to be saved: {:?}", ByteSize::b(sum));
    } else {
        println!("Total space saved: {:?}", ByteSize::b(sum));
    }
    

    Ok(())
}

fn is_candidate(entry: &DirEntry, metadata: &Metadata) -> bool {
    // If we can't get the modified date, we're likely not supposed to be here.
    guard!(let Ok(modified) = metadata.modified() else { return false } );
    guard!(let Ok(elapsed) = modified.elapsed() else { return false});

    //                 m    h    d
    let max_secs = 60 * 60 * 24 * 30;

    metadata.is_dir() && elapsed.as_secs() > max_secs
}


