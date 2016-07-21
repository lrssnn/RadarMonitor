extern crate ftp;
extern crate sfml;
extern crate time;

use std::thread;
use std::sync::{Arc, Mutex};

mod image_viewer;
use image_viewer::open_window;

mod downloader;
use downloader::{save_files, init, wait_mins};

// Main function.
fn main() {

    init();
    save_files();

    // Create a boolean variable which we will send to the child thread when it is time to regenerate the texture list
    // and one which tells the main thread when the child thread has been closed
    let finish = Arc::new(Mutex::new(false));
    let update = Arc::new(Mutex::new(false));

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
        let mut update = update.lock().unwrap();
	*update = true;
    }
}
