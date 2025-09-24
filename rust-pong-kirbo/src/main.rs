use rand::rngs::ThreadRng;
use rand::seq::index;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::rect::{Point, Rect};
use sdl2::image::{LoadTexture, LoadSurface};
use sdl2::surface::Surface;
use sdl2::sys::Window;
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

const ICON_DIM: u32 = 50;

const CHAR_SELECT_BOUND: i32 = 2;
const CHAR_COUNT: i32=5;
const SELECTOR_OFFSET: i32 = 20;
const CHAR_DIM: u32 = 100;

// Basic game loop, check inputs, clear screen, re render screen after updates
// inputs -> clear -> render
enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

enum GameStage {
    CharacterSelect,
    StageSelect,
    GameLoop,
}

struct Character<'a>  {
    sprite: Rect,
    texture: Texture<'a>,
}
struct Player<'a> {
    position: Point,
    character: Character<'a>,
    selected: bool,
    choice_pos: i32,
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

fn render_char_select(
    canvas: &mut WindowCanvas,
    color: Color,
    p1: &Player,
    p2: &Player,
    characters: &[&'static str],
) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.clear();
    let mut filepath = format!("assets/characters/{}/{}_icon.png", characters[0], characters[0]);
    let (width, height) = canvas.output_size()?;
    let texture_creator = canvas.texture_creator();
    let mut icon = texture_creator.load_texture(filepath).unwrap();
    let selector_texture = texture_creator.load_texture("assets/misc/select.png").unwrap();
    let offset = (width / 5) as i32;
    let mut screen_position = Point::new(width as i32 / 2 - (2 * offset), height as i32 / 2);
    let mut draw_rect = Rect::from_center(screen_position, ICON_DIM, ICON_DIM);
    canvas.copy(&icon, None, draw_rect)?;
    let mut index_offset = 0;
    for i in 0..=4{
        index_offset = i as i32 - 2;
        filepath = format!("assets/characters/{}/{}_icon.png", characters[i], characters[i]);
        icon = texture_creator.load_texture(filepath).unwrap();
        screen_position = Point::new(width as i32 / 2 + (index_offset * offset), height as i32 / 2);
        draw_rect = Rect::from_center(screen_position, ICON_DIM, ICON_DIM);
        canvas.copy(&icon, None, draw_rect)?;
        if index_offset == p1.choice_pos{
            draw_rect = Rect::from_center(screen_position.offset(-SELECTOR_OFFSET, -SELECTOR_OFFSET), ICON_DIM, ICON_DIM);
            canvas.copy(&selector_texture, None, draw_rect)?;
            draw_rect = Rect::from_center(Point::new(width as i32 / 4, height as i32 / 4), CHAR_DIM, CHAR_DIM);
            filepath = format!("assets/characters/{}/{}_select.png", characters[i], characters[i]);
            icon = texture_creator.load_texture(filepath).unwrap();
            canvas.copy(&icon, None, draw_rect)?;
        }
        if index_offset == p2.choice_pos{
            draw_rect = Rect::from_center(screen_position.offset(SELECTOR_OFFSET, SELECTOR_OFFSET), ICON_DIM, ICON_DIM);
            canvas.copy_ex(&selector_texture, None, draw_rect, 0.0, None, true, false)?;
            draw_rect = Rect::from_center(Point::new(3 * width as i32 / 4, 3 * height as i32 / 4), CHAR_DIM, CHAR_DIM);
            filepath = format!("assets/characters/{}/{}_select.png", characters[i], characters[i]);
            icon = texture_creator.load_texture(filepath).unwrap();
            canvas.copy(&icon, None, draw_rect)?;
        }
    }
    canvas.present();
    Ok(())
}

