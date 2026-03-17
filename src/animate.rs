use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::sync::watch::Receiver;
use tokio::time::sleep;
use rand::Rng;
use std::io::{self, Write};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{Hide, Show},
};
use image::{DynamicImage, GenericImageView, imageops::grayscale};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Weather {
    Clear,
    Rain,
    Snow,
    Thunderstorm,
}

impl Weather {
    pub fn from_str(weather_str: &str) -> Self {
        if weather_str.contains("rain") || weather_str.contains("drizzle") || weather_str.contains("rain showers") {
            Weather::Rain
        } else if weather_str.contains("snow") || weather_str.contains("Snow") {
            Weather::Snow
        } else if weather_str.contains("Thunderstorm") {
            Weather::Thunderstorm
        } else {
            Weather::Clear
        }
    }
}

#[derive(Debug, Clone)]
struct Particle {
    x: u16,
    y: u16,
}

#[derive(Debug, Clone, Copy)]
enum Intensity {
    Light,
    Moderate,
    Heavy,
}

impl Intensity {
    fn delay_ms(&self) -> u64 {
        match self {
            Intensity::Light => 80,
            Intensity::Moderate => 40,
            Intensity::Heavy => 15,
        }
    }

    fn spawn_probability(&self) -> f64 {
        match self {
            Intensity::Light => 0.2,
            Intensity::Moderate => 0.5,
            Intensity::Heavy => 0.8,
        }
    }
}

pub async fn animate_weather(
    image: &DynamicImage,
    weather: &Weather,
    weather_str: &str,
    is_grayscale: bool,
    mut exit_rx: Receiver<bool>,
) -> Result<()> {
    let (cols, rows) = get_terminal_size();

    execute!(io::stdout(), EnterAlternateScreen, Hide)?;

    let intensity = match weather {
        Weather::Rain => Intensity::Moderate,
        Weather::Snow => Intensity::Light,
        Weather::Thunderstorm => Intensity::Heavy,
        Weather::Clear => Intensity::Light,
    };
    
    let result = match weather {
        Weather::Rain => animate_rain(image, weather_str, rows, cols, intensity, is_grayscale, &mut exit_rx).await,
        Weather::Snow => animate_snow(image, weather_str, rows, cols, intensity, is_grayscale, &mut exit_rx).await,
        Weather::Thunderstorm => animate_thunderstorm(image, weather_str, rows, cols, intensity, is_grayscale, &mut exit_rx).await,
        Weather::Clear => print_static(image, weather_str, is_grayscale, &mut exit_rx).await,
    };

    execute!(io::stdout(), LeaveAlternateScreen, Show)?;
    
    result
}

async fn animate_rain(
    image: &DynamicImage,
    weather_str: &str,
    rows: u16,
    cols: u16,
    intensity: Intensity,
    is_grayscale: bool,
    exit_rx: &mut Receiver<bool>,
) -> Result<()> {
    let mut particles: Vec<Particle> = Vec::new();
    let mut rng = rand::thread_rng();
    let delay = Duration::from_millis(intensity.delay_ms());
    let spawn_prob = intensity.spawn_probability();
    let mut last_frame = Instant::now();

    let resized = image.resize_exact(cols as u32, rows.saturating_sub(2) as u32 * 2, image::imageops::FilterType::Lanczos3);
    let resized = if is_grayscale {
        image::DynamicImage::ImageLuma8(grayscale(&resized))
    } else {
        resized
    };

    loop {
        if *exit_rx.borrow() {
            break;
        }

        if rng.gen_bool(spawn_prob) {
            let x = rng.gen_range(0..cols);
            particles.push(Particle { x, y: 0 });
        }

        let speed = match intensity {
            Intensity::Light => 1,
            Intensity::Moderate => 2,
            Intensity::Heavy => 3,
        };
        
        particles.retain_mut(|p| {
            p.y += speed;
            p.y < rows.saturating_sub(2)
        });

        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        println!("{weather_str}\n");

        for term_y in 0..rows.saturating_sub(2) as u32 {
            for x in 0..cols as u32 {
                let top = resized.get_pixel(x, term_y * 2);
                let bot = resized.get_pixel(x, term_y * 2 + 1);
                let is_raining = particles.iter().any(|p| p.x as u32 == x && p.y as u32 == term_y);
                let (fr, fg, fb) = if is_raining {
                    ((bot[0] as u16 + 100).min(255) as u8,
                     (bot[1] as u16 + 150).min(255) as u8,
                     (bot[2] as u16 + 255).min(255) as u8)
                } else { (bot[0], bot[1], bot[2]) };
                let (br, bg, bb) = if is_raining {
                    ((top[0] as u16 + 100).min(255) as u8,
                     (top[1] as u16 + 150).min(255) as u8,
                     (top[2] as u16 + 255).min(255) as u8)
                } else { (top[0], top[1], top[2]) };
                print!("\x1b[38;2;{fr};{fg};{fb}m\x1b[48;2;{br};{bg};{bb}m▄");
            }
            print!("\x1b[m");
        }
        
        io::stdout().flush()?;

        let elapsed = last_frame.elapsed();
        if elapsed < delay {
            sleep(delay - elapsed).await;
        }
        last_frame = Instant::now();
    }

    Ok(())
}

