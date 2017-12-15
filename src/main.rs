#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_contrib;

extern crate ftp;

use std::thread;
use std::env;

use std::fs;
use std::fs::File;
use std::path::PathBuf;

use rocket_contrib::Json;

mod downloader;
use downloader::{save_files, init, wait_mins};

// Configuration constants
const DL_DIR:    &'static str = "img/"; // Folder to keep images in.
const CODE_LOW:  &'static str = "IDR042";
const CODE_MID:  &'static str = "IDR043"; // BOM product code for the desired radar image set
const CODE_HIGH: &'static str = "IDR044";

// Server index. Returns a directory listing in the form of nested arrays of strings.
#[get("/")]
fn index() -> Result<Json<Vec<Vec<PathBuf>>>, usize> {
    
    let dirs;
    // If DL_DIR doesn't exist, there is nothing to clean
    match fs::read_dir(DL_DIR) {
        Ok(d)  => { dirs = d }
        Err(_) => { return Err(500) }
    };

    // Will contain a Vec for each zoom level that exists
    let mut zooms = vec![];

    // Iterate through the subdirectories (zoom levels)
    for dir in dirs {
        
        // Will contain the filename of each file in this directory
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
    // Respond to the request with a Json serialisation of the array
    Ok(Json(zooms))
}

// Image request: respond with the file
#[get("/img/<dl_dir>/<file_name>")]
fn image(dl_dir: String, file_name: String) -> Result<File, usize> {
    // Simply open the file and return it if it exists
    match File::open(format!("img/{}/{}", dl_dir, file_name)) {
        Ok(f)  => { Ok(f) }
        Err(_) => { Err(404) }
    }
}


fn main() {
   
    let mut clean = false;

    // 30 images corresponds to 3 hours
    let mut image_keep_limit = 30;

    println!("Radar Monitor:");
    // Check the program args

    let mut used = false; // Indicator that the last argument was used
    let args: Vec<String> = env::args().skip(1).collect();
    for (i, arg) in args.iter().enumerate() {
        if used {
            used = false;
        } else if arg == "--clean"  || arg == "-c" {
            clean = true;
        } else if arg == "--limit" || arg == "-l" {
            used = true;
            if i + 1 == args.len() {
                println!("Missing argument to --limit");
                return;
            } else {
                let lim = &args[i + 1];
                let lim_n: usize = lim.parse().expect(&format!("Invalid argument: {}", lim));
                image_keep_limit = lim_n;
            }
        } else {
            println!("Unknown argument: {}", arg);
            return;
        }
    }

    println!("Save limit is {}", image_keep_limit);

    init();

    if clean {
        println!("Cleaning images directory");
        downloader::clean();
    }


    // Spawn the webserver thread
    thread::spawn(move || {
        rocket::ignite()
            .mount("/", routes![index, image])
            .launch();
    });


    loop {
        // Wait for 5 minutes, then check the server every minute until we get at least
        // 1 new file
        wait_mins(5);

        while !save_files().is_ok() {
            wait_mins(1);
        }

        downloader::trim_files(30);
    }
}
