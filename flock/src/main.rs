#[macro_use] extern crate scan_fmt;

use std::process::{Command, Stdio, Child};
use std::path::PathBuf;
use std::thread;
use std::io::{BufReader, BufRead};
use std::sync::mpsc::{self, channel, Receiver};
use flock::{BoidFlock, FPS};
use ggez::event;
use ggez::graphics::{self, Color};
use ggez::{Context, timer, GameResult};
use glam::*;
use structopt::StructOpt;

// Command line arguments
#[derive(Debug, StructOpt)]
#[structopt(name="Boids", about="A program that runs a flock simulation following a finger from a camera.")]
struct Opt {
    /// Path to the python file
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    /// Number of boids to spawn
    #[structopt(long, default_value="150")]
    count: usize,
}

struct MainState {
    boids: BoidFlock,
    process: Child,
    process_out: Receiver<String>,
    process_active: bool,
}

impl MainState {
    fn new(ctx: &mut Context, mut process: Child, count: usize) -> GameResult<MainState> {
        let screen = graphics::screen_coordinates(ctx);
        let size = Vec2::new(screen.w, screen.h);

        let process_out = process.stdout.take().unwrap();
        let (sender, reciever) = channel::<String>();
        // Spawning a thread that will process the childs output and send over the channel
        thread::spawn(move|| {
            let reader = BufReader::new(process_out);
            // Blocking call to continually read until done
            // If an error occurs, this either means an error in reading, or the reciever was dropped, so just break as well
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        match sender.send(line) {
                            Ok(_) => {},
                            Err(_) => break,
                        }
                    },
                    Err(_) => break,
                }
            }
        });

        let s = MainState { boids: BoidFlock::new(count, size), process, process_out: reciever, process_active: true };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.process_active {
            // Checking if process has crashed
            match self.process.try_wait() {
                Ok(Some(status)) => {
                    println!("Process exited with: {}", status);
                    self.process_active = false;
                    self.boids.set_goal(None);
                }
                Err(e) => {
                    println!("Error trying to get process status: {}", e);
                    self.process_active = false;
                    self.boids.set_goal(None);
                }
                Ok(None) => {
                    // Process still active
                    match self.process_out.try_recv() {
                        Ok(line) => {
                            if line == "None" {
                                self.boids.set_goal(None);
                            }
                            else if let Ok((x,y)) = scan_fmt!(&line, "{},{}", f32, f32) {
                                let screen = graphics::screen_coordinates(ctx);
                                self.boids.set_goal(Some(Vec2::new(x * screen.w, y * screen.h)));
                            }
                            else {
                                println!("Error reading line: {}", line);
                                self.boids.set_goal(None);
                            }
                        }
                        Err(e) => {
                            if e == mpsc::TryRecvError::Disconnected {
                                // Disconnceted
                                println!("Thread monitoring process output exited");
                                self.process_active = false;
                                self.boids.set_goal(None);
                            }
                        }
                    }
                }
            }
        }
        while timer::check_update_time(ctx, FPS) {
            self.boids.update();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::WHITE.into());

        // The shape for a boid
        let boid_gfx = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            7.5,
            2.0,
            Color::BLACK,
        )?;
        // Drawing all boids
        for boid in self.boids.iter() {
            graphics::draw(ctx, &boid_gfx, (boid.pos,))?;
        }
        // Drawing the current target if it exists
        match self.boids.get_goal() {
            Some(pos) => {
                let point_gfx = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::stroke(2.0),
                    Vec2::new(0.0, 0.0),
                    10.0,
                    2.0,
                    Color::new(1.0, 0.0, 0.0, 1.0),
                )?;
                graphics::draw(ctx, &point_gfx, (pos,))?;
            },
            None => {}
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.boids.resize(Vec2::new(width, height));
        graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width, height)).unwrap();
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        // Killing the python process if it exists
        let _ = self.process.kill();
        false
    }
}

fn main() -> GameResult {
    // Get command arguments
    let opt = Opt::from_args();

    let child = Command::new("python3")
        .arg(opt.path)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start applicaiton");
    
    println!("Child spawned");

    // Make a Context.
    let (mut ctx, event_loop) = ggez::ContextBuilder::new("Flocking", "Haley Workman")
        .window_setup(ggez::conf::WindowSetup::default().title("Boids"))
        .window_mode(ggez::conf::WindowMode::default()
            .resizable(true)
            //.maximized(true)
            .fullscreen_type(ggez::conf::FullscreenType::Desktop)
        )
        .build()
        .expect("uh oh, could not create ggez context!");

    let flocker = MainState::new(&mut ctx, child, opt.count)?;

    // Run!
    event::run(ctx, event_loop, flocker);
}