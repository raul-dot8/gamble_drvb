#![warn(unused_extern_crates)]

use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    time::Duration,
    process::exit,
};
use base45::decode;
use ciborium::de::from_reader;
use clap::Parser;
use crossterm::event::{
    self,
    Event,
    KeyCode,
    KeyEventKind
};
use image::EncodableLayout;
use rand::{distr::weighted::WeightedIndex, prelude::*, rng};
use rqrr::PreparedImage;
use text_io::read;
use nokhwa::{
    Camera,
    pixel_format::LumaFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType, ApiBackend},
    query
};
use imageproc::contrast::adaptive_threshold;
use facet::Facet;
use facet_yaml::{
    from_str
};

#[derive(Parser, Debug)]
#[command(version="1.0.0", about, long_about = None)]
struct Args {
    #[arg(short='r', long="read-qr", default_value_t = false, help="Force read QR code")]
    read_qr: bool,
    #[arg(short='w', long="find-camera", default_value_t = false, help="Tool to list cameras and their indices to choose for config.yaml")]
    find_camera: bool,
    #[arg(short='s', long="save-pp", default_value_t = false, help="Save the QR code photo for preprocessor tuning")]
    save_photo: bool,
    #[arg(short='c', long="config-loc", default_value = "config.yaml", help="The location of the config.yaml file")]
    config_location: String
}

#[derive(Facet)]
struct Preprocessing {
    block_size: u32,
    delta: i32,
}

#[derive(Facet)]
struct Config {
    qr_decode: String,
    camera_index: u32,
    preprocessing: Preprocessing
}

fn pause() { // Source Claude
    print!("\nPress Enter to continue...");
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

fn get_picture(config: &Config) -> image::ImageBuffer<image::Luma<u8>, Vec<u8>> { // Source nokhwa docs and example and Claude    
    let index = CameraIndex::Index(config.camera_index);
    let requested = RequestedFormat::new::<LumaFormat>(RequestedFormatType::AbsoluteHighestResolution); // Create our requested format
    let mut camera = Camera::new(index, requested).expect("Open camera"); // Open the camera
    camera.open_stream().unwrap();
    let frame = camera.frame().expect("Capture a frame"); // Get one frame

    let mut image = frame.decode_image::<LumaFormat>().expect("Decode picture"); // Turn the frame into a Luma format for rqrr

    image = adaptive_threshold(&image, config.preprocessing.block_size, config.preprocessing.delta); // Apply contrast preprocessing to sharpen the QR code

    return image; // Yield this photo for later use
}

fn wheres_my_cameras() { // Function to list the cameras in the system and pick the right one
    let cameras = query(ApiBackend::Auto).unwrap();
    for cam in cameras {
        println!("{}: {}", cam.index(), cam.human_name());
    }
}

fn read_drvb(args: Args, config: Config) -> HashMap<i32, f64> { // Read the discrete random variable from the pictures or text
    let drv_behaviour: HashMap<i32, f64>;

    let hasrunbefore: bool = fs::exists(&config.qr_decode).expect("Decoded file exists already");

    if hasrunbefore && args.read_qr {
        println!("Are you sure you want to re-read the QR code? It will take a long time. Only do this if the data has changed.");
        println!("Ctrl+C to abort and change configuration as required. Otherwise carry on.");
        pause();
    }

    if args.read_qr || !hasrunbefore {
        //println!("Confirm you have placed the QR code photo in {} before continuing.", args.qr_decode); // Ensure to avoid errors
        //pause();

        //let img = image::open(args.qr_decode).expect("Have you provided an absolute path to the file?").to_luma8(); // Get the image. Below is adapted from the use of the rqrr example code.

        println!("Present the QR code.");
        pause();

        let img = get_picture(&config); // Grab a frame through the camera.

        if args.save_photo { img.save("img_proc.png").expect("Save picture") };

        let mut img = PreparedImage::prepare(img);
        let grids = img.detect_grids();

        assert_eq!(grids.len(), 1, "QR code failed to scan. Check preprocessing");
        
        let (_meta, content) = grids[0].decode().expect("Successful unwrap");

        fs::write(config.qr_decode, &content).expect("Write to file QR contents"); // Saving the contents because it takes a while to work its magic and you can reconfigure to use the saved data with config.qr_decode.

        drv_behaviour = from_reader(decode(content).unwrap().as_bytes()).unwrap();
    } else {
        drv_behaviour = from_reader(
            decode(
                fs::read_to_string(config.qr_decode)
                .expect("Read file to string")
            ).unwrap().as_bytes()
        ).unwrap(); // Get the key information you want.
    }

    return drv_behaviour;
}

fn main(){
    let args = Args::parse();
    let cfg = fs::read_to_string(&args.config_location).expect("Open YAML config file");
    let cfg: Config = from_str(&cfg).unwrap();

    if args.find_camera { wheres_my_cameras(); exit(0); };

    let drv_hashmap = read_drvb(args, cfg);
    
    let weights: &Vec<f64> = &drv_hashmap.clone().into_values().collect();
    let choices: &Vec<i32> = &drv_hashmap.into_keys().collect();

    let distribution = WeightedIndex::new(weights).unwrap();

    print!("How many rolls of the die to take: ");
    let dicerolls: i32 = read!();

    println!("Outcomes:");
    
    //let mut vec = Vec::new();

    for _i in 1..(dicerolls+1) {
        let mut rng = rng();

        //vec.insert((i-1).try_into().unwrap(), choices[distribution.sample(&mut rng)]);

        println!("{}", choices[distribution.sample(&mut rng)]);
    }

    //fs::write("testdata-10k.txt", vec.iter().map(|n| n.to_string()).collect::<Vec<String>>().join(" "));
}
