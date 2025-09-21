use rand::rngs::ThreadRng;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::rect::{Point, Rect};
use sdl2::image::{LoadTexture, LoadSurface};
use sdl2::surface::Surface;
use sdl2::ttf;
use soloud::*;

use std::time::Duration;
use std::cmp;
use rand::Rng;

// Game param macros
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

// Player info macros
const PLAYER_MOVEMENT_SPEED: i32 = 10;
const PADDLE_HEIGHT: u32 = 80;
const PADDLE_WIDTH: u32 = 15;
const POSITION_SCALE_MOD: u32 = 8;

// Ball info macros
const BALL_HEIGHT: u32 = 10;
const BALL_WIDTH: u32=10;
const BALL_START_XSPEED: i32 = 4;
const BALL_START_YSPEED: i32 = 2;
const BALL_RAMP_SPEED: i32 = 1;
const BALL_MAX_XSPEED: i32 = 13;
const BALL_MAX_YSPEED: i32 = 7;

// Scoreboard info macros
const WIN_CONDITIONS: u32 = 6;
const SCORE_SCALE_MOD: u32 = 4;
const SCORE_HEIGHT: u32 = 60;
const SCORE_WIDTH: u32 = 30;
const CONS_HEIGHT: u32 = 50;
const CONS_WIDTH: u32 = 200;

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
    // sprite: Rect,
    speed: i32,    // note in soku pong, holding shift should slow down the player so maybe I would either store the slow down speed or multiply this value by a factor to slow
    direction: [Direction; 4],
    current_frame: i32,
}

struct Ball {
    position: Point,
    velocity: [i32;2],
    direction_x: Direction,
    direction_y:  Direction,
}

struct Scoreboard {
    p1_score: u32,
    p2_score: u32,
    p1_s_pos: Point,
    p2_s_pos: Point,
    instructions_pos: Point,
}

// struct 

fn render(
    canvas: &mut WindowCanvas,
    font: &ttf::Font,
    color: Color,
    background: &Texture,
    scoreboard: &Scoreboard,
    player1: &Player,
    player2: &Player,
    ball: &Ball,
) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.clear();

    let (width, height) = canvas.output_size()?;
    let texture_creator = canvas.texture_creator();
    // let (frame_width, frame_height) = player.sprite.size();
    // let current_frame = Rect::new(
    //     player.sprite.x() + frame_width as i32 * (player.current_frame / 3),
    //     player.sprite.y(),
    //     frame_width,
    //     frame_height,
    // );

    // // Treat the center of the screen as the (0, 0) coordinate
    // let screen_position = player.position + Point::new(width as i32 / 2, height as i32 / 2);
    // let screen_rect = Rect::from_center(screen_position, frame_width, frame_height);
    // canvas.copy(background, None, None)?;
    // canvas.copy(hud, None, Rect::new(0,0,WINDOW_WIDTH,150))?;
    // canvas.copy(texture, current_frame, screen_rect)?;
    canvas.copy(background, None, None)?;
    let mut text = format!("First to {}!", WIN_CONDITIONS);
    let mut rendered_text = font.render(&text);
    let mut surface = rendered_text.solid(Color::BLACK).unwrap();
    let mut text_texture = texture_creator.create_texture_from_surface(surface).unwrap();
    let mut screen_position = scoreboard.instructions_pos + Point::new(width as i32 / 2, height as i32 / 2);
    let mut draw_rect = Rect::from_center(screen_position, CONS_WIDTH, CONS_HEIGHT);
    canvas.copy(&text_texture, None, draw_rect)?;
    text = format!("{}", scoreboard.p1_score);
    rendered_text = font.render(&text);
    surface = rendered_text.solid(Color::BLACK).unwrap();
    text_texture = texture_creator.create_texture_from_surface(surface).unwrap();
    screen_position = scoreboard.p1_s_pos + Point::new(width as i32 / 2, height as i32 / 2);
    draw_rect = Rect::from_center(screen_position, SCORE_WIDTH, SCORE_HEIGHT);
    canvas.copy(&text_texture, None, draw_rect)?;
    text = format!("{}", scoreboard.p2_score);
    rendered_text = font.render(&text);
    surface = rendered_text.solid(Color::BLACK).unwrap();
    text_texture = texture_creator.create_texture_from_surface(surface).unwrap();
    screen_position = scoreboard.p2_s_pos + Point::new(width as i32 / 2, height as i32 / 2);
    draw_rect = Rect::from_center(screen_position, SCORE_WIDTH, SCORE_HEIGHT);
    canvas.copy(&text_texture, None, draw_rect)?;

    // Treat the center of the screen as the (0, 0) coordinate
    screen_position = player1.position + Point::new(width as i32 / 2, height as i32 / 2);
    draw_rect = Rect::from_center(screen_position, PADDLE_WIDTH, PADDLE_HEIGHT);
    canvas.fill_rect(draw_rect)?;
    screen_position = player2.position + Point::new(width as i32 / 2, height as i32 / 2);
    draw_rect = Rect::from_center(screen_position, PADDLE_WIDTH, PADDLE_HEIGHT);
    canvas.fill_rect(draw_rect)?;
    screen_position = ball.position + Point::new(width as i32 / 2, height as i32 / 2);
    draw_rect = Rect::from_center(screen_position, BALL_WIDTH,BALL_HEIGHT);
    canvas.fill_rect(draw_rect)?;
    // canvas.copy(hud, None, Rect::new(0,0,WINDOW_WIDTH,150))?;
    // canvas.copy(texture, current_frame, screen_rect)?;
    canvas.present();

    Ok(())
}

