use gphoto2::{Context, Result};
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::sync::mpsc;
use rppal::gpio::Gpio;
use rppal::gpio::Trigger;
use rppal::gpio::Level;


const STATES_LOW: [Level; 8] = [Level::Low; 8];
const STATES_HIGH: [Level; 8] = [Level::High; 8];
const STOP_COUNT: u64 = 64;

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
	let (tx_from_capture, rx_from_capture) = mpsc::channel();
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

			// 
			tx_from_capture.send(Some(1));
		}

		Ok(())
	});

	let mut pin_input_states: [Level; 16] = [Level::High; 16];
	let mut capturing = false;


	let mut signal_counter: u64 = 0;
	let mut now = Instant::now();
	let start = Instant::now();

	loop {
		pin_output.set_high();
		//pin_input.poll_interrupt(true, None);

		pin_input_states[15] = pin_input.read();
		//println!("{}", pin_input_states[15]);
		if !capturing && is_falling(pin_input_states) {
			// Capture image
			tx_from_feedback.send(Some(1));
			capturing = true;
			//println!("Capture start");
			now = Instant::now();
		}

		if capturing && is_rising(pin_input_states) {
			pin_output.set_low();
			println!("{}", now.elapsed().as_millis());
			//println!("Stop motor");

			// wait for capture feedback
			while let Some(_) = rx_from_capture.recv().unwrap() {
				break;
			}

			signal_counter += 1;
			if signal_counter > STOP_COUNT {
				break;
			}

			capturing = false;
			//println!("Capture end");

		}

		for i in 0..15 {
			pin_input_states[i] = pin_input_states[i + 1];
		}


		pin_output.set_high();

		//thread::sleep(Duration::from_millis(POLLING_FREQUENCY_MS));
		thread::sleep(Duration::from_micros(POLLING_FREQUENCY_US));
	}

	pin_output.set_low();


	println!("Total time: {}", start.elapsed().as_millis());
	println!("Average time per frame: {}", start.elapsed().as_millis() / (STOP_COUNT as u128 + 1));


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

fn is_rising(states: [Level; 16]) -> bool
{
	return states[0..8] == STATES_HIGH && states[8..16] == STATES_LOW;
}

fn is_falling(states: [Level; 16]) -> bool
{
	return states[0..8] == STATES_LOW && states[8..16] == STATES_HIGH;
}
