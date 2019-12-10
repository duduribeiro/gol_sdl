extern crate sdl2;
extern crate gol;

use sdl2::rect::{Point, Rect};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::{Window, WindowContext};
use sdl2::render::{Canvas, Texture, TextureCreator};

pub const SQUARE_SIZE: u32 = 5;
pub const PLAYGROUND_WIDTH: u32 = 128;
pub const PLAYGROUND_HEIGHT: u32 = 128;

mod game_of_life {
    #[derive(Copy, Clone)]
    pub enum State {
        Paused,
        Playing,
    }

    pub struct GameOfLife {
        state: State,
    }

    impl GameOfLife {
        pub fn new() -> GameOfLife {
            GameOfLife {
                state: State::Paused,
            }
        }

        pub fn toggle_state(&mut self) {
            self.state = match self.state {
                State::Paused => State::Playing,
                State::Playing => State::Paused,
            }
        }

        pub fn state(&self) -> State {
            self.state
        }
    }
}

fn dummy_texture<'a>(canvas: &mut Canvas<Window>, texture_creator: &'a TextureCreator<WindowContext>) -> Result<(Texture<'a>, Texture<'a>), String> {
    enum TextureColor {
        Yellow,
        White,
    };
    let mut square_texture1 = texture_creator.create_texture_target(None, SQUARE_SIZE, SQUARE_SIZE).map_err(|e| e.to_string())?;
    let mut square_texture2 = texture_creator.create_texture_target(None, SQUARE_SIZE, SQUARE_SIZE).map_err(|e| e.to_string())?;
        // let's change the textures we just created
    {
        let textures = vec![
            (&mut square_texture1, TextureColor::Yellow),
            (&mut square_texture2, TextureColor::White)
        ];
        canvas.with_multiple_texture_canvas(textures.iter(), |texture_canvas, user_context| {
            texture_canvas.set_draw_color(Color::RGB(0, 0, 0));
            texture_canvas.clear();
            match *user_context {
                TextureColor::Yellow => {
                    for i in 0..SQUARE_SIZE {
                        for j in 0..SQUARE_SIZE {
                            if (i+j) % 4 == 0 {
                                texture_canvas.set_draw_color(Color::RGB(255, 255, 0));
                                texture_canvas.draw_point(Point::new(i as i32, j as i32))
                                    .expect("could not draw point");
                            }
                            if (i+j*2) % 9 == 0 {
                                texture_canvas.set_draw_color(Color::RGB(200, 200, 0));
                                texture_canvas.draw_point(Point::new(i as i32, j as i32))
                                    .expect("could not draw point");
                            }
                        }
                    }
                },
                TextureColor::White => {
                    for i in 0..SQUARE_SIZE {
                        for j in 0..SQUARE_SIZE {
                            // drawing pixel by pixel isn't very effective, but we only do it once and store
                            // the texture afterwards so it's still alright!
                            if (i+j) % 7 == 0 {
                                // this doesn't mean anything, there was some trial and error to find
                                // something that wasn't too ugly
                                texture_canvas.set_draw_color(Color::RGB(192, 192, 192));
                                texture_canvas.draw_point(Point::new(i as i32, j as i32))
                                    .expect("could not draw point");
                            }
                            if (i+j*2) % 5 == 0 {
                                texture_canvas.set_draw_color(Color::RGB(64, 64, 64));
                                texture_canvas.draw_point(Point::new(i as i32, j as i32))
                                    .expect("could not draw point");
                            }
                        }
                    }
                }
            };
            for i in 0..SQUARE_SIZE {
                for j in 0..SQUARE_SIZE {
                    // drawing pixel by pixel isn't very effective, but we only do it once and store
                    // the texture afterwards so it's still alright!
                    if (i+j) % 7 == 0 {
                        // this doesn't mean anything, there was some trial and serror to find
                        // something that wasn't too ugly
                        texture_canvas.set_draw_color(Color::RGB(192, 192, 192));
                        texture_canvas.draw_point(Point::new(i as i32, j as i32))
                            .expect("could not draw point");
                    }
                    if (i+j*2) % 5 == 0 {
                        texture_canvas.set_draw_color(Color::RGB(64, 64, 64));
                        texture_canvas.draw_point(Point::new(i as i32, j as i32))
                            .expect("could not draw point");
                    }
                }
            }
        }).map_err(|e| e.to_string())?;
    }
    Ok((square_texture1, square_texture2))
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // the window is the representation of a window in your operating system,
    // however you can only manipulate properties of that window, like its size, whether it's
    // fullscreen, ... but you cannot change its content without using a Canvas or using the
    // `surface()` method.
    let window = video_subsystem
        .window("rust-sdl2 demo: Game of Life",
                SQUARE_SIZE*PLAYGROUND_WIDTH,
                SQUARE_SIZE*PLAYGROUND_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    // the canvas allows us to both manipulate the property of the window and to change its content
    // via hardware or software rendering. See CanvasBuilder for more info.
    let mut canvas = window.into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    println!("Using SDL_Renderer \"{}\"", canvas.info().name);
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    // clears the canvas with the color we set in `set_draw_color`.
    canvas.clear();
    // However the canvas has not been updated to the window yet, everything has been processed to
    // an internal buffer, but if we want our buffer to be displayed on the window, we need to call
    // `present`. We need to call this everytime we want to render a new frame on the window.
    canvas.present();

    // this struct manages textures. For lifetime reasons, the canvas cannot directly create
    // textures, you have to create a `TextureCreator` instead.
    let texture_creator : TextureCreator<_> = canvas.texture_creator();

    // Create a "target" texture so that we can use our Renderer with it later
    let (square_texture1, square_texture2) = dummy_texture(&mut canvas, &texture_creator)?;
    let mut game = game_of_life::GameOfLife::new();
    let mut universe = gol::Universe::new();

    let mut event_pump = sdl_context.event_pump()?;
    let mut frame : u32 = 0;
    'running: loop {
        // get the inputs here
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Space), repeat: false, .. } => {
                    game.toggle_state();
                },
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for (i, cell) in universe.get_cells().iter().enumerate() {
            let i = i as u32;
            let symbol = if *cell == gol::Cell::Dead { '◻' } else { '◼' };
            let square_texture = if frame >= 15 {
                &square_texture1
            } else {
                &square_texture2
            };
            if *cell == gol::Cell::Alive {
                canvas.copy(square_texture,
                            None,
                            Rect::new(((i % PLAYGROUND_WIDTH) * SQUARE_SIZE) as i32,
                                    ((i / PLAYGROUND_WIDTH) * SQUARE_SIZE) as i32,
                                    SQUARE_SIZE,
                                    SQUARE_SIZE))?;
            }
        }

        canvas.present();
        if let game_of_life::State::Playing = game.state() {
            universe.tick();
            frame += 1;
        };
    }

    Ok(())
}