fn whack_sound(rng: &mut ThreadRng, music_player: &mut Soloud, speech: &mut Speech){
    match rng.random_range(0..=3){
        0 => {
            let _ = speech.set_text("take this");
        },
        1 => {
            let _ = speech.set_text("back at you");
        },
        2 => {
            let _ = speech.set_text("how's this");
        },
        _ => {
            let _ = speech.set_text("it's over");
        },
    }
    music_player.play(speech);
}

fn update_ball(canvas: &mut WindowCanvas, ball: &mut Ball, player: &Player) {
    use self::Direction::*;
    match ball.direction_y {
        Up => {
            if - ball.velocity[1] <  (- ball.position.y() - (WINDOW_HEIGHT - BALL_HEIGHT) as i32 / 2) {
                ball.direction_y = Down;
            }
            ball.position = ball.position.offset(0, cmp::max(-ball.velocity[1], - ball.position.y() - (WINDOW_HEIGHT - BALL_HEIGHT) as i32 / 2));

        },
        Down => {
            if ball.velocity[1] >  ((WINDOW_HEIGHT - BALL_HEIGHT) as i32 / 2 - ball.position.y()) {
                ball.direction_y = Up;
            }
            ball.position = ball.position.offset(0, cmp::min(ball.velocity[1], (WINDOW_HEIGHT - BALL_HEIGHT) as i32 / 2 - ball.position.y()));
        },
        _ => {

        }
    }

    match ball.direction_x {
        Left => {
            if - ball.velocity[0] <=  (- ball.position.x() - (WINDOW_WIDTH - BALL_WIDTH) as i32 / 2){
                ball.direction_x = Right;
            }
            ball.position = ball.position.offset(cmp::max(-ball.velocity[0], - ball.position.x() - (WINDOW_WIDTH - BALL_WIDTH) as i32 / 2), 0);
            if ball_deflected(canvas, ball, player) {
                ball.direction_x = Right;
                ball.velocity = [cmp::min(ball.velocity[0] + BALL_RAMP_SPEED, BALL_MAX_XSPEED), cmp::min(ball.velocity[1] + BALL_RAMP_SPEED, BALL_MAX_YSPEED)];
            }
        },
        Right => {
            if ball.velocity[0] >=  ((WINDOW_WIDTH - BALL_WIDTH) as i32 / 2 - ball.position.x()) {
                ball.direction_x = Left;
            }
            ball.position = ball.position.offset(cmp::min(ball.velocity[0], (WINDOW_WIDTH - BALL_WIDTH) as i32 / 2 - ball.position.x()), 0);
            if ball_deflected(canvas, ball, player) {
                ball.direction_x = Left;
                ball.velocity = [cmp::min(ball.velocity[0] + BALL_RAMP_SPEED, BALL_MAX_XSPEED), cmp::min(ball.velocity[1] + BALL_RAMP_SPEED, BALL_MAX_YSPEED)];
            }
        },
        _ => {

        }
    }
    
}

