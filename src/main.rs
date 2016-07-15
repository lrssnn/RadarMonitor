extern crate ftp;

use std::str;
use ftp::FtpStream;

use std::io::prelude::*;
use std::fs::File;

use std::thread::sleep;
use std::time::Duration;

// Connect to the BOM ftp server, get the radar files and save them as file_name locally.
// Returns whether or not any files were downloaded.
fn save_files() -> bool {

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
	    match File::open(&file_name){
                Ok(_) => continue,
		Err(_) => println!("Choosing to download '{}'", file_name)
	    };

	    // Get the file from the server
	    let remote_file = match ftp_stream.simple_retr(&file_name){
		Ok(file) => file,
		Err(e) => {println!("Failed to get file: {}", e); return false;}
	    };

	    // Create a new file locally (overwriting if already exists)
	    let mut file = File::create(&file_name).ok().unwrap();

	    // Write the file
	    file.write_all(remote_file.into_inner().as_slice());

	    downloads = true;
    }

    // Disconnect from the server
    let _ = ftp_stream.quit();

    downloads
}

fn correct_code_filter(name: &String) -> bool {
    name.contains("IDR043") && !name.contains(".gif")
}

fn main() {

   loop {
	while !save_files(){
	    println!("No new files");
	    wait_mins(1, true);
	}
        wait_mins(3, true);
    }
}

fn wait_mins(mut mins: u8, verbose: bool){
    let ten_sec = Duration::new(10, 0);
    loop {
        if verbose {
            print!("{}", mins);
	    std::io::stdout().flush();
	}
	for i in 0..6 {
	    sleep(ten_sec);
	    if verbose {
	        print!(".");
	        std::io::stdout().flush();
	    }
	}
	mins -= 1;
	if mins == 0 {
            if verbose {println!("0")};
	    return;
	}

    }	
}
