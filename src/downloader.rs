extern crate ftp;

use std;
use std::str;
use std::str::FromStr;
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::thread::sleep;
use std::time::Duration;
use ftp::FtpStream;

use super::DL_DIR;
use super::CODE_MID;
use super::CODE_LOW;
use super::CODE_HIGH;

//Simple timecode struct used to determine file contiguousness
#[derive(Debug)]
struct Timecode {
    year: usize,
    month: usize,
    day: usize,
    hour: usize,
    min: usize
}

pub fn run_loop() -> Result<(), ()> {
    loop {
        // Wait for 5 minutes, then check the server every minute until we get at least
        // 1 new file
        wait_mins(5);

        while save_files().is_err() {
            wait_mins(1)
        }
    }
}

// Connect to the BOM ftp server and download any new files 
// Saves files for all three zoom levels
// Files are saved to the folder DL_DIR/lc_code and are prefixed with an 'x' to designate them as
// new.
// Returns Ok(()) if everything was ok. Propogates an error if there is an ftp error
pub fn save_files() -> ftp::types::Result<()> {


    // Connect to the server, login, change directory
    let mut ftp_stream = FtpStream::connect("ftp2.bom.gov.au:21")?;
    ftp_stream.login("anonymous", "guest")?;
    ftp_stream.cwd("anon/gen/radar")?;

    // Find out which files are currently on the server
    let filenames = ftp_stream.nlst(Option::None)?;

    for lc_code in &[CODE_LOW, CODE_MID, CODE_HIGH]{

        let mut downloads = 0;
        let mut filenames = filenames.clone();

        // Retain only the correct files (right prefix and right filetype)
        filenames.retain(|e| e.contains(lc_code) && !e.contains(".gif"));

        for file_name in filenames {
            // Prefix filename to designate it as a new file
            let file_name_x = "x".to_string() + &file_name;

            // Check if the file already exists locally.
            // Open will return an error if it does not exist, so err = good.
            // If open succeeds, the file exists and we move to the next file
            if File::open(DL_DIR.to_string() + lc_code + "/" + &file_name).is_ok() {
                continue;
            }
            
            // Print a message (one line only regardless of number of files)
            print!("\r({:02}) Choosing to download '{}'", downloads + 1, file_name);
            std::io::stdout().flush().expect("Error flushing stdout");

            // Get the file from the server
            let remote_file = ftp_stream.simple_retr(&file_name)?;

            // Create a new file locally
            let mut file = File::create(DL_DIR.to_string() + lc_code + "/" + &file_name_x)
                .expect("Error creating file on disk");

            // Write the file
            file.write_all(remote_file.into_inner().as_slice())
                .expect("Error writing file to disk");

            downloads += 1;
        }

        if downloads > 0 {
            println!();
        }
    }

    // Disconnect from the server
    let _ = ftp_stream.quit();
    Ok(())
}

// Wait for 'mins' minutes while printing a report of how long remains.
pub fn wait_mins(mins: usize) {
    let mut secs = mins * 60;

    let one_sec = Duration::new(1, 0);

    while secs > 0 {
        print!("\rWaiting {} seconds...     ", secs);
        std::io::stdout().flush().expect("Error flushing stdout");

        sleep(one_sec);

        secs -= 1;
    }

    if mins == 1 {
        println!("\rWaited 1 minute.       ");
    } else {
        println!("\rWaited {} minutes.      ", mins);
    }
}

// Run first time initialisation tasks such as creating directories and priming with images
// Will panic if initialisation fails
pub fn init(){
    // Attempt to create the download directory, not caring if it succeeds or if it fails 
    // (the directory already exists)
    match fs::create_dir(DL_DIR) {
        Ok(_) | Err(_) => (),
    };

    init_background(CODE_LOW).expect("Initialisation Failure");
    init_background(CODE_MID).expect("Initialisation Failure");
    init_background(CODE_HIGH).expect("Initialisation Failure");

    // Any pre-existing files will not be prefixed, and will not be re-downloaded by
    // save_files(), once that has completed we re-prefix the pre-existing files to be made
    // into textures by the other thread
    save_files().ok();

    mark_files_as_new(CODE_LOW);
    mark_files_as_new(CODE_MID);
    mark_files_as_new(CODE_HIGH);
}

