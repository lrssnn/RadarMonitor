extern crate ftp;
extern crate sfml;
extern crate time;

use std::str;
use ftp::FtpStream;

use std::io::prelude::*;
use std::fs::File;

use std::string::String;

use std::thread::sleep;
use std::time::Duration;
use std::time::SystemTime;

use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::window::{VideoMode, event, window_style, Key};

use sfml::graphics::Texture;
use sfml::graphics::Sprite;

use std::iter::Iterator;

use std::fs;

use std::thread;

use std::sync::Arc;
use std::sync::Mutex;

use time::Tm;


const DOWNLOAD_FOLDER: &'static str = "img/";
const LOCATION_CODE: &'static str = "IDR043";
// TODO:
// Need to use this in order to delete older files.
// To do this, will need to figure out a way to list the directory again.
// Should be easier because we aren't doing comparisons
const IMAGES_KEPT: usize = 10;

// Main function.
fn main() {

    init();

    save_files();

    loop {

	// Create a boolean variable which we will send to the child thread when it is time to terminate
        let finish = Arc::new(Mutex::new(false));
        // Cloning an Arc actually just gives us a new reference. We move this reference into the other thread
	// but it points to the same boolean we have here.
	let clone = finish.clone();
	// Start the thread which displays the window
        thread::spawn(move || {
	    open_window(&clone);
	});
        // Wait for 5 minutes, then check the server every minute until we get at least 1 new file
	wait_mins(5, true);
	while !save_files() {
	    wait_mins(1, true);
	}
        // Lock the mutex, then tell the other thread to finish, we will replace the window with a new one next iteration.
	// Mutex is released implicitly when this loop scope ends.
        let mut finish = finish.lock().unwrap();
	*finish = true;
    }
}

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
	    file.write_all(remote_file.into_inner().as_slice());

	    downloads = true;
    }

    // Disconnect from the server
    let _ = ftp_stream.quit();

    downloads
}

fn correct_code_filter(name: &String) -> bool {
    name.contains(LOCATION_CODE) && !name.contains(".gif")
}

fn wait_mins(mut mins: u8, verbose: bool){
    let ten_sec = Duration::new(10, 0);
    loop {
        if verbose {
            print!("{}", mins);
	    std::io::stdout().flush();
	}
	for _ in 0..6 {
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

// Opens a new window, displaying only the files that currently exist in img
fn open_window(finish: &Arc<Mutex<bool>>){
    let textures = create_textures_from_files();
    let mut current_index = 0;

    let mut last_frame = time::now();
    let mut this_frame = time::now();

    let mut time_per_frame = 200;

    let mut sprite = Sprite::new_with_texture(&textures[0]).unwrap();

    let bg_texture = Texture::new_from_file(&(LOCATION_CODE.to_string() + ".background.png")).unwrap();
    let lc_texture = Texture::new_from_file(&(LOCATION_CODE.to_string() + ".locations.png")).unwrap();

    let bg = Sprite::new_with_texture(&bg_texture).unwrap();
    let lc = Sprite::new_with_texture(&lc_texture).unwrap();

    let mut window = RenderWindow::new(VideoMode::new_init(512, 512, 32),
                                        "Image Viewer",
					window_style::CLOSE,
					&Default::default())
	.unwrap();
    window.set_vertical_sync_enabled(true);

    loop {
        for event in window.events() {
	    match event {
	        event::Closed => return,
		event::KeyPressed { code: Key::Escape, .. } => return,
		//event::KeyPressed { code: Key::PageUp, .. } => time_per_frame -= 100,
		//event::KeyPressed { code: Key::PageDown, .. } => time_per_frame += 100,
                _ => {}
            }
        }

	window.clear(&Color::black());
	window.draw(&bg);
	window.draw(&lc);
	window.draw(&sprite);
	window.display();

	this_frame = time::now();
	if (this_frame - last_frame).num_milliseconds() >= time_per_frame {
	    current_index = next_image(&mut sprite, &textures, current_index);
	    last_frame = time::now();
	}

        let finish = finish.lock().unwrap();
	if *finish {
	    return;
	}
    }
}

fn create_textures_from_files() -> Vec<Texture> {
    // Get a list of filenames in the folder
    let files = fs::read_dir("./img/").unwrap();
    let mut file_names: Vec<_> = files.map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    file_names.sort();

    let textures: Vec<Texture> = file_names.iter().map(|e| Texture::new_from_file(&(DOWNLOAD_FOLDER.to_string() + e)).unwrap()).collect();
    textures
}

fn next_image<'a>(sprite: &mut Sprite<'a>, textures: &'a Vec<Texture>, current_index: usize) -> usize{
    let index = if current_index + 1 < textures.len() { current_index + 1 } else { 0 };
    sprite.set_texture(&textures[index], true);
    index
}

// Save the radar background if it is not already present
fn init() {

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
	bg_file.write_all(background_file.into_inner().as_slice());
	lc_file.write_all(location_file.into_inner().as_slice());

        // Disconnect from the server
        let _ = ftp_stream.quit();
    }
        
}
