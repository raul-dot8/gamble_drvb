#![warn(unused_extern_crates)]

use std::{collections::HashMap, fs, io};
use ciborium::de::from_reader;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io::Write;
use std::time::Duration;
use image::{self, EncodableLayout};
use rqrr;
use base45::decode;

// You MUST manually take the photo of the QR code and place it in the QR_PATH location.

// ---------- FILE CONFIGURATIONS ----------
const READ_QR: bool = true; // this option will skip reading the CBOR_TEXT
const QR_PATH: &str = "qr_code/qr.jpg";
const QR_DCDE: &str = "qr_code/b64qr.txt";

const FILE_PATH: &str = &QR_DCDE /*"drvb_files/drvb.cbor"*/;
// ----------  END CONFIGURATION  ----------

fn iterate_kv(map: &mut HashMap<i32, f64>) { // https://stackoverflow.com/a/45724688 modified to use i32 and f64
    for (key, value) in &*map {
        println!("{} / {}", key, value);
    }
    map.clear();
}

fn pause() {
    print!("\nPress Enter to exit...");
    io::stdout().flush().unwrap();

    crossterm::terminal::enable_raw_mode().unwrap();

    while event::poll(Duration::from_secs(0)).unwrap() {
        let _ = event::read(); 
    }

    loop {
        if let Ok(Event::Key(key_event)) = event::read() {
            if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Enter {
                break;
            }
        }
    }

    crossterm::terminal::disable_raw_mode().unwrap();
    println!();
}

fn main() {
    let mut drv_behaviour: HashMap<i32, f64>;

    if READ_QR {
        println!("Confirm you have placed the QR code photo in {QR_PATH} before continuing."); // Ensure to avoid errors
        pause();

        let img = image::open(QR_PATH).unwrap().to_luma8(); // Get the image. Below is adapted from the use of the rqrr example code.

        let mut img = rqrr::PreparedImage::prepare(img);
        let grids = img.detect_grids();

        assert_eq!(grids.len(), 1);
        
        let (_meta, content) = grids[0].decode().expect("Successful unwrap");

        fs::write(QR_DCDE, &content).expect("Write to file QR contents"); // Saving the contents because it takes a while to work its magic and you can reconfigure to use the saved data with FILE_PATH.

        drv_behaviour = from_reader(decode(content).unwrap().as_bytes()).unwrap();
    } else {
        let drvb_file = fs::File::open(FILE_PATH).expect("File should open.");
        let mut reader = io::BufReader::new(drvb_file);

        drv_behaviour = from_reader(&mut reader).unwrap(); // Get the key information you want.
    }

    iterate_kv(&mut drv_behaviour);
}
