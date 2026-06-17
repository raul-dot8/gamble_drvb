#![warn(unused_extern_crates)]

use std::{collections::HashMap, fs};
use text_io::read;
use ciborium::ser::into_writer;
use base45::encode;
use fast_qr::convert::{svg::SvgBuilder, Builder, Shape};
use fast_qr::qr::QRBuilder;

// ---------- FILE CONFIGURATIONS ----------
const CREATE_CBOR_TEXT: bool = true;
const CREATE_CBOR_QR: bool = true;

const CBOR_FILE: &str = "drvb_files/drvb.cbor";
const CBOR_TEXT: &str = "drvb_files/drvb.b64qr";
const CBOR_QR: &str = "drvb_files/drvb.b64qr.svg";
// ----------  END CONFIGURATION  ----------

fn main(){
    let mut drv: HashMap<i32, f64> = HashMap::new();

    print!("x start value: ");
    let start_x: i32 = read!();
    print!("x end value: ");
    let end_x: i32 = read!();

    let mut p: f64;

    print!("\n");

    for i in start_x..(end_x+1) { // Get the probability value for each value of x required.
        print!("p value of {}: ",i);
        p = read!();

        drv.insert(i, p);
    }

    let mut vec = Vec::new();
    into_writer(&drv, &mut vec).expect("Serialise drv HashMap"); // Serialise the data above into CBOR.
    fs::write(CBOR_FILE, &vec).expect("CBOR written"); // Write the raw CBOR to the file

    let mut base45_cbor = String::new();
    if CREATE_CBOR_TEXT {
        base45_cbor = encode(&vec);
        fs::write(CBOR_TEXT, &base45_cbor).expect("Write text variant (base45) for QR codes"); // If chosen by user write the text variant (required for QR).
    }
    if CREATE_CBOR_QR && CREATE_CBOR_TEXT {
        let qr = QRBuilder::new(base45_cbor).build().unwrap(); // Create the QR code.
        
        let _svg = SvgBuilder::default().shape(Shape::Square).to_file(&qr, CBOR_QR); // Save the QR code.
    }
}