fn render(
    canvas: &mut WindowCanvas,
    font: &ttf::Font,
    font_color: Color,
    draw_color: Color,
    background: &Texture,
    scoreboard: &Scoreboard,
    player1: &Player,
    player2: &Player,
    ball: &Ball,
) -> Result<(), String> {
    canvas.set_draw_color(draw_color);
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
    let mut surface = rendered_text.solid(font_color).unwrap();
    let mut text_texture = texture_creator.create_texture_from_surface(surface).unwrap();
    let mut screen_position = scoreboard.instructions_pos + Point::new(width as i32 / 2, height as i32 / 2);
    let mut draw_rect = Rect::from_center(screen_position, CONS_WIDTH, CONS_HEIGHT);
    canvas.copy(&text_texture, None, draw_rect)?;
    text = format!("{}", scoreboard.p1_score);
    rendered_text = font.render(&text);
    surface = rendered_text.solid(font_color).unwrap();
    text_texture = texture_creator.create_texture_from_surface(surface).unwrap();
    screen_position = scoreboard.p1_s_pos + Point::new(width as i32 / 2, height as i32 / 2);
    draw_rect = Rect::from_center(screen_position, SCORE_WIDTH, SCORE_HEIGHT);
    canvas.copy(&text_texture, None, draw_rect)?;
    text = format!("{}", scoreboard.p2_score);
    rendered_text = font.render(&text);
    surface = rendered_text.solid(font_color).unwrap();
    text_texture = texture_creator.create_texture_from_surface(surface).unwrap();
    screen_position = scoreboard.p2_s_pos + Point::new(width as i32 / 2, height as i32 / 2);
    draw_rect = Rect::from_center(screen_position, SCORE_WIDTH, SCORE_HEIGHT);
    canvas.copy(&text_texture, None, draw_rect)?;

    // Treat the center of the screen as the (0, 0) coordinate
    screen_position = player1.position + Point::new(width as i32 / 2, height as i32 / 2);
    draw_rect = Rect::from_center(screen_position, PADDLE_WIDTH, PADDLE_HEIGHT);
    canvas.fill_rect(draw_rect)?;
    canvas.copy(&player1.character.texture, None, Rect::from_center(screen_position.offset(-(PADDLE_WIDTH as i32), 0), 50, 50))?;
    screen_position = player2.position + Point::new(width as i32 / 2, height as i32 / 2);
    draw_rect = Rect::from_center(screen_position, PADDLE_WIDTH, PADDLE_HEIGHT);
    canvas.fill_rect(draw_rect)?;
    canvas.copy_ex(&player2.character.texture, None, Rect::from_center(screen_position.offset((PADDLE_WIDTH as i32), 0), 50, 50), 0.0, None, true, false)?;
    screen_position = ball.position + Point::new(width as i32 / 2, height as i32 / 2);
    draw_rect = Rect::from_center(screen_position, BALL_WIDTH * 3,BALL_HEIGHT * 3);
    canvas.copy(&texture_creator.load_texture("assets/misc/ball.png").unwrap(), None, draw_rect)?;
    // canvas.fill_rect(draw_rect)?;
     // canvas.copy(hud, None, Rect::new(0,0,WINDOW_WIDTH,150))?;
    // canvas.copy(texture, current_frame, screen_rect)?;
    canvas.present();

    Ok(())
}

