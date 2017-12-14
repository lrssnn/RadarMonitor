#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_contrib;

extern crate ftp;

#[macro_use] extern crate serde_derive;

use std::thread;
use std::sync::mpsc;
use std::env;

use std::fs;
use std::fs::File;
use std::path::PathBuf;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

mod downloader;
use downloader::{save_files, init, wait_mins};

use rocket::http::RawStr;

// Configuration constants
const DL_DIR:    &'static str = "img/"; // Folder to keep images in.
const CODE_LOW:  &'static str = "IDR042";
const CODE_MID:  &'static str = "IDR043"; // BOM product code for the desired radar image set
const CODE_HIGH: &'static str = "IDR044";

// Milliseconds per frame
const SPEED_SLOW: usize = 200;
const SPEED_MID:  usize = 100;
const SPEED_FAST: usize = 60;

use rocket_contrib::Json;

#[derive(Serialize)]
struct IndexTemplate {
    message: &'static str
}

#[get("/")]
fn index() -> Result<Json<Vec<Vec<PathBuf>>>, usize> {
    // Return an array of the available files on the server
    
    let dirs;
    // If DL_DIR doesn't exist, there is nothing to clean
    match fs::read_dir(DL_DIR) {
        Ok(d)  => { dirs = d }
        Err(_) => { return Err(500) }
    };

    let mut zooms = vec![];

    // Iterate through the subdirectories (zoom levels)
    for dir in dirs {
        
        let mut zoom_array = vec![];

        let files = fs::read_dir(dir.expect("File Error").path()).expect("File Error");

        // Get the path for each file
        let mut file_names: Vec<_> = files.map(|e| {
            e.expect("File error")
                .path()
        })
        .collect();
        file_names.sort();

        // Iterate through the files
        for filename in file_names.iter() {
            zoom_array.push(filename.clone());
        }

        zooms.push(zoom_array);
    }

    Ok(Json(zooms))
}

#[get("/img/<dl_dir>/<file_name>")]
fn image(dl_dir: String, file_name: String) -> Result<File, usize> {

    match File::open(format!("img/{}/{}", dl_dir, file_name)) {
        Ok(f)  => { Ok(f) }
        Err(_) => { Err(404) }
    }
}

// Main function.
fn main() {

   
    let mut clean = false;

    println!("Radar Monitor:");
    // Check the program args
    for arg in env::args().skip(1) {
        if arg == "--clean"  || arg == "-c" {
            clean = true;
        } else {
            println!("Unknown argument: {}", arg);
            return;
        }
    }

    init();

    if clean {
        println!("Cleaning images directory");
        downloader::clean();
    }

    // Create our channels
    let (update_tx, update_rx) = mpsc::channel();
    let (finish_tx, finish_rx) = mpsc::channel();

    // Spawn the webserver thread
    thread::spawn(move || {
        rocket::ignite()
            .mount("/", routes![index, image])
            .launch();
    });


    loop {
        // Wait for 5 minutes, then check the server every minute until we get at least
        // 1 new file
        if wait_mins(5, &finish_rx) {
            return;
        }

        while !save_files().is_ok() {
            if wait_mins(1, &finish_rx) {
                return;
            }
        }

        // Tell the other thread to update the image display
        update_tx.send(()).unwrap();
    }
}
