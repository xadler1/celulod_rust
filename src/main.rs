use gphoto2::{Context, Result};
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::sync::mpsc;
use rppal::gpio::Gpio;
use rppal::gpio::Trigger;
use rppal::gpio::Level;


fn main() -> Result<()>
{
	// init section
	println!("Hello, world!");

	const GPIO_INPUT: u8 = 3;
	const GPIO_OUTPUT: u8 = 17;
	const POLLING_FREQUENCY_MS: u64 = 3;
	const POLLING_FREQUENCY_US: u64 = 125;
	//
	let (tx_from_feedback, rx_from_feedback) = mpsc::channel();
	let pin_input = Gpio::new().unwrap().get(GPIO_INPUT).unwrap().into_input_pullup();
	let mut pin_output = Gpio::new().unwrap().get(GPIO_OUTPUT).unwrap().into_output();
	pin_output.set_low();
	let mut count: u32 = 1;

	// first capture takes a lot longer
	capture_image(format!("capture_000000000.arw").to_string());


	//pin_input.set_interrupt(Trigger::FallingEdge);

	let capture_thread = thread::spawn(move || -> Result<()> {
		let mut camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");
		let mut capture_name = format!("capture_{count:0>9}.arw").to_string();
		let mut file;

		while let Some(_) = rx_from_feedback.recv().unwrap() {
			//let now = Instant::now();
			file = camera.capture_image().wait()?;

			camera
				.fs()
				.download_to(&file.folder(), &file.name(), Path::new(&capture_name))
				.wait()?;
			println!("Downloaded image {}", capture_name);

			count += 1;
			capture_name = format!("capture_{count:0>9}.arw").to_string();

			// Renew camera context
			drop(camera);
			camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");
		}

		Ok(())
	});

	let mut ps_0 = Level::Low;
	let mut ps_1 = Level::Low;
	let mut ps_2 = Level::Low;
	let mut ps_3 = Level::Low;
	let mut ps_4 = Level::Low;
	let mut ps_5 = Level::Low;
	let mut ps_6 = Level::Low;
	let mut ps_7 = Level::Low;
	let mut ps_8 = Level::Low;
	let mut ps_9 = Level::Low;
	let mut ps_10 = Level::Low;
	let mut ps_11 = Level::Low;
	let mut ps_12 = Level::Low;
	let mut ps_13 = Level::Low;
	let mut ps_14 = Level::Low;
	let mut ps_15 = Level::Low;


	let mut signal_counter: u32 = 0;

	loop {
		pin_output.set_high();
		//pin_input.poll_interrupt(true, None);

		ps_15 = pin_input.read();
		if ps_0 == Level::High && ps_1 == Level::High && ps_2 == Level::High && ps_3 == Level::High && ps_4 == Level::High && ps_5 == Level::High && ps_6 == Level::High && ps_7 == Level::High &&
		   ps_8 == Level::Low && ps_9 == Level::Low && ps_10 == Level::Low && ps_11 == Level::Low && ps_12 == Level::Low && ps_13 == Level::Low && ps_14 == Level::Low && ps_15 == Level::Low {
			// Capture image
			tx_from_feedback.send(Some(1));
			thread::sleep(Duration::from_millis(20));

			pin_output.set_low();

			// could wait for feedback from capture thread instead
			thread::sleep(Duration::from_millis(3500));

			signal_counter += 1;
			if signal_counter > 64 {
				break;
			}

		}

		ps_0 = ps_1;
		ps_1 = ps_2;
		ps_2 = ps_3;
		ps_3 = ps_4;
		ps_4 = ps_5;
		ps_5 = ps_6;
		ps_6 = ps_7;
		ps_7 = ps_8;
		ps_8 = ps_9;
		ps_9 = ps_10;
		ps_10 = ps_11;
		ps_11 = ps_12;
		ps_12 = ps_13;
		ps_13 = ps_14;
		ps_14 = ps_15;


		pin_output.set_high();

		//thread::sleep(Duration::from_millis(POLLING_FREQUENCY_MS));
		thread::sleep(Duration::from_micros(POLLING_FREQUENCY_US));
	}

	pin_output.set_low();
	capture_thread.join().unwrap()?;

	println!("The wait is over.");
	Ok(())
}

// Don't know why, but reusing camera (context) leads to errors
fn capture_image(capture_name: String) -> Result<()>
{
	let camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");
	//println!("Capturing image {} ...", capture_name);
	let file = camera.capture_image().wait()?;
	camera
		.fs()
		.download_to(&file.folder(), &file.name(), Path::new(&capture_name))
		.wait()?;
	println!("Downloaded image {}", capture_name);

	Ok(())
}
