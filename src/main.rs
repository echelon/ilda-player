// Copyright (c) 2017 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

//! ilda-player is a program that plays ILDA laser projection files on laser
//! projector DACs.

#![deny(missing_docs)]
#![deny(unreachable_patterns)]
#![deny(unused_extern_crates)]
#![deny(unused_imports)]
#![deny(unused_qualifications)]

extern crate argparse;
extern crate ilda;
extern crate lase;

use ilda::animation::Animation;
use lase::Point;
use lase::tools::find_first_etherdream_dac;
use argparse::ArgumentParser;
use argparse::{Store, StoreTrue};

fn main() {
  let mut filename = String::new();
  let mut show_blanking = false;

  { // Limit scope of borrow.
    let mut parser = ArgumentParser::new();

    parser.set_description("ILDA laser projection file player.");
    parser.refer(&mut filename)
        .add_argument("filename", Store, "ILDA file to load");
    parser.refer(&mut show_blanking)
        .add_option(&["-b", "--show-blanking"], StoreTrue,
            "Show the blanking points");

    parser.parse_args_or_exit();
  }

  println!("Reading ILDA file... {}", filename);

  let animation = Animation::read_file(&filename).expect("File should load.");

  println!("Searching for EtherDream DAC...");

  let mut dac = find_first_etherdream_dac().expect("Unable to find DAC");

  let mut current_frame = 0;
  let mut current_point = 0;

  dac.play_function(move |num_points: u16| {
    let num_points = num_points as usize;
    let mut buf = Vec::new();

    while buf.len() < num_points {
      let frame = match animation.get_frame(current_frame) {
        Some(frame) => frame,
        None => {
          current_frame = 0;
          current_point = 0;
          continue;
        }
      };

      let point = match frame.get_point(current_point) {
        Some(point) => point,
        None => {
          current_frame += 1;
          current_point = 0;
          continue;
        },
      };

      current_point += 1;

      if point.is_blank && !show_blanking {
        buf.push(Point::xy_blank(invert_x(point.x), point.y));
      } else {
        // The DAC supports a wider colorspace than ILDA.
        buf.push(Point::xy_rgb(
          invert_x(point.x),
          point.y,
          expand(point.r),
          expand(point.g),
          expand(point.b)));
      }
    }

    buf
  }).expect("Streaming to the DAC is broken.");
}

fn invert_x(x_coordinate: i16) -> i16 {
  // Compensate for flipped x-coordinate plane.
  // TODO: This might be a bug in the ILDA parser, or perhaps Etherdream.rs.
  x_coordinate.saturating_mul(-1)
}

fn expand(color: u8) -> u16 {
  (color as u16) * 257 // or the incorrect: (color as u16) << 8
}
