#![warn(clippy::all, clippy::pedantic)]

use chrono::{DateTime, Utc};
use rdev::{listen, Event, EventType, Key};
use std::env;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use xcap::Monitor;

const DEFAULT_WORK_DIR: &str = "Default";

#[derive(Debug)]
struct CombAppleBoardPress {
    meta_pressed: bool,
    shift_pressed: bool,
}

impl CombAppleBoardPress {
    fn new() -> Self {
        Self {
            shift_pressed: false,
            meta_pressed: false,
        }
    }

    fn set_meta_pressed(&mut self, pressed: bool) {
        self.meta_pressed = pressed;
    }

    fn set_shift_pressed(&mut self, pressed: bool) {
        self.shift_pressed = pressed;
    }
}

fn main() {
    hello();

    let args: Vec<String> = env::args().collect();
    let screenshots_dir: String = args
        .get(1)
        .unwrap_or(&DEFAULT_WORK_DIR.to_string())
        .to_string();

    match init_path(&screenshots_dir) {
        Ok(path) => {
            let mut pressed: CombAppleBoardPress = CombAppleBoardPress::new();
            if let Err(error) = listen(move |e| {
                handle_print_screen(e, &path, &mut pressed);
            }) {
                println!("Ошибка: {:?}", error);
            }
        }
        Err(e) => {
            println!("Не удалось инициализировать путь: {:?}", e);
        }
    }
}

fn hello() {
    println!(
        "Укажите название директории первым аргументом, в которую будут сохраняться скриншоты, \
        или оставьте пустым, тогда инициализируется директория: {}",
        DEFAULT_WORK_DIR
    );
}

fn init_path(dir: &str) -> std::io::Result<PathBuf> {
    // Получение текущего рабочего каталога
    let mut path = env::current_dir()?;
    path.push(dir);

    create_dir_all(&path)?;
    Ok(path)
}

// MetaLeft и MetaRight в сочитании shift + Num3 - реализация под расскладку apple
fn handle_print_screen(event: Event, dir: &Path, pressed: &mut CombAppleBoardPress) {
    match event.event_type {
        EventType::KeyPress(key) => {
            match key {
                Key::PrintScreen => {
                    make_screen(dir);
                }
                Key::MetaLeft | Key::MetaRight => {
                    pressed.set_meta_pressed(true);
                }
                Key::ShiftLeft | Key::ShiftRight => {
                    pressed.set_shift_pressed(true);
                }
                Key::Num3 => {
                    // Проверяем, нажаты ли клавиши Command и Shift вместе с клавишей 3
                    if pressed.meta_pressed && pressed.shift_pressed {
                        make_screen(dir);
                    }
                }
                _ => {}
            }
        }
        EventType::KeyRelease(key) => {
            match key {
                Key::MetaLeft | Key::MetaRight => {
                    pressed.set_meta_pressed(false); // Сбрасываем флаг meta_pressed, когда клавиша Command отпущена
                }
                Key::ShiftLeft | Key::ShiftRight => {
                    pressed.set_shift_pressed(false) // Сбрасываем флаг shift_pressed, когда клавиша Shift отпущена
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn normalized(filename: &str) -> String {
    filename.replace(['|', '\\', ':', '/'], "")
}

fn make_screen(dir: &Path) {
    let screens: Vec<Monitor> = Monitor::all().unwrap();

    for screen in screens {
        let image = screen.capture_image().unwrap();
        let now: DateTime<Utc> = Utc::now();

        let file_path = dir.join(format!(
            "{}-{}.png",
            now.format("%d-%m-%Y_%H_%M_%S_%f"),
            normalized(&screen.name())
        ));

        image.save(file_path).unwrap();
    }
}