fn whack_sound(rng: &mut ThreadRng, music_player: &mut Soloud, wav: &mut Wav){
    wav.load(&std::path::Path::new("assets/sfx/poyo-happy.mp3")).unwrap();
    // match rng.random_range(0..=3){
    //     0 => {
    //         let _ = wav.load(&std::path::Path::new("assets/sfx/poyo-mad.mp3")).unwrap();
    //     },
    //     1 => {
    //         let _ = speech.set_text("back at you");
    //     },
    //     2 => {
    //         let _ = speech.set_text("how's this");
    //     },
    //     _ => {
    //         let _ = speech.set_text("it's over");
    //     },
    // }
    music_player.play(wav);
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

fn setup_stage<'a>(texture_creator: &'a TextureCreator<sdl2::video::WindowContext>, rng:&mut ThreadRng, bg_texture: &mut Texture<'a>, wav: &mut Wav, draw_color: &mut Color, font_color: &mut Color){
    match rng.random_range(0..=5){
        0 => {
            *bg_texture = texture_creator.load_texture("assets/backgrounds/allies.jpeg").unwrap();
            (*wav).load(&std::path::Path::new("assets/bgm/StarAllies-FF.mp3")).unwrap();
            *draw_color = Color::BLACK;
            *font_color = Color::BLACK;

        },
        1 => {
            *bg_texture = texture_creator.load_texture("assets/backgrounds/aura.png").unwrap();
            (*wav).load(&std::path::Path::new("assets/bgm/Tak-PPPP.ogg")).unwrap();
            *draw_color = Color::BLACK;
            *font_color = Color::BLACK;
        },
        2 => {
            *bg_texture = texture_creator.load_texture("assets/backgrounds/knife.png").unwrap();
            (*wav).load(&std::path::Path::new("assets/bgm/StarAllies-Holy.mp3")).unwrap();
            *draw_color = Color::BLACK;
            *font_color = Color::BLACK;
        },
        3 => {
            *bg_texture = texture_creator.load_texture("assets/backgrounds/robobot.jpg").unwrap();
            (*wav).load(&std::path::Path::new("assets/bgm/Robobot-Ordeal.ogg")).unwrap();
            *draw_color = Color::BLACK;
            *font_color = Color::BLACK;
        },
        4 => {
            *bg_texture = texture_creator.load_texture("assets/backgrounds/yarn.jpg").unwrap();
            (*wav).load(&std::path::Path::new("assets/bgm/EpicYarn-OuterRing.mp3")).unwrap();
            *draw_color = Color::BLACK;
            *font_color = Color::BLACK;
        },
        _ => {
            *bg_texture = texture_creator.load_texture("assets/backgrounds/sad.jpg").unwrap();
            (*wav).load(&std::path::Path::new("assets/bgm/Mili-LilyTree.mp3")).unwrap();
            *draw_color = Color::BLACK;
            *font_color = Color::BLACK;
        },
    }
}

