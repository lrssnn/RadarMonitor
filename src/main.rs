#![allow(missing_docs_in_private_items)]

extern crate ftp;
extern crate time;
#[macro_use]
extern crate glium;

use std::thread;
use std::sync::{Arc};
use std::sync::atomic::{Ordering, AtomicBool};

mod image_viewer;
use image_viewer::open_window;

mod downloader;
use downloader::{save_files, init, wait_mins, remove_old_files};

// Configuration constants
const VERBOSE: bool = true;    // If false, program will not print to standard out
const IMAGES_KEPT: usize = 0; // Number of images to keep in the rotating set

const DOWNLOAD_FOLDER: &'static str = "img/"; // Folder to keep images in. Relative to start dir or absolute
const LOCATION_CODE: &'static str = "IDR043"; // BOM product code for the desired radar image set
// Milliseconds per frame
const SPEED_SLOW: usize = 300;
const SPEED_MID: usize  = 200;
const SPEED_FAST: usize = 100;

// Main function.
fn main() {

    init();
    save_files();

    // Create a boolean variable which we will send to the child thread when it is time to regenerate the texture list
    // and one which tells the main thread when the child thread has been closed
    let finish = Arc::new(AtomicBool::new(false));
    let update = Arc::new(AtomicBool::new(false));

    // Cloning an Arc actually just gives us a new reference. We move this reference into the other thread
    // but it points to the same boolean we have here.
    let finish_clone = finish.clone();
    let update_clone = update.clone();

    // Start the thread which displays the window
    thread::spawn(move || {
        open_window(&finish_clone, &update_clone);
    });
	
    loop {
        // Wait for 5 minutes, then check the server every minute until we get at least 1 new file
	if wait_mins(5, &finish) {
	    return;
	}
	while !save_files() {
	    if wait_mins(1, &finish) {
	        return;
	    }
	}

        // Lock the mutex, then tell the other thread to update the list
	// Mutex is released implicitly when this loop scope ends.
    update.store(true, Ordering::Relaxed);

	remove_old_files();
    }
}
