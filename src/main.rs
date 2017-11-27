#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate rocket;

extern crate ftp;

use std::thread;
use std::sync::mpsc;
use std::env;

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

#[get("/")]
fn index() -> &'static str {
    "Hello Sailor"
}

#[get("/<message>")]
fn message(message: &RawStr) -> String {
    format!("The message is: {}", message.as_str())
}

// Main function.
fn main() {

    println!("I am a webserber");
    rocket::ignite().mount("/", routes![index, message]).launch();
    println!("What happens now");

   
    /*
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
    */
}