fn main() -> Result<(), String>{
    let sdl_context = sdl2::init()?;
    let characters = ["bandana", "crash", "kirby", "parasol", "plasma"];
    let video_subsystem = sdl_context.video()?;
    let mut color_grad_flag = false;
    let mut window = video_subsystem.window("Pong", WINDOW_WIDTH, WINDOW_HEIGHT)
        // .position_centered()
        .build()
        .expect("could not initialize video subsystem");
    let window_icon = Surface::from_file("assets/misc/mascot.png")?;
    window.set_icon(window_icon);
    let mut canvas = window.into_canvas().build().expect("could not make a canvas");
    let mut rng = rand::rng();
    let mut draw_color = Color::BLACK;
    let mut font_color = Color::BLACK;
    // canvas.set_draw_color(Color::RGB(0, 255, 255));
    // canvas.clear();
    // canvas.present();
    let mut sl = Soloud::default().unwrap();
    let font_manager = ttf::init()?;
    let font = font_manager.load_font("assets/misc/kirby-classic.ttf", 24)?;
    let mut wav = audio::Wav::default();
    let mut wav2 = audio::Wav::default();
    let mut speech = audio::Speech::default();
    let texture_creator = canvas.texture_creator();
    let opening = texture_creator.load_texture("assets/misc/grokopener.jpg").unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.copy(&opening, None, None)?;
    wav.load(&std::path::Path::new("assets/sfx/kirby_pawnch.mp3")).unwrap();
    canvas.present();
    let mut handle = sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep
    while sl.voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    // wav.load(&std::path::Path::new("assets/Malkuth battle.ogg")).unwrap();
    
    // let handle = sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep
    // sl.set_looping(handle, true);

    // let okuu_surface = Surface::from_file("assets/walkcycle/walkcycle.png")?;
    // okuu_surface.set_color_key(true, Color::RGB(0xFF, 0x00, 0xFF))?;
    // let texture = texture_creator.create_texture_from_surface(okuu_surface).unwrap();
    // let hud = texture_creator.load_texture("assets/battle_hud.png")?;
    let mut bg_texture: Texture = texture_creator.load_texture("assets/backgrounds/sad.jpg").unwrap();
    let mut p1_texture: Texture = texture_creator.load_texture("assets/characters/kirby/kirby_icon.png").unwrap();;
    let mut p2_texture: Texture = texture_creator.load_texture("assets/characters/kirby/kirby_icon.png").unwrap();;
    // setup_stage(&texture_creator, &mut rng, &mut bg_texture, &mut wav, &mut draw_color, &mut font_color);
    // let mut handle = sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep
    // sl.set_looping(handle, true);
    // let texture = texture_creator.load_texture("assets/walkcycle/walkcycle.png")?;
    // pub fn set_color_key(
    //     &mut self,
    //     enable: bool,
    //     color: Color,
    // ) -> Result<(), String>
    // let (mut player1, mut player2) = 
    let mut player1 = Player {
        position: Point::new(-((WINDOW_WIDTH / 2 - WINDOW_WIDTH / POSITION_SCALE_MOD) as i32), 0),
        character: Character {
            sprite: Rect::new(0, 0, 1,1),
            texture: p1_texture,
        },
        selected: false,
        choice_pos: 0,
        // src position in the spritesheet
        // sprite: Rect::new(0, 0, 113, 113),
        speed: PLAYER_MOVEMENT_SPEED,
        direction: [Direction::None, Direction::None, Direction::None, Direction::None],
        current_frame: 0,
    };
    let mut player2 = Player {
        position: Point::new((WINDOW_WIDTH / 2 - WINDOW_WIDTH / POSITION_SCALE_MOD) as i32, 0),
        character: Character {
            sprite: Rect::new(0, 0, 1,1),
            texture: p2_texture,
        },
        selected: false,
        choice_pos: 0,
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
    let mut color_i = 0;
    let mut event_pump = sdl_context.event_pump()?;
    let mut current_stage = GameStage::CharacterSelect;
    'running: loop {
        if !color_grad_flag{
            color_i = (color_i + 1);
            if color_i == 255{
                color_grad_flag = true;
            }
        } else {
            color_i = (color_i - 1);
            if color_i == 0{
                color_grad_flag = false;
            }
        }
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(Keycode::RETURN), repeat: false, .. } => {
                    // player.speed2 = PLAYER_MOVEMENT_SPEED;
                    match current_stage {
                        GameStage::CharacterSelect => {
                            if player1.selected & player2.selected {
                                current_stage = GameStage::StageSelect;
                                player1.character.texture = texture_creator.load_texture( format!("assets/characters/{}/{}_select.png", characters[(player1.choice_pos + 2) as usize], characters[(player1.choice_pos + 2) as usize])).unwrap();
                                player2.character.texture = texture_creator.load_texture( format!("assets/characters/{}/{}_select.png", characters[(player2.choice_pos + 2) as usize], characters[(player2.choice_pos + 2) as usize])).unwrap();
                            }
                        },
                        _=> {}
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Z), repeat: false, .. } => {
                    // player.speed2 = PLAYER_MOVEMENT_SPEED;
                    player1.selected = true;
                },
                Event::KeyDown { keycode: Some(Keycode::X), repeat: false, .. } => {
                    // player.speed2 = PLAYER_MOVEMENT_SPEED;
                    player1.selected = false;
                },
                Event::KeyDown { keycode: Some(Keycode::J), repeat: false, .. } => {
                    // player.speed2 = PLAYER_MOVEMENT_SPEED;
                    player2.selected = true;
                },     
                Event::KeyDown { keycode: Some(Keycode::K), repeat: false, .. } => {
                    // player.speed2 = PLAYER_MOVEMENT_SPEED;
                    player2.selected = false;
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
                    if !player2.selected {
                        player2.choice_pos = cmp::max(player2.choice_pos - 1, -CHAR_SELECT_BOUND);
                    }
                    player2.direction[2] = Direction::Up;
                },
                Event::KeyDown { keycode: Some(Keycode::Right), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    if !player2.selected {
                        player2.choice_pos = cmp::min(player2.choice_pos + 1, CHAR_SELECT_BOUND);
                    }
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
                    if !player1.selected {
                        player1.choice_pos = cmp::max(player1.choice_pos - 1, -CHAR_SELECT_BOUND);
                    }
                    player1.direction[2] = Direction::Up;
                },
                Event::KeyDown { keycode: Some(Keycode::D), repeat: false, .. } => {
                    // player.speed = PLAYER_MOVEMENT_SPEED;
                    if !player1.selected {
                        player1.choice_pos = cmp::min(player1.choice_pos + 1, CHAR_SELECT_BOUND);
                    }
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

        match current_stage{
            GameStage::CharacterSelect => {
                if sl.voice_count() == 0 {
                    wav.load(&std::path::Path::new("assets/bgm/Bomb_Rally.mp3")).unwrap();
                    handle = sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep
                    // sl.set_loop_point(handle, 23.0f64);
                    sl.set_looping(handle, true);
                    sl.loop_point(handle);
                    println!("{}", sl.loop_point(handle))
                }
                render_char_select(&mut canvas, Color::RGB(color_i, 120, 255 - color_i),&player1, &player2, &characters)?;
            },
            GameStage::StageSelect => {
                sl.stop(handle);
                setup_stage(&texture_creator, &mut rng, &mut bg_texture, &mut wav, &mut draw_color, &mut font_color);
                current_stage = GameStage::GameLoop;
                handle = sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep
                sl.set_looping(handle, true);
            }
            _=> {
                // Update
                update_player(&mut player1);
                update_player(&mut player2);
                match ball.direction_x {
                    Direction::Left => {
                        update_ball(&mut canvas, &mut ball, &player1);
                        if round_over(&mut ball){
                            update_scoreboard(&ball, &mut scoreboard);
                            wav2.load(&std::path::Path::new("assets/sfx/poyo-mad.mp3")).unwrap();
                            // let _ = speech.set_text("dammit");
                            sl.play(&wav2);
                            reset_positions(&mut player1, &mut player2, &mut ball);
                        } else {
                            match ball.direction_x {
                                Direction::Right => {
                                    whack_sound(&mut rng, &mut sl, &mut wav2);
                                },
                                _ => {}
                            }
                        }
                    },
                    _ => {
                        update_ball(&mut canvas, &mut ball, &player2);
                        if round_over(&mut ball){
                            update_scoreboard(&ball, &mut scoreboard);
                            wav2.load(&std::path::Path::new("assets/sfx/poyo-mad.mp3")).unwrap();
                            sl.play(&wav2);
                            reset_positions(&mut player1, &mut player2, &mut ball);
                        } else {
                            match ball.direction_x {
                                Direction::Left => {
                                    whack_sound(&mut rng, &mut sl, &mut wav2);
                                },
                                _ => {}
                            }
                        }
                    }
                }
                // Render
                render(&mut canvas, &font, font_color, draw_color, &bg_texture, &scoreboard, &player1, &player2, &ball)?;
                if scoreboard.p1_score == WIN_CONDITIONS  {
                    let _ = speech.set_text("Player 1 Wins");
                    sl.stop(handle);
                    sl.play(&speech);
                    while sl.voice_count() > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    reset_positions(&mut player1, &mut player2, &mut ball);
                    reset_scoreboard(&mut scoreboard);
                    setup_stage(&texture_creator, &mut rng, &mut bg_texture, &mut wav, &mut draw_color, &mut font_color);
                    handle = sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep
                    sl.set_looping(handle, true);
                    render(&mut canvas, &font, font_color, draw_color, &bg_texture, &scoreboard, &player1, &player2, &ball)?;
                } else if scoreboard.p2_score == WIN_CONDITIONS {
                    let _ = speech.set_text("Player 2 Wins");
                    sl.stop(handle);
                    sl.play(&speech);
                    while sl.voice_count() > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    reset_positions(&mut player1, &mut player2, &mut ball);
                    reset_scoreboard(&mut scoreboard);
                    setup_stage(&texture_creator, &mut rng, &mut bg_texture, &mut wav, &mut draw_color, &mut font_color);
                    handle = sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep
                    sl.set_looping(handle, true);
                    render(&mut canvas, &font, font_color, draw_color, &bg_texture, &scoreboard, &player1, &player2, &ball)?;
                }
            }
        }
        // Time management!
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}