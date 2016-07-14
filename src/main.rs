extern crate ftp;

use std::str;
use std::io::Cursor;
use ftp::FtpStream;

use std::io::prelude::*;
use std::fs::File;

// Connect to the BOM ftp server, get the file at /file_dir/file_name and save it as file_name locally.
// file_dir can be a nested directory dira/dirb
fn save_file(file_dir: &str, file_name: &str) {

    // Connect to the server
    let mut ftp_stream = match FtpStream::connect("ftp2.bom.gov.au:21"){
    	Ok(s) => s,
	Err(e) => panic!("{}", e)
    };
    
    // Login anonymously
    match ftp_stream.login("anonymous", "guest") {
    	Ok(_) => (),
	Err(e) => panic!("{}", e)
    };
    
    // Change to the required directory
    match ftp_stream.cwd(file_dir){
    	Ok(_) => (),
	Err(e) => panic!("{}", e)
    };
    	
    // Get the file from the server
    let remote_file = match ftp_stream.simple_retr(file_name){
    	Ok(file) => file,
	Err(e) => panic!("{}", e)
    };

    // Create a new file locally (overwriting if already exists)
    let mut file = File::create(file_name).ok().unwrap();

    // Write the file
    file.write_all(remote_file.into_inner().as_slice());

    // Disconnect from the server
    let _ = ftp_stream.quit();
}

fn main() {
    save_file("anon/gen/radar", "IDR043.gif");
}
