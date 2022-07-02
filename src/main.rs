#[macro_use]
extern crate glium;
extern crate ftp;

use std::env;
use std::thread;

mod downloader;
mod image_viewer;

// Configuration constants
const DL_DIR: &str = "img/"; // Folder to keep images in.
const CODE_LOW: &str = "IDR042";
const CODE_MID: &str = "IDR043"; // BOM product code for the desired radar image set
const CODE_HIGH: &str = "IDR044";

// Milliseconds per frame
const SPEED_SLOW: usize = 200;
const SPEED_MID: usize = 100;
const SPEED_FAST: usize = 60;

// Main function.
fn main() {
    let mut clean = false;

    println!("Radar Monitor:");
    // Check the program args
    for arg in env::args().skip(1) {
        if arg == "--clean" || arg == "-c" {
            clean = true;
        } else {
            println!("Unknown argument: {}", arg);
            return;
        }
    }

    downloader::init();

    if clean {
        println!("Cleaning images directory");
        downloader::clean();
    }

    // Start the thread which downloads the files
    thread::spawn(move || {
        downloader::run_loop().expect("Downloading Error");
    });

    // Open the window. This has to happen on the main thread for reasons
    image_viewer::open_window().expect("Drawing Error");
}
