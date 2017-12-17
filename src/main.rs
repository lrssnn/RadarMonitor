#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_contrib;

extern crate ftp;

use std::thread;
use std::env;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::Read;

use std::time;

use rocket_contrib::Json;

mod downloader;
use downloader::{save_files, init, wait_mins};

// Configuration constants
const DL_DIR:    &str = "img/"; // Folder to keep images in.
const CODE_LOW:  &str = "IDR042";
const CODE_MID:  &str = "IDR043"; // BOM product code for the desired radar image set
const CODE_HIGH: &str = "IDR044";

// Index: Serve the html file which displays the images
#[get("/")]
fn index() -> File {
    File::open("res/index.html").expect("Index Missing")
}

// Resources: Provide the javascript and css required by the html file
#[get("/res/<file>")]
fn resource(file: String) -> File {
    File::open(format!("res/{}", file)).expect("Resource Missing")
}

// Backgrounds: Provide the unchanging background and location image layers
#[get("/img/<file>")]
fn backgrounds(file: String) -> File {
    File::open(format!("img/{}", file)).expect("Background Missing")
}

// Listing: Returns a directory listing in the form of nested arrays of strings.
// Note that the first array is one element long, the Unix timestamp of the next time the
// server will be checking for new files on the BOM server
#[get("/listing")]
fn listing() -> Result<Json<Vec<Vec<String>>>, usize> {

    let dirs;
    match fs::read_dir(DL_DIR) {
        Ok(d)  => { dirs = d }
        Err(_) => { return Err(500) }
    };

    // Will contain a Vec for each zoom level that exists
    let mut zooms = vec![];

    // Insert the timestamp first
    zooms.push(vec![format!("{}", load_refresh_time())]);

    // Iterate through the subdirectories (zoom levels)
    for dir in dirs {
        if let Ok(dir) = dir {
            if !dir.file_type().unwrap().is_dir() {
                continue;
            }

            // Will contain the filename of each file in this directory
            let mut zoom_array = vec![];

            let files = fs::read_dir(dir.path()).expect("File Error");

            // Get the path for each file
            let mut file_names: Vec<_> = files.map(|e| {
                e.expect("File error")
                    .path()
            })
            .collect();
            file_names.sort();

            // Put in a length hint in case the server just started
            if zooms[0].len() == 1 {
                zooms[0].push(format!("{}", file_names.len()));
            }

            // Iterate through the files
            for filename in file_names.iter() {
                zoom_array.push(filename.clone().to_str().unwrap().repeat(1));
            }

            zooms.push(zoom_array);
        }
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
            .mount("/", routes![index, image, resource, listing, backgrounds])
            .launch();
    });


    loop {
        // Wait for 5 minutes, then check the server every minute until we get at least
        // 1 new file
        save_refresh_time(5);
        wait_mins(5);

        while !save_files().is_ok() {
            save_refresh_time(1);
            wait_mins(1);
        }

        downloader::trim_files(30);
    }
}

// Save the unix timestamp of the next time the server will check with the BOM server
// in 'time.txt'
fn save_refresh_time(mins: u64) {
    let time = time::SystemTime::now() + time::Duration::from_secs(mins * 60);
    let secs = time.duration_since(time::UNIX_EPOCH).unwrap().as_secs();
    let mut file = File::create("time.txt").unwrap();
    file.write_all(format!("{:?}", secs).as_bytes());
}

// Load the unix timestamp stored in time.txt
fn load_refresh_time() -> u64 {
    let mut file = File::open("time.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    contents.parse().unwrap()
}
