extern crate ftp;

use std::str;
use std::io::Cursor;
use ftp::FtpStream;

use std::io::prelude::*;
use std::fs::File;

fn main() {
    let mut ftp_stream = match FtpStream::connect("ftp2.bom.gov.au:21"){
    	Ok(s) => s,
	Err(e) => panic!("{}", e)
    };

    match ftp_stream.login("anonymous", "guest") {
    	Ok(_) => (),
	Err(e) => panic!("{}", e)
    };

    match ftp_stream.pwd(){
    	Ok(dir) => println!("{}", dir),
	Err(e) => panic!("{}", e)
    };

    match ftp_stream.cwd("anon/gen"){
    	Ok(_) => (),
	Err(e) => panic!("{}", e)
    };
    	

    match ftp_stream.pwd(){
    	Ok(dir) => println!("{}", dir),
	Err(e) => panic!("{}", e)
    };

    let remote_file = match ftp_stream.simple_retr("README"){
    	Ok(file) => file,
	Err(e) => panic!("{}", e)
    };


    let mut file = File::create("test.txt").ok().unwrap();

    file.write_all(remote_file.into_inner().as_slice());

    let _ = ftp_stream.quit();
}
