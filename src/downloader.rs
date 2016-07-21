extern crate ftp;

use std;
use std::str;
use std::io::prelude::*;
use std::fs::File;
use std::string::String;
use std::thread::sleep;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use ftp::FtpStream;

const VERBOSE: bool = true;

const DOWNLOAD_FOLDER: &'static str = "img/";
const LOCATION_CODE: &'static str = "IDR043";

// TODO:
// Need to use this in order to delete older files.
// To do this, will need to figure out a way to list the directory again.
// Should be easier because we aren't doing comparisons
// const IMAGES_KEPT: usize = 10;

// Connect to the BOM ftp server, get the radar files and save them as file_name locally.
// Returns whether or not any files were downloaded.
pub fn save_files() -> bool {

    let mut downloads = false;

    // Connect to the server
    let mut ftp_stream = match FtpStream::connect("ftp2.bom.gov.au:21"){
    	Ok(s) => s,
	Err(e) => {println!("Failed to connect to server: {}", e); return false;}
    };
    
    // Login anonymously
    match ftp_stream.login("anonymous", "guest") {
    	Ok(_) => (),
	Err(e) => {println!("Failed to log in: {}", e); return false;}
    };
    
    // Change to the required directory
    match ftp_stream.cwd("anon/gen/radar"){
    	Ok(_) => (),
	Err(e) => {println!("Failed to navigate to directory: {}", e); return false;}
    };

    // Find out which files are currently on the server
    let mut filenames = match ftp_stream.nlst(Option::None){
        Ok(v) => v,
	Err(e) => {println!("Failed to get file list: {}", e); return false;}
    };

    // Retain only the correct files (right prefix and right filetype)
    filenames.retain(correct_code_filter);

    for file_name in filenames{

    	    // Check if the file already exists locally.
	    // Open will return an error if it does not exist, so err = good.
	    match File::open(DOWNLOAD_FOLDER.to_string() + &file_name){
                Ok(_) => continue,
		Err(_) => println!("Choosing to download '{}'", file_name)
	    };

	    // Get the file from the server
	    let remote_file = match ftp_stream.simple_retr(&file_name){
		Ok(file) => file,
		Err(e) => {println!("Failed to get file: {}", e); return false;}
	    };

	    // Create a new file locally (overwriting if already exists)
	    let mut file = File::create(DOWNLOAD_FOLDER.to_string() + &file_name).ok().unwrap();

	    // Write the file
	    file.write_all(remote_file.into_inner().as_slice()).unwrap();

	    downloads = true;
    }

    // Disconnect from the server
    let _ = ftp_stream.quit();

    downloads
}

fn correct_code_filter(name: &String) -> bool {
    name.contains(LOCATION_CODE) && !name.contains(".gif")
}

pub fn wait_mins(mins: usize, terminate: &Arc<Mutex<bool>>) -> bool{
    let mut secs = mins * 60;
    
    let one_sec = Duration::new(1, 0);

    while secs > 0 {
        if VERBOSE {
	    print!("\rWaiting {} seconds...", secs);
	    std::io::stdout().flush().unwrap();
	}
        
        sleep(one_sec);

	let terminate = terminate.lock().unwrap();
	if *terminate {
	    if VERBOSE { println!("")};
	    return true;
	}

	secs -= 1;
    }
    if VERBOSE { println!("")};
    false
}

// Save the radar background if it is not already present
pub fn init() {

    let background_file_name = &(LOCATION_CODE.to_string() + ".background.png");
    let location_file_name = &(LOCATION_CODE.to_string() + ".locations.png");
    // Check if the files are present
    let mut missing = false;

    match File::open(background_file_name){
        Ok(_) => println!("Background file already exists: {}", background_file_name), 
	Err(_) => missing = true
    };
    
    match File::open(location_file_name){
        Ok(_) => println!("Locations file already exists"),
	Err(_) => missing = true
    };

    // Redownload if missing
    if missing{
        // Connect to the server
        let mut ftp_stream = match FtpStream::connect("ftp2.bom.gov.au:21"){
    	    Ok(s) => s,
	    Err(e) => {println!("Failed to connect to server: {}", e); return;}
        };
    
        // Login anonymously
        match ftp_stream.login("anonymous", "guest") {
    	    Ok(_) => (),
	    Err(e) => {println!("Failed to log in: {}", e); return;}
        };
    
        // Change to the required directory
        match ftp_stream.cwd("anon/gen/radar_transparencies"){
    	    Ok(_) => (),
	    Err(e) => {println!("Failed to navigate to directory: {}", e); return;}
        };

	// Get the files from the server
	let background_file = match ftp_stream.simple_retr(&background_file_name){
	    Ok(file) => file,
	    Err(e) => {println!("Failed to get file: {}", e); return;}
	};

	let location_file = match ftp_stream.simple_retr(&location_file_name){
	    Ok(file) => file,
	    Err(e) => {println!("Failed to get file: {}", e); return;}
	};

	// Create a new file locally (overwriting if already exists)
	let mut bg_file = File::create(background_file_name).ok().unwrap();
	let mut lc_file = File::create(location_file_name).ok().unwrap();
	
	// Write the files
	bg_file.write_all(background_file.into_inner().as_slice()).unwrap();
	lc_file.write_all(location_file.into_inner().as_slice()).unwrap();

        // Disconnect from the server
        let _ = ftp_stream.quit();
    }
        
}
