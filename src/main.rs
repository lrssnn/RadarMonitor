extern crate ftp;
extern crate sfml;

use std::str;
use ftp::FtpStream;

use std::io::prelude::*;
use std::fs::File;

use std::string::String;

use std::thread::sleep;
use std::time::Duration;
use std::time::SystemTime;

use sfml::graphics::{Color, RenderTarget, RenderWindow, Transformable};
use sfml::window::{Key, VideoMode, event, window_style};

use sfml::graphics::Texture;
use sfml::graphics::Sprite;

use std::iter::{Iterator, Cycle};
use std::slice::Iter;

use std::fs;
use std::fs::DirEntry;
use std::cmp::Ordering;

const DOWNLOAD_FOLDER: &'static str = "img/";
// TODO:
// Need to use this in order to delete older files.
// To do this, will need to figure out a way to list the directory again.
// Should be easier because we aren't doing comparisons
const IMAGES_KEPT: usize = 10;

// I wish I didn't have to do this
static mut texture: Option<Texture>;

// Connect to the BOM ftp server, get the radar files and save them as file_name locally.
// Returns whether or not any files were downloaded.
fn save_files(textures: &mut Vec<Texture>) -> bool {

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

	    //Add the new file to the texture list
	    textures.push(Texture::new_from_file(&(DOWNLOAD_FOLDER.to_string() + &file_name)).unwrap());
    }

    // Disconnect from the server
    let _ = ftp_stream.quit();

    downloads
}

fn correct_code_filter(name: &String) -> bool {
    name.contains("IDR043") && !name.contains(".gif")
}
/*
fn main2() {

   loop {
	while !save_files(){
	    println!("No new files");
	    wait_mins(1, true);
	}
        wait_mins(3, true);
    }
}
*/

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

fn main() {

    let mut textures: Vec<Texture> =  vec!();

    let mut window = RenderWindow::new(VideoMode::new_init(800, 600, 32),
                                       "Image Viewer",
                                       window_style::CLOSE,
                                       &Default::default())
        .unwrap();
    window.set_vertical_sync_enabled(true);

    let mut current_img = String::new();


    let mut sprite = Sprite::new().unwrap();

    save_files(&mut textures);
    let mut last_check = SystemTime::now();
    
    loop {
        for event in window.events() {
            match event {
                event::Closed => return,
                event::KeyPressed { code: Key::Escape, .. } => return,
                event::KeyPressed { code: Key::Right, .. } => move_sprite(&mut sprite, 5.0, 0.0),
                event::KeyPressed { code: Key::Left, .. } => move_sprite(&mut sprite,-5.0, 0.0),
                event::KeyPressed { code: Key::Up, .. } => move_sprite(&mut sprite, 0.0, -5.0),
                event::KeyPressed { code: Key::Down, .. } => move_sprite(&mut sprite, 0.0, 5.0),
                //event::KeyPressed { code: Key::Return, .. } => next_index = next_image(&mut sprite, &textures, next_index),
                event::KeyPressed { code: Key::Return, .. } => current_img = next_image(&mut sprite, current_img),
                _ => {}
            }
        }

        window.clear(&Color::black());
        window.draw(&sprite);
        window.display();

	if last_check.elapsed().unwrap().as_secs() > 3600 {
	    //save_files(&mut textures);
	    last_check = SystemTime::now();
	}
    }
}


fn move_sprite(sprite: &mut Sprite, x: f32, y: f32){
    sprite.move2f(x, y);
}

/*
fn next_image<'a>(sprite: &mut Sprite<'a>, images: &'a Vec<Texture>, next_texture: usize) -> usize{
    sprite.set_texture(&images[next_texture], true);
    
    println!("next_texture: {} | len(): {}", next_texture, images.len());
    if next_texture +1 < images.len() {return next_texture + 1;} else {return 0;}
}
*/

fn next_image(sprite: &mut Sprite, current_img: String) -> String {
    //First pull in a list of all the images in the directory and order it
    let files = fs::read_dir("./img/").unwrap();
    let mut file_names: Vec<_> = files.map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    file_names.sort();

    //Iterate through the list until we find the filename we are currently displaying, and show the next
    let mut found = false;
    let mut target = String::new();
    for file_name in file_names {
        if found {
	    target = file_name;
	    break;
	} else if file_name.eq(file_names.last().unwrap()) {
	    target = file_names.first().unwrap().to_string();
	} else if file_name.eq(&current_img) {
	    found = true;
	}
    }

    //Set the texture
    unsafe {
	texture = Texture::new_from_file(&target);
	sprite.set_texture(&texture.unwrap(), true);
    }

    target
}

