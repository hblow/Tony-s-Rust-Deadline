use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{WindowCanvas, Texture};
use sdl2::rect::{Point, Rect};
use sdl2::image::{LoadTexture, LoadSurface};
use sdl2::surface::Surface;
use soloud::*;

use std::time::Duration;
use std::cmp;
use rand::Rng;

const PLAYER_MOVEMENT_SPEED: i32 = 4;
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

// Basic game loop, check inputs, clear screen, re render screen after updates
// inputs -> clear -> render
enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}
struct Player {
    position: Point,
    sprite: Rect,
    speed: i32,    // note in soku pong, holding shift should slow down the player so maybe I would either store the slow down speed or multiply this value by a factor to slow
    direction: [Direction; 4],
    current_frame: i32,
}

fn render(
    canvas: &mut WindowCanvas,
    color: Color,
    texture: &Texture,
    background: &Texture,
    hud: &Texture,
    player: &Player,
) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.clear();

    let (width, height) = canvas.output_size()?;

    let (frame_width, frame_height) = player.sprite.size();
    let current_frame = Rect::new(
        player.sprite.x() + frame_width as i32 * (player.current_frame / 3),
        player.sprite.y(),
        frame_width,
        frame_height,
    );

    // Treat the center of the screen as the (0, 0) coordinate
    let screen_position = player.position + Point::new(width as i32 / 2, height as i32 / 2);
    let screen_rect = Rect::from_center(screen_position, frame_width, frame_height);
    canvas.copy(background, None, None)?;
    canvas.copy(hud, None, Rect::new(0,0,WINDOW_WIDTH,150))?;
    canvas.copy(texture, current_frame, screen_rect)?;

    canvas.present();

    Ok(())
}

fn update_player(player: &mut Player) {
    use self::Direction::*;

    for i in 0..player.direction.len(){ 
        match player.direction[i] {
            Left => {
                player.position = player.position.offset(cmp::max(-player.speed, - player.position.x() - WINDOW_WIDTH as i32 / 2), 0);
            },
            Right => {
                player.position = player.position.offset(cmp::min(player.speed, WINDOW_WIDTH as i32 / 2 - player.position.x()), 0);
            },
            Up => {
                player.position = player.position.offset(0, cmp::max(-player.speed, - player.position.y() - WINDOW_HEIGHT as i32 / 2));
            },
            Down => {
                player.position = player.position.offset(0, cmp::min(player.speed, WINDOW_HEIGHT as i32 / 2 - player.position.y()));
            },
            None => {

            }
        }
    }
    player.current_frame = (player.current_frame + 1)  % 12;
    // match player.direction {
    //     Left => {
    //         player.position = player.position.offset(-player.speed, 0);
    //     },
    //     Right => {
    //         player.position = player.position.offset(player.speed, 0);
    //     },
    //     Up => {
    //         player.position = player.position.offset(0, -player.speed2);
    //     },
    //     Down => {
    //         player.position = player.position.offset(0, player.speed);
    //     },
    //     None => {

    //     }
    // }
}

fn main() -> Result<(), String>{
    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;
    let mut window = video_subsystem.window("Pong", WINDOW_WIDTH, WINDOW_HEIGHT)
        // .position_centered()
        .build()
        .expect("could not initialize video subsystem");
    let window_icon = Surface::from_file("assets/komahappy.png")?;
    window.set_icon(window_icon);
    let mut canvas = window.into_canvas().build().expect("could not make a canvas");
    let mut rng = rand::rng();
    // canvas.set_draw_color(Color::RGB(0, 255, 255));
    // canvas.clear();
    // canvas.present();
    let mut sl = Soloud::default().unwrap();

    let mut wav = audio::Wav::default();

    wav.load(&std::path::Path::new("assets/Hod battle.ogg")).unwrap();
    
    let handle = sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep
    sl.set_looping(handle, true);

    let texture_creator = canvas.texture_creator();
    let okuu_surface = Surface::from_file("assets/walkcycle/walkcycle.png")?;
    // okuu_surface.set_color_key(true, Color::RGB(0xFF, 0x00, 0xFF))?;
    let texture = texture_creator.create_texture_from_surface(okuu_surface).unwrap();
    let hud = texture_creator.load_texture("assets/battle_hud.png")?;
    let bg_texture: Texture;
    match rng.random_range(0..=2){
        0 => {
            bg_texture = texture_creator.load_texture("assets/backgrounds/lake.png").unwrap();
        },
        1 => {
            bg_texture = texture_creator.load_texture("assets/backgrounds/dream.png").unwrap();
        },
        2 => {
            bg_texture = texture_creator.load_texture("assets/backgrounds/clock.png").unwrap();
        },
        _ => {
            bg_texture = texture_creator.load_texture("assets/backgrounds/order.png").unwrap();
        },
    }
    // let texture = texture_creator.load_texture("assets/walkcycle/walkcycle.png")?;
    // pub fn set_color_key(
    //     &mut self,
    //     enable: bool,
    //     color: Color,
    // ) -> Result<(), String>
    let mut player = Player {
        position: Point::new(0, 0),
        // src position in the spritesheet
        sprite: Rect::new(0, 0, 113, 113),
        speed: PLAYER_MOVEMENT_SPEED,
        direction: [Direction::None, Direction::None, Direction::None, Direction::None],
        current_frame: 0,
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
                Event::KeyDown { keycode: Some(Keycode::Up), repeat: false, .. } => {
                    // player.speed2 = PLAYER_MOVEMENT_SPEED;
                    player.direction[0] = Direction::Up;
                },
                Event::KeyDown { keycode: Some(Keycode::Down), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player.direction[1] = Direction::Down;
                },
                Event::KeyDown { keycode: Some(Keycode::Left), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player.direction[2] = Direction::Left;
                },
                Event::KeyDown { keycode: Some(Keycode::Right), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player.direction[3] = Direction::Right;
                },
                Event::KeyUp { keycode: Some(Keycode::Up), repeat: false, .. } => {
                    // player.speed2 = PLAYER_MOVEMENT_SPEED;
                    player.direction[0] = Direction::None;
                },
                Event::KeyUp { keycode: Some(Keycode::Down), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player.direction[1] = Direction::None;
                },
                Event::KeyUp { keycode: Some(Keycode::Left), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player.direction[2] = Direction::None;
                },
                Event::KeyUp { keycode: Some(Keycode::Right), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player.direction[3] = Direction::None;
                },
                _ => {}
            }
        }

        // Update
        i = (i + 1) % 255;
        update_player(&mut player);
        // Render
        render(&mut canvas, Color::RGB(i, 64, 255 - i), &texture, &bg_texture, &hud, &player)?;

        // Time management!
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}