async fn animate_snow(
    image: &DynamicImage,
    weather_str: &str,
    rows: u16,
    cols: u16,
    intensity: Intensity,
    is_grayscale: bool,
    exit_rx: &mut Receiver<bool>,
) -> Result<()> {
    let mut particles: Vec<Particle> = Vec::new();
    let mut rng = rand::thread_rng();
    let delay = Duration::from_millis(intensity.delay_ms());
    let spawn_prob = intensity.spawn_probability();
    let mut last_frame = Instant::now();

    let resized = image.resize_exact(cols as u32, rows.saturating_sub(2) as u32 * 2, image::imageops::FilterType::Lanczos3);
    let resized = if is_grayscale {
        image::DynamicImage::ImageLuma8(grayscale(&resized))
    } else {
        resized
    };

    loop {
        if *exit_rx.borrow() {
            break;
        }

        if rng.gen_bool(spawn_prob) {
            let x = rng.gen_range(0..cols);
            particles.push(Particle { x, y: 0 });
        }

        particles.retain_mut(|p| {
            p.y += 1;
            if rng.gen_bool(0.4) {
                p.x = (p.x as i16 + rng.gen_range(-1..=1)).clamp(0, cols as i16 - 1) as u16;
            }
            p.y < rows.saturating_sub(2)
        });

        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        println!("{weather_str}\n");
        
        for term_y in 0..rows.saturating_sub(2) as u32 {
            for x in 0..cols as u32 {
                let top = resized.get_pixel(x, term_y * 2);
                let bot = resized.get_pixel(x, term_y * 2 + 1);
                let is_snowing = particles.iter().any(|p| p.x as u32 == x && p.y as u32 == term_y);
                let (fr, fg, fb) = if is_snowing {
                    ((bot[0] as u16 + 80).min(255) as u8,
                     (bot[1] as u16 + 80).min(255) as u8,
                     (bot[2] as u16 + 120).min(255) as u8)
                } else { (bot[0], bot[1], bot[2]) };
                let (br, bg, bb) = if is_snowing {
                    ((top[0] as u16 + 80).min(255) as u8,
                     (top[1] as u16 + 80).min(255) as u8,
                     (top[2] as u16 + 120).min(255) as u8)
                } else { (top[0], top[1], top[2]) };
                print!("\x1b[38;2;{fr};{fg};{fb}m\x1b[48;2;{br};{bg};{bb}m▄");
            }
            print!("\x1b[m");
        }
        
        io::stdout().flush()?;

        let elapsed = last_frame.elapsed();
        if elapsed < delay {
            sleep(delay - elapsed).await;
        }
        last_frame = Instant::now();
    }

    Ok(())
}

