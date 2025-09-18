use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{WindowCanvas, Texture};
use sdl2::rect::{Point, Rect};
use sdl2::image::LoadTexture;
use soloud::*;

use std::time::Duration;

// Basic game loop, check inputs, clear screen, re render screen after updates
// inputs -> clear -> render

struct Player {
    position: Point,
    sprite: Rect,
    speed: i32,    // note in soku pong, holding shift should slow down the player so maybe I would either store the slow down speed or multiply this value by a factor to slow
}

fn render(
    canvas: &mut WindowCanvas,
    color: Color,
    texture: &Texture,
    player: &Player,
) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.clear();

    let (width, height) = canvas.output_size()?;

    // Treat the center of the screen as the (0, 0) coordinate
    let screen_position = player.position + Point::new(width as i32 / 2, height as i32 / 2);
    let screen_rect = Rect::from_center(screen_position, player.sprite.width(), player.sprite.height());
    canvas.copy(texture, player.sprite, screen_rect)?;

    canvas.present();

    Ok(())
}
fn main() -> Result<(), String>{
    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem.window("Pong", 800, 600)
        // .position_centered()
        .build()
        .expect("could not initialize video subsystem");
    let mut canvas = window.into_canvas().build().expect("could not make a canvas");
    
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let sl = Soloud::default().unwrap();

    let mut wav = audio::Wav::default();

    wav.load(&std::path::Path::new("assets/Malkuth battle.ogg")).unwrap();

    sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture("assets/okuu.png")?;
    let mut player = Player {
        position: Point::new(0, 0),
        // src position in the spritesheet
        sprite: Rect::new(1, 85, 120, 120),
        speed: 2,
    };
    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;
    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    player.position = player.position.offset(-player.speed, 0);
                },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    player.position = player.position.offset(player.speed, 0);
                },
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    player.position = player.position.offset(0, -player.speed);
                },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    player.position = player.position.offset(0, player.speed);
                },
                _ => {}
            }
        }

        // Update
        i = (i + 1) % 255;

        // Render
        render(&mut canvas, Color::RGB(i, 64, 255 - i), &texture, &player)?;

        // Time management!
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}