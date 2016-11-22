extern crate ftp;

use std;
use std::str;
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::thread::sleep;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use ftp::FtpStream;

use super::VERBOSE;
use super::DL_DIR;
use super::CODE_MID;
use super::CODE_LOW;
use super::CODE_HIGH;
use super::IMAGES_KEPT;

pub fn save_all_files() -> bool {
    let a = save_files(CODE_LOW);
    let b = save_files(CODE_MID);
    let c = save_files(CODE_HIGH);
    a || b || c
}
// Connect to the BOM ftp server, get the radar files and save them as file_name locally.
// Returns whether or not any files were downloaded.
pub fn save_files(lc_code: &str) -> bool {

    let mut downloads = 0;

    // Connect to the server
    let mut ftp_stream = match FtpStream::connect("ftp2.bom.gov.au:21") {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to connect to server: {}", e);
            return false;
        }
    };

    // Login anonymously
    match ftp_stream.login("anonymous", "guest") {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to log in: {}", e);
            return false;
        }
    };

    // Change to the required directory
    match ftp_stream.cwd("anon/gen/radar") {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to navigate to directory: {}", e);
            return false;
        }
    };

    // Find out which files are currently on the server
    let mut filenames = match ftp_stream.nlst(Option::None) {
        Ok(v) => v,
        Err(e) => {
            println!("Failed to get file list: {}", e);
            return false;
        }
    };

    // Retain only the correct files (right prefix and right filetype)
    filenames.retain(|e| e.contains(lc_code) && !e.contains(".gif"));

    for file_name in filenames {

        let file_name_x = "x".to_string() + &file_name;

        // Check if the file already exists locally.
        // Open will return an error if it does not exist, so err = good.
        match File::open(DL_DIR.to_string() + lc_code + "/" + &file_name) {
            Ok(_) => continue,
            Err(_) => {
                print!("\r({:02}) Choosing to download '{}'", downloads+1, file_name);
                std::io::stdout().flush().expect("Error flushing stdout");
            },
        };

        // Get the file from the server
        let remote_file = match ftp_stream.simple_retr(&file_name) {
            Ok(file) => file,
            Err(e) => {
                println!("\nFailed to get file: {}", e);
                return false;
            }
        };


        // Create a new file locally (overwriting if already exists)
        let mut file = File::create(DL_DIR.to_string() + lc_code + "/" + &file_name_x)
            .expect("Error creating file on disk");

        // Write the file
        file.write_all(remote_file.into_inner().as_slice())
            .expect("Error writing file to disk");

        downloads += 1;
    }

    // Disconnect from the server
    let _ = ftp_stream.quit();
    if downloads > 0{
        println!("");
    }

    downloads > 0
}

pub fn wait_mins(mins: usize, terminate: &Arc<AtomicBool>) -> bool {
    let mut secs = mins * 60;

    let one_sec = Duration::new(1, 0);

    while secs > 0 {
        if VERBOSE {
            print!("\rWaiting {} seconds...     ", secs);
            std::io::stdout().flush().expect("Error flushing stdout");
        }

        sleep(one_sec);

        let terminate = terminate.load(Ordering::Relaxed);
        if terminate {
            if VERBOSE {
                println!("")
            };
            return true;
        }

        secs -= 1;
    }
    if VERBOSE {
        if mins == 1 {
            println!("\rWaited 1 minute.       ");
        } else {
            println!("\rWaited {} minutes.      ", mins);
        }
    }
    false
}

pub fn init() {
    match fs::create_dir(DL_DIR){
        Ok(_) => (),
        Err(_) => (),
    };

    init_background(CODE_LOW);
    init_background(CODE_MID);
    init_background(CODE_HIGH);
    
    save_all_files();

    mark_files_as_new(CODE_LOW);
    mark_files_as_new(CODE_MID);
    mark_files_as_new(CODE_HIGH);
}

// Save the radar background if it is not already present
pub fn init_background(lc_code: &str) {

    match fs::create_dir(DL_DIR.to_string() + lc_code + "/"){
        Ok(_) => (),
        Err(_) => (),
    };

    let background_file_name = &(lc_code.to_string() + ".background.png");
    let location_file_name = &(lc_code.to_string() + ".locations.png");

    // Connect to the server
    let mut ftp_stream = match FtpStream::connect("ftp2.bom.gov.au:21") {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to connect to server: {}", e);
            return;
        }
    };

    // Login anonymously
    match ftp_stream.login("anonymous", "guest") {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to log in: {}", e);
            return;
        }
    };

    // Change to the required directory
    match ftp_stream.cwd("anon/gen/radar_transparencies") {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to navigate to directory: {}", e);
            return;
        }
    };

    // Get the files from the server
    let background_file = match ftp_stream.simple_retr(background_file_name) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to get file: {}", e);
            return;
        }
    };

    let location_file = match ftp_stream.simple_retr(location_file_name) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to get file: {}", e);
            return;
        }
    };

    // Create a new file locally (overwriting if already exists)
    let mut bg_file = File::create(background_file_name)
        .expect("Error creating file on disk");
    let mut lc_file = File::create(location_file_name)
        .expect("Error creating file on disk");

    // Write the files
    bg_file.write_all(background_file.into_inner().as_slice())
        .expect("Error writing file to disk");
    lc_file.write_all(location_file.into_inner().as_slice())
        .expect("Error writing file to disk");
    // Disconnect from the server
    let _ = ftp_stream.quit();
}

pub fn remove_old_files() {
    // If the number to keep is 0, means keep everything
    if IMAGES_KEPT == 0 {
        return;
    }

    // Get a list of every file in the image folder
    let files = fs::read_dir(DL_DIR).expect("Error reading file system");

    let mut file_names: Vec<_> = files.map(|e| {
            e.expect("Error reading file system")
                .file_name()
                .into_string()
                .expect("Error extracting file name")
        })
        .collect();
    file_names.sort();

    println!("Images in img/: {}", file_names.len());

    let mut file_names = file_names.iter();

    while file_names.len() > IMAGES_KEPT {
        // Delete a file
        let name = file_names.next().expect("Error finding first filename");
        println!("List length: {}, target: {} Removing: {}",
                 file_names.len(),
                 IMAGES_KEPT,
                 name);
        fs::remove_file(DL_DIR.to_string() + name).expect("Error deleting file");
    }
}

fn mark_files_as_new(location_code: &str){
    let dir = DL_DIR.to_string() + location_code + "/";
    let mut files = fs::read_dir(&dir).unwrap();

    let file_names: Vec<_> = files.map(|e| {
           e.expect("Error reading image filename")
               .file_name()
               .into_string()
               .expect("Error extracting image filename")
       })
       .collect();

    let mut file_names = file_names.iter().filter(|e| !e.starts_with('x')).collect::<Vec<_>>();

    file_names.sort();

    for file_name in file_names {
        let new_name = "x".to_string() + file_name;
        fs::rename(&(dir.to_string() + file_name),
                   &(dir.to_string() + &new_name))
            .expect("Error renaming file");
    }
}
