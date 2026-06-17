#![warn(unused_extern_crates)]

use nokhwa::{
    Camera,
    pixel_format::LumaFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType, ApiBackend},
    query
};

use rqrr::PreparedImage;

use imageproc::contrast::adaptive_threshold;

fn wheres_my_cameras() { // Function to list the cameras in the system and pick the right one
    let cameras = query(ApiBackend::Auto).unwrap();
    for cam in cameras {
        println!("{}: {}", cam.index(), cam.human_name());
    }
}

fn get_picture() -> image::ImageBuffer<image::Luma<u8>, Vec<u8>> {
    //wheres_my_cameras();
    
    let index = CameraIndex::Index(1); // Create our camera index from 1 (webcam) known from function above.

    let requested = RequestedFormat::new::<LumaFormat>(RequestedFormatType::AbsoluteHighestResolution); // Create our requested format

    let mut camera = Camera::new(index, requested).expect("Open camera"); // Open the camera

    let frame = camera.frame().expect("Capture a frame"); // Get one frame

    let mut image = frame.decode_image::<LumaFormat>().expect("Decode picture"); // Turn the frame into a Luma format for rqrr

    image = adaptive_threshold(&image, 25, 7); // Apply contrast preprocessing to sharpen the QR code

    return image; // Yield this photo for later use
}

fn main() {
    let image = get_picture();

    image.save("frame.png").unwrap(); // Save it to see what it looks like and adjust preprocessing

    let mut image = PreparedImage::prepare(image);
    let grids = image.detect_grids();

    assert_eq!(grids.len(), 1); // Need this to make sure it will actually read it properly below
        
    let (_meta, content) = grids[0].decode().expect("Decoding QR code"); // Decode the QR code

    println!("{}", content); // Read out the content within
}