fn ball_deflected(canvas: &mut WindowCanvas, ball: &mut Ball, player: &Player) -> bool{
    let (width, height) = canvas.output_size().unwrap();

    let paddle_position = player.position + Point::new(width as i32 / 2, height as i32 / 2);
    let ball_position = ball.position + Point::new(width as i32 / 2, height as i32 / 2);
    return Rect::from_center(paddle_position, PADDLE_WIDTH, PADDLE_HEIGHT).contains_point(ball_position);
}

fn round_over(ball: &mut Ball) -> bool{
    if ball.position.x() == -((WINDOW_WIDTH - BALL_WIDTH) as i32 / 2) {
        return true
    } else if ball.position.x() == ((WINDOW_WIDTH - BALL_WIDTH) as i32 / 2){
        return true
    }
    return false
}

fn update_scoreboard(ball: &Ball, scoreboard: &mut Scoreboard) {
    match ball.direction_x {
        Direction::Left => {
            scoreboard.p1_score += 1
        },
        _ => {
            scoreboard.p2_score += 1
        }
    }
}

fn update_player(player: &mut Player) {
    use self::Direction::*;

    let mut moved:[bool; 2] = [false, false];

    for i in 0..player.direction.len(){ 
        match player.direction[i] {
            // Left => {
            //     player.position = player.position.offset(cmp::max(-player.speed, - player.position.x() - WINDOW_WIDTH as i32 / 2), 0);
            // },
            // Right => {
            //     player.position = player.position.offset(cmp::min(player.speed, WINDOW_WIDTH as i32 / 2 - player.position.x()), 0);
            // },
            Up => {
                if !moved[0] {
                    player.position = player.position.offset(0, cmp::max(-player.speed, - player.position.y() - (WINDOW_HEIGHT - PADDLE_HEIGHT) as i32 / 2));
                    moved[0] = true
                }
            },
            Down => {
                if !moved[1] {
                    player.position = player.position.offset(0, cmp::min(player.speed, (WINDOW_HEIGHT - PADDLE_HEIGHT) as i32 / 2 - player.position.y()));
                    moved[1] = true
                }
            },
            _ => {

            }
        }
    }
    player.current_frame = (player.current_frame + 1)  % 12;

}

fn reset_positions(player1: &mut Player, player2: &mut Player, ball: &mut Ball) {
    player1.position.y = 0;
    player2.position.y = 0;
    ball.position = Point::new(0,0);
    ball.velocity = [BALL_START_XSPEED, BALL_START_YSPEED]
}

fn reset_scoreboard(scoreboard: &mut Scoreboard) {
    scoreboard.p1_score = 0;
    scoreboard.p2_score = 0;
}

// fn setup_stage(rng:&mut ThreadRng, bg: &mut Texture, wav: &mut Wav, music_player: &mut Soloud, draw_color: &mut Color) -> Handle{