async fn animate_thunderstorm(
    image: &DynamicImage,
    weather_str: &str,
    rows: u16,
    cols: u16,
    intensity: Intensity,
    is_grayscale: bool,
    exit_rx: &mut Receiver<bool>,
) -> Result<()> {
    let mut particles: Vec<Particle> = Vec::new();
    let mut rng = rand::thread_rng();
    let delay = Duration::from_millis(intensity.delay_ms());
    let spawn_prob = intensity.spawn_probability().min(0.9);
    let mut last_frame = Instant::now();
    let mut flash_counter = 0;
    let flash_interval = 40;

    let resized = image.resize_exact(cols as u32, rows.saturating_sub(2) as u32 * 2, image::imageops::FilterType::Lanczos3);
    let resized = if is_grayscale {
        image::DynamicImage::ImageLuma8(grayscale(&resized))
    } else {
        resized
    };

    loop {
        if *exit_rx.borrow() {
            break;
        }

        if rng.gen_bool(spawn_prob) {
            let x = rng.gen_range(0..cols);
            particles.push(Particle { x, y: 0 });
        }

        let speed = match intensity {
            Intensity::Light => 1,
            Intensity::Moderate => 2,
            Intensity::Heavy => 4,
        };
        
        particles.retain_mut(|p| {
            p.y += speed;
            p.y < rows.saturating_sub(2)
        });

        let should_flash = flash_counter % flash_interval == 0 && rng.gen_bool(0.15);

        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        
        if should_flash {
            println!("\x1b[48;2;255;255;150m{}\x1b[0m\n", weather_str);
        } else {
            println!("{weather_str}\n");
        }
        
        for term_y in 0..rows.saturating_sub(2) as u32 {
            for x in 0..cols as u32 {
                let mut top = resized.get_pixel(x, term_y * 2);
                let mut bot = resized.get_pixel(x, term_y * 2 + 1);
                if should_flash {
                    top = image::Rgba([
                        (top[0] as u16 + 150).min(255) as u8,
                        (top[1] as u16 + 150).min(255) as u8,
                        (top[2] as u16 + 100).min(255) as u8, 255]);
                    bot = image::Rgba([
                        (bot[0] as u16 + 150).min(255) as u8,
                        (bot[1] as u16 + 150).min(255) as u8,
                        (bot[2] as u16 + 100).min(255) as u8, 255]);
                }
                let is_raining = particles.iter().any(|p| p.x as u32 == x && p.y as u32 == term_y);
                let (fr, fg, fb) = if is_raining {
                    ((bot[0] as u16 + 60).min(255) as u8,
                     (bot[1] as u16 + 100).min(255) as u8,
                     (bot[2] as u16 + 180).min(255) as u8)
                } else {
                    let a = ((bot[0] as u16 + bot[1] as u16 + bot[2] as u16) / 3) as u8;
                    (a, a, a)
                };
                let (br, bg, bb) = if is_raining {
                    ((top[0] as u16 + 60).min(255) as u8,
                     (top[1] as u16 + 100).min(255) as u8,
                     (top[2] as u16 + 180).min(255) as u8)
                } else {
                    let a = ((top[0] as u16 + top[1] as u16 + top[2] as u16) / 3) as u8;
                    (a, a, a)
                };
                print!("\x1b[38;2;{fr};{fg};{fb}m\x1b[48;2;{br};{bg};{bb}m▄");
            }
            print!("\x1b[m");
        }
        
        io::stdout().flush()?;
        flash_counter += 1;

        let elapsed = last_frame.elapsed();
        if elapsed < delay {
            sleep(delay - elapsed).await;
        }
        last_frame = Instant::now();
    }

    Ok(())
}

async fn print_static(
    image: &DynamicImage,
    weather_str: &str,
    is_grayscale: bool,
    exit_rx: &mut Receiver<bool>,
) -> Result<()> {
    let (cols, rows) = get_terminal_size();
    let resized = image.resize_exact(cols as u32, rows.saturating_sub(2) as u32 * 2, image::imageops::FilterType::Lanczos3);
    let resized = if is_grayscale {
        image::DynamicImage::ImageLuma8(grayscale(&resized))
    } else {
        resized
    };
    
    execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
    println!("{weather_str}\n");
    
    for term_y in 0..rows.saturating_sub(2) as u32 {
        for x in 0..cols as u32 {
            let top = resized.get_pixel(x, term_y * 2);
            let bot = resized.get_pixel(x, term_y * 2 + 1);
            print!("\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m▄",
                bot[0], bot[1], bot[2], top[0], top[1], top[2]);
        }
        print!("\x1b[m");
    }
    
    io::stdout().flush()?;

    loop {
        if *exit_rx.borrow() {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}

fn get_terminal_size() -> (u16, u16) {
    match crossterm::terminal::size() {
        Ok((cols, rows)) => (cols, rows),
        Err(_) => (80, 24),
    }
}