// Save the radar background for location code 'lc_code' and create the subdirectory for 
// that radar's images
// Propogates any FTP errors upstream, panics on file system errors
pub fn init_background(lc_code: &str) -> ftp::types::Result<()> {

    // Do nothing on an error. Generally an error here means that the directory 
    // already exists which is what we want
    match fs::create_dir(DL_DIR.to_string() + lc_code + "/") {
        Ok(_) | Err(_) => (),
    };

    let background_file_name = &(lc_code.to_string() + ".background.png");
    let location_file_name = &(lc_code.to_string() + ".locations.png");

    // Connect to the server, login, navigate to directory
    let mut ftp_stream = FtpStream::connect("ftp2.bom.gov.au:21")?;
    ftp_stream.login("anonymous", "guest")?;
    ftp_stream.cwd("anon/gen/radar_transparencies")?;

    // Get the files from the server
    let background_file = ftp_stream.simple_retr(background_file_name)?;
    let location_file = ftp_stream.simple_retr(location_file_name)?;

    // Create a new file locally (overwriting if already exists)
    let mut bg_file = File::create(background_file_name).expect("Error creating file on disk");
    let mut lc_file = File::create(location_file_name).expect("Error creating file on disk");

    // Write the files
    bg_file.write_all(background_file.into_inner().as_slice())
        .expect("Error writing file to disk");
    lc_file.write_all(location_file.into_inner().as_slice())
        .expect("Error writing file to disk");
    // Disconnect from the server
    let _ = ftp_stream.quit();

    Ok(())
}

// For all files in the folder DL_DIR/location_code/ which do not have an 'x' prefix, add that
// prefix to their filename.
fn mark_files_as_new(location_code: &str) {
    // Read the directory and convert to filenames
    let dir = DL_DIR.to_string() + location_code + "/";
    let files: Vec<_> = fs::read_dir(&dir)
        .expect("Error reading directory")
        .map(|e| {
            e.expect("Error reading image filename")
                .file_name()
                .into_string()
                .expect("Error extracting image filename")
        })
        .collect();

    // Filter out any files which already have the prefix
    let mut file_names = files.iter().filter(|e| !e.starts_with('x')).collect::<Vec<_>>();

    file_names.sort();

    // Prefix each file
    for file_name in file_names {
        let new_name = "x".to_string() + file_name;
        fs::rename(&(dir.to_string() + file_name),
                   &(dir.to_string() + &new_name))
            .expect("Error renaming file");
    }
}

// Removes non-contiguous files from each image directory
// i.e. leaves only the most recent streak of contiguous images based on the 
// assumption that each image from the radar comes six minutes after the previous one
pub fn clean() {
    let file_error = "Error reading file system";

    let dirs;
    // If DL_DIR doesn't exist, there is nothing to clean
    match fs::read_dir(DL_DIR) {
        Ok(d)  => { dirs = d }
        Err(_) => { return }
    };

    // Iterate through the subdirectories (zoom levels)
    for dir in dirs {
        let files = fs::read_dir(dir.expect(file_error).path()).expect(file_error);

        // Get the path for each file
        let mut file_names: Vec<_> = files.map(|e| {
            e.expect(file_error)
                .path()
        })
        .collect();
        file_names.sort();

        let mut del = 0;
        // Iterate through the files in reverse order (newest to oldest)
        // If del is true we have already found a break in continuity and are just deleting
        for (i, prev) in file_names.iter().enumerate().rev(){
            if i+1 != file_names.len() {
                // Look ahead 1
                let file = &file_names[i+1];
                if del > 0 {
                    del += 1;
                    print!("\r({:02}) Deleting: {:?}", del, prev);
                    fs::remove_file(prev).expect(file_error);
                } else if !consecutive_files(prev, file) {
                    del += 1;
                    print!("\r({:02}) Deleting: {:?}", del, prev);
                    fs::remove_file(prev).expect(file_error);
                }
            }
        }
        if del > 0 {
            println!();
        }
    }
}

// Converts a file path to a Timecode object
// Assumes that the file given is a BOM radar image
fn timecode_from_path(name: &std::path::Path) -> Timecode{ 
    // Get the file name
    let name = name.file_name().expect("Error reading file name")
        .to_str().expect("String conversion error");

    // Split at each dot and take the third (just the timecode)
    let string = name.split('.').nth(2).expect("Unexpected file format");

    // Split the timecode into the respective parts
    let (year, string) = string.split_at(4);
    let (month, string) = string.split_at(2);
    let (day, string) = string.split_at(2);
    let (hour, string) = string.split_at(2);
    let (min, _) = string.split_at(2);

    // Convert into numbers and return
    Timecode{
        year:  usize::from_str(year).expect("Unexpected file format"),
        month: usize::from_str(month).expect("Unexpected file format"),
        day:   usize::from_str(day).expect("Unexpected file format"),
        hour:  usize::from_str(hour).expect("Unexpected file format"),
        min:   usize::from_str(min).expect("Unexpected file format"),
    }
}

// Returns true if the two files given are consecutive, in that the timecodes contained
// in the file names are 6 minutes apart
fn consecutive_files(prev: &std::path::Path, next: &std::path::Path) -> bool {
    // Convert filenames to timecodes
    let prev = timecode_from_path(prev);
    let next = timecode_from_path(next);

    // Chain through the different levels of time timecode
    // Not entirely sure that every level works but it at least works across 
    // Hour boundaries so for the most part is ok
    if prev.min + 6 == next.min ||
    (next.min <= 6 && prev.hour + 1 == next.hour) ||
    (next.hour == 0 && prev.day + 1 == next.day) ||
    (next.day == 0 && prev.month + 1 == next.month){
        true
    } else {
        next.month == 0 && prev.year + 1 == next.year
    }
}