// }

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
    let mut draw_color = Color::BLACK;
    // canvas.set_draw_color(Color::RGB(0, 255, 255));
    // canvas.clear();
    // canvas.present();
    let mut sl = Soloud::default().unwrap();
    let font_manager = ttf::init()?;
    let font = font_manager.load_font("assets/CirnosFirstAlphabet.ttf", 24)?;
    let mut wav = audio::Wav::default();
    let mut speech = audio::Speech::default();
    let texture_creator = canvas.texture_creator();
    let opening = texture_creator.load_texture("assets/opener.jpg").unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.copy(&opening, None, None)?;
    wav.load(&std::path::Path::new("assets/LIMBUSCOMPANY.ogg")).unwrap();
    canvas.present();
    sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep
    while sl.voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    wav.load(&std::path::Path::new("assets/Malkuth battle.ogg")).unwrap();
    
    let handle = sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep
    sl.set_looping(handle, true);

    // let okuu_surface = Surface::from_file("assets/walkcycle/walkcycle.png")?;
    // okuu_surface.set_color_key(true, Color::RGB(0xFF, 0x00, 0xFF))?;
    // let texture = texture_creator.create_texture_from_surface(okuu_surface).unwrap();
    // let hud = texture_creator.load_texture("assets/battle_hud.png")?;
    let bg_texture: Texture;
    match rng.random_range(0..=2){
        0 => {
            bg_texture = texture_creator.load_texture("assets/backgrounds/dream.png").unwrap();
        },
        1 => {
            bg_texture = texture_creator.load_texture("assets/backgrounds/dream.png").unwrap();
        },
        2 => {
            bg_texture = texture_creator.load_texture("assets/backgrounds/dream.png").unwrap();
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
    let mut player1 = Player {
        position: Point::new(-((WINDOW_WIDTH / 2 - WINDOW_WIDTH / POSITION_SCALE_MOD) as i32), 0),
        // src position in the spritesheet
        // sprite: Rect::new(0, 0, 113, 113),
        speed: PLAYER_MOVEMENT_SPEED,
        direction: [Direction::None, Direction::None, Direction::None, Direction::None],
        current_frame: 0,
    };
    let mut player2 = Player {
        position: Point::new((WINDOW_WIDTH / 2 - WINDOW_WIDTH / POSITION_SCALE_MOD) as i32, 0),
        // src position in the spritesheet
        // sprite: Rect::new(0, 0, 113, 113),
        speed: PLAYER_MOVEMENT_SPEED,
        direction: [Direction::None, Direction::None, Direction::None, Direction::None],
        current_frame: 0,
    };
    let mut ball = Ball {
        position: Point::new(0, 0),
        velocity: [BALL_START_XSPEED, BALL_START_YSPEED],
        direction_x: Direction::Left,
        direction_y: Direction::Down,
    };
    let mut scoreboard = Scoreboard {
        p1_score: 0,
        p2_score: 0,
        p1_s_pos:Point::new(-((WINDOW_WIDTH / 2 - WINDOW_WIDTH / SCORE_SCALE_MOD) as i32), (WINDOW_HEIGHT / 2 - SCORE_HEIGHT / 2) as i32),
        p2_s_pos:Point::new((WINDOW_WIDTH / 2 - WINDOW_WIDTH / SCORE_SCALE_MOD) as i32, (WINDOW_HEIGHT / 2 - SCORE_HEIGHT / 2) as i32),
        instructions_pos:Point::new(0, (WINDOW_HEIGHT / 2 - CONS_HEIGHT / 2) as i32),
    };
    let mut event_pump = sdl_context.event_pump()?;
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
                    player2.direction[0] = Direction::Up;
                },
                Event::KeyDown { keycode: Some(Keycode::Down), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player2.direction[1] = Direction::Down;
                },
                Event::KeyDown { keycode: Some(Keycode::Left), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player2.direction[2] = Direction::Up;
                },
                Event::KeyDown { keycode: Some(Keycode::Right), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player2.direction[3] = Direction::Down;
                },
                Event::KeyUp { keycode: Some(Keycode::Up), repeat: false, .. } => {
                    // player.speed2 = PLAYER_MOVEMENT_SPEED;
                    player2.direction[0] = Direction::None;
                },
                Event::KeyUp { keycode: Some(Keycode::Down), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player2.direction[1] = Direction::None;
                },
                Event::KeyUp { keycode: Some(Keycode::Left), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player2.direction[2] = Direction::None;
                },
                Event::KeyUp { keycode: Some(Keycode::Right), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player2.direction[3] = Direction::None;
                },
                Event::KeyDown { keycode: Some(Keycode::W), repeat: false, .. } => {
                    // player.speed2 = PLAYER_MOVEMENT_SPEED;
                    player1.direction[0] = Direction::Up;
                },
                Event::KeyDown { keycode: Some(Keycode::S), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player1.direction[1] = Direction::Down;
                },
                Event::KeyDown { keycode: Some(Keycode::A), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player1.direction[2] = Direction::Up;
                },
                Event::KeyDown { keycode: Some(Keycode::D), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player1.direction[3] = Direction::Down;
                },
                Event::KeyUp { keycode: Some(Keycode::W), repeat: false, .. } => {
                    // player.speed2 = PLAYER_MOVEMENT_SPEED;
                    player1.direction[0] = Direction::None;
                },
                Event::KeyUp { keycode: Some(Keycode::S), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player1.direction[1] = Direction::None;
                },
                Event::KeyUp { keycode: Some(Keycode::A), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player1.direction[2] = Direction::None;
                },
                Event::KeyUp { keycode: Some(Keycode::D), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    player1.direction[3] = Direction::None;
                },
                _ => {}
            }
        }

        // Update
        update_player(&mut player1);
        update_player(&mut player2);
        match ball.direction_x {
            Direction::Left => {
                update_ball(&mut canvas, &mut ball, &player1);
                if round_over(&mut ball){
                    update_scoreboard(&ball, &mut scoreboard);
                    let _ = speech.set_text("dammit");
                    sl.play(&speech);
                    reset_positions(&mut player1, &mut player2, &mut ball);
                } else {
                    match ball.direction_x {
                        Direction::Right => {
                            whack_sound(&mut rng, &mut sl, &mut speech);
                        },
                        _ => {}
                    }
                }
            },
            _ => {
                update_ball(&mut canvas, &mut ball, &player2);
                if round_over(&mut ball){
                    update_scoreboard(&ball, &mut scoreboard);
                    let _ = speech.set_text("how dare you");
                    sl.play(&speech);
                    reset_positions(&mut player1, &mut player2, &mut ball);
                } else {
                    match ball.direction_x {
                        Direction::Left => {
                            whack_sound(&mut rng, &mut sl, &mut speech);
                        },
                        _ => {}
                    }
                }
            }
        }
        // Render
        render(&mut canvas, &font, draw_color, &bg_texture, &scoreboard, &player1, &player2, &ball)?;
        if scoreboard.p1_score == WIN_CONDITIONS  {
            let _ = speech.set_text("Player 1 Wins");
            sl.stop(handle);
            sl.play(&speech);
            while sl.voice_count() > 0 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            reset_positions(&mut player1, &mut player2, &mut ball);
            reset_scoreboard(&mut scoreboard);
            render(&mut canvas, &font, draw_color, &bg_texture, &scoreboard, &player1, &player2, &ball)?;
        } else if scoreboard.p2_score == WIN_CONDITIONS {
            let _ = speech.set_text("Player 2 Wins");
            sl.stop(handle);
            sl.play(&speech);
            while sl.voice_count() > 0 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            reset_positions(&mut player1, &mut player2, &mut ball);
            reset_scoreboard(&mut scoreboard);
            render(&mut canvas, &font, draw_color, &bg_texture, &scoreboard, &player1, &player2, &ball)?;
        }

        // Time management!
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}