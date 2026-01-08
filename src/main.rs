mod entry;
mod log;

use entry::Entry;
use log::Log;

fn main() {
    let mut log = Log::new();

    let e1 = Entry::new("Fawz", "Stellar Reunion");
    let e2 = Entry::new("Fawz", "Working on stationInvariant");

    let off1 = log.append(e1);
    let off2 = log.append(e2);

    println!("Inserted at offsets: {}, {}", off1, off2);
    println!("Log length: {}", log.len());

    println!("\nReplay from offset 0:");
    for entry in log.replay_from(0) {
        println!("{:?}", entry);
    }

    println!("\nReplay from offset 1:");
    for entry in log.replay_from(1) {
        println!("{:?}", entry);
    }
}

