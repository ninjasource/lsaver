#![windows_subsystem = "windows"]

use rand::prelude::*;
use std::collections::HashMap;
use std::env;

// FIXME: Lookup fullscreen resolution instead of hardcoding it
const WINDOW_HEIGHT: f64 = 1440.0;
const WINDOW_WIDTH: f64 = 2560.0;

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as AppWindow;
use graphics::{Context, Graphics};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use piston_window::AdvancedWindow;

mod lsystem_draw;
mod lsystem_generate;
use lsystem_draw::*;

pub struct Parameters {
    // params for rule generation
    min_rules: usize,
    max_rules: usize,
    min_start_length: usize,
    max_start_length: usize,
    min_rule_length: usize,
    max_rule_length: usize,
    lsystem_max_length: usize,
    random_angle_chance: f64,

    // params for drawing
    distance_per_movement: f64,
    line_width: f64,
    seconds_per_turtle_move: f64,
    seconds_per_fade: f64,
}

impl Parameters {
    fn new() -> Self {
        Parameters {
            min_rules: 2,
            max_rules: 5,
            min_start_length: 1,
            max_start_length: 5,
            min_rule_length: 2,
            max_rule_length: 10,
            lsystem_max_length: 2000,
            random_angle_chance: 0.5,
            distance_per_movement: 10.0,
            line_width: 0.75,
            seconds_per_turtle_move: 0.04,
            seconds_per_fade: 0.04,
        }
    }
}

pub struct TurtleStates {
    current_string_pos: usize,
    lsys: LSystem,
}

#[derive(Debug)]
struct LSystem {
    seed: String,
    string: String,
    rules: HashMap<char, String>,
    angle: f64,
}

#[derive(Clone, Debug)]
pub struct CurrentString {
    string: String,
    angle: f64,
}

#[derive(Clone)]
pub struct TurtleState {
    pos: Position,
    colour: [f32; 4], // FIXME: should this be here??
    position_stack: Vec<Position>,
}

#[derive(Copy, Clone)]
struct Position {
    x: f64,
    y: f64,
    angle: f64,
}

const FS_PER_TURTLE_MOVE: usize = 5; // number of lines to draw at once
const MAX_GROWTH_CYCLES: usize = 200;
const MIN_ANGLE: f64 = 0.08726646;
const MAX_ANGLE: f64 = 3.124139;
const NON_RANDOM_ANGLES: [f64; 7] = [
    0.3490659, 0.5235988, 0.6283185, 0.7853982, 1.047198, 1.570796, 2.356194,
];

pub struct App {
    gl: GlGraphics,
    current_strings: Vec<CurrentString>,
    turtle_states: TurtleStates,
    current_turtle_state: TurtleState,
    next_turtle_state: TurtleState,
    seconds_to_next_turtle_move: f64,
    seconds_to_next_fade: f64,
    should_fade: bool,
    params: Parameters,
    rng: ThreadRng,
}

fn main() {
    // If there are any args just quit immediately
    if env::args().len() > 2 {
        return;
    }

    let opengl = OpenGL::V3_2;

    // FIXME: get the full screen resolution automatically
    let mut window: AppWindow = WindowSettings::new("Lsaver", (2560, 1440))
        //.exit_on_esc(true)
        .graphics_api(opengl)
        .vsync(true)
        .fullscreen(true)
        .samples(4) // Anti-aliasing
        .build()
        .unwrap();

    window.set_capture_cursor(true);
    let params = Parameters::new();
    let mut rng = rand::thread_rng();

    let mut app = App {
        gl: GlGraphics::new(opengl),
        current_strings: Vec::new(),
        turtle_states: TurtleStates::new(&params, &mut rng),
        current_turtle_state: TurtleState::new(&mut rng),
        next_turtle_state: TurtleState::new(&mut rng),
        seconds_to_next_turtle_move: 0.0,
        seconds_to_next_fade: 0.0,
        should_fade: false,
        params,
        rng,
    };

    let mut mouse_move_count = 0;
    let mut key_press_count = 0;
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        // Render
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        // Timeline
        if let Some(args) = e.update_args() {
            app.update(&args);
        }

        // Exit when the user moves the mouse
        e.mouse_relative(|_| {
            mouse_move_count += 1;
            if mouse_move_count > 20 {
                std::process::exit(0);
            }
        });

        // Any key exits
        if let Some(_) = e.press_args() {
            key_press_count += 1;
            if key_press_count > 1 {
                std::process::exit(0);
            }
        }
    }

    println!("ESC pressed");
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let current_strings = &self.current_strings;

        // Since we mutate the turtle as we draw it we need to clone it beforehand as we
        // may be drawing the same turtle over and over again
        self.next_turtle_state = self.current_turtle_state.clone();

        let state = &mut self.next_turtle_state;
        let params = &self.params;

        // toggle line fading
        let should_fade = self.should_fade;
        if should_fade {
            self.should_fade = false;
        }

        self.gl.draw(args.viewport(), move |c, gl| {
            if should_fade {
                // Draw a filled semi-transparrent rectangle over everything to give the appearance that
                // everything is fading out to black
                rectangle(
                    color::hex("00000030"),
                    [0.0, 0.0, WINDOW_WIDTH, WINDOW_HEIGHT],
                    c.transform,
                    gl,
                );
            }

            for curent_string in current_strings {
                draw_lsystem_substring(
                    &curent_string.string,
                    curent_string.angle,
                    state,
                    params,
                    c,
                    gl,
                );
            }
        });
    }

    // Updates what to draw according to how the clock has progresses
    // This disconnects the draw speed from the frame rate
    fn update(&mut self, args: &UpdateArgs) {
        self.seconds_to_next_turtle_move -= args.dt;
        self.seconds_to_next_fade -= args.dt;

        // We fetch 5 or so turtle lines at a time and this controlls the rate so therefore
        // the speed at which we draw lines
        if self.seconds_to_next_turtle_move <= 0.0 {
            let current_strings: Vec<CurrentString> = loop {
                if let Some(cs) = self.turtle_states.next() {
                    break cs;
                } else {
                    // When we come to the end of our current turtle we candomly generate another one
                    // can change the pen colour
                    self.turtle_states = TurtleStates::new(&self.params, &mut self.rng);
                    self.next_turtle_state.colour = rand_colour(&mut self.rng);
                }
            };

            self.current_strings = current_strings;
            self.current_turtle_state = self.next_turtle_state.clone();
            self.seconds_to_next_turtle_move = self.params.seconds_per_turtle_move;
        }

        // This controlls the fadeout of the lines
        if self.seconds_to_next_fade <= 0.0 {
            self.seconds_to_next_fade = self.params.seconds_per_fade;
            self.should_fade = true;
        }
    }
}

fn rand_colour(rng: &mut ThreadRng) -> [f32; 4] {
    let red = rng.gen_range(0.5, 1.0);
    let green = rng.gen_range(0.5, 1.0);
    let blue = rng.gen_range(0.5, 1.0);
    [red, green, blue, 1.0]
}

impl TurtleState {
    fn new(rng: &mut ThreadRng) -> Self {
        TurtleState {
            pos: Position {
                x: 0.0,
                y: 0.0,
                angle: 0.0,
            },
            colour: rand_colour(rng),
            position_stack: Vec::new(),
        }
    }
}
