extern crate sfml;

use sfml::graphics::{Color, RenderTarget, RenderWindow, Transformable};
use sfml::window::{Key, VideoMode, Event, window_style};

use sfml::graphics::Texture;
use sfml::graphics::Sprite;

use std::iter::{Iterator, Cycle};
use std::slice::Iter;

fn main() {
    let mut window = RenderWindow::new(VideoMode::new_init(800, 600, 32),
                                       "Image Viewer",
                                       window_style::CLOSE,
                                       &Default::default())
        .unwrap();
    window.set_vertical_sync_enabled(true);

    let texture = Texture::new_from_file("Test_Image.png").unwrap();


    let mut textures = vec!();
    textures.push(Texture::new_from_file("Test_Image.png").unwrap());
    textures.push(Texture::new_from_file("Test2.png").unwrap());

    let mut next_index: usize = 0;


    let mut sprite = Sprite::new_with_texture(&texture).unwrap();
    
    loop {
        for event in window.events() {
            match event {
                Event::Closed => return,
                Event::KeyPressed { code: Key::Escape, .. } => return,
                Event::KeyPressed { code: Key::Right, .. } => move_sprite(&mut sprite, 5.0, 0.0),
                Event::KeyPressed { code: Key::Left, .. } => move_sprite(&mut sprite,-5.0, 0.0),
                Event::KeyPressed { code: Key::Up, .. } => move_sprite(&mut sprite, 0.0, -5.0),
                Event::KeyPressed { code: Key::Down, .. } => move_sprite(&mut sprite, 0.0, 5.0),
                Event::KeyPressed { code: Key::Return, .. } => next_index = next_image(&mut sprite, &textures, next_index),
                _ => {}
            }
        }

        window.clear(&Color::black());
        window.draw(&sprite);
        window.display();
    }
}

fn move_sprite(sprite: &mut Sprite, x: f32, y: f32){
    sprite.move2f(x, y);
}

fn next_image<'a>(sprite: &mut Sprite<'a>, images: &'a Vec<Texture>, next_texture: usize) -> usize{
    sprite.set_texture(&images[next_texture], true);
    
    println!("next_texture: {} | len(): {}", next_texture, images.len());
    if next_texture +1 < images.len() {return next_texture + 1;} else {return 0;}
}
