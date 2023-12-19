use gphoto2::{Context, Result};
use std::fs::OpenOptions;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use rppal::gpio::Gpio;
use rppal::gpio::Trigger;
use rppal::gpio::Level;


const STATES_LOW: [Level; 8] = [Level::Low; 8];
const STATES_HIGH: [Level; 8] = [Level::High; 8];
const STOP_COUNT: u64 = 16;
const POLLING_FREQUENCY_MS: u64 = 3;
const POLLING_FREQUENCY_US: u64 = 125;
const GPIO_INPUT: u8 = 3;
const GPIO_OUTPUT: u8 = 17;


fn main() -> Result<()>
{
	// init section
	println!("Hello, world!");
	stop_motor();


	// first capture takes a lot longer
	//capture_image(format!("capture_000000000.arw").to_string());


	//pin_input.set_interrupt(Trigger::FallingEdge);
	let (tx_from_feedback, rx_from_feedback) = mpsc::channel::<Result<Option<u64>>>();
	let (tx_from_capture, rx_from_capture) = mpsc::channel::<Result<Option<u64>>>();
	let (tx_from_main_stop, rx_from_main_stop) = mpsc::channel();
	let (tx_from_main_start, rx_from_main_start) = mpsc::channel();
	let (tx_from_main_status, rx_from_main_status) = mpsc::channel();
	let mut count: u64 = 1;
	let mut debug_stats: Vec<Level> = vec![];

	let capture_thread = thread::spawn(move || -> Result<()> {
		let mut camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");
		let mut capture_name = format!("capture_{count:0>9}.arw").to_string();
		let mut file;

		loop {
			match rx_from_feedback.recv().unwrap() {
				Ok(_) => {file = camera.capture_image().wait()?;

				camera
					.fs()
					.download_to(&file.folder(), &file.name(), Path::new(&capture_name))
					.wait()?;
				//println!("Downloaded image {}", capture_name);

				count += 1;
				capture_name = format!("capture_{count:0>9}.arw").to_string();

				// Renew camera context
				drop(camera);
				camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");

				// Signal that capture is complete
				tx_from_capture.send(Ok(Some(1)));
				},
				Err(_) => break,
			}
		}

		Ok(())
	});


	let mut capturing = false;
	let mut signal_counter = 0;
	let pin_input = Gpio::new().unwrap().get(GPIO_INPUT).unwrap().into_input_pullup();
	let mut pin_output = Gpio::new().unwrap().get(GPIO_OUTPUT).unwrap().into_output();
	let mut pin_input_states: [Level; 16] = [Level::High; 16];
	let mut stop_count = STOP_COUNT;

	let control_thread = thread::spawn(move || {
		loop {
			while let Some(message) = rx_from_main_start.recv().unwrap() {
				stop_count = message;
				break;
			}

			println!("Capturing {} images", stop_count);
			let mut now = Instant::now();
			let mut now_complement = Instant::now();
			let mut waited: u128 = 0;
			let mut waited_complement: u128 = 0;
			let mut file = OpenOptions::new().write(true).create(true).append(true).open("stats.txt").unwrap();
			loop {
				pin_output.set_high();
				thread::sleep(Duration::from_micros(POLLING_FREQUENCY_US));

				pin_input_states[15] = pin_input.read();
				debug_stats.push(pin_input_states[15]);

				if !capturing && is_rising(pin_input_states) {
					// Capture image
					tx_from_feedback.send(Ok(Some(1)));
					capturing = true;
					now = Instant::now();
					waited_complement = now_complement.elapsed().as_millis();
				}

				if capturing && is_falling(pin_input_states) {
					waited = now.elapsed().as_millis();

					pin_output.set_low();
					println!("Waited: {}, Waited complement: {}", waited, waited_complement);
						// wait for capture feedback
					loop {
						match rx_from_capture.recv().unwrap() {
							Ok(_) => break,
							Err(_) => break,
						}
					}


					signal_counter += 1;
					if stop_count != 0 && signal_counter >= stop_count {
						break;
					}

					capturing = false;

					match rx_from_main_status.try_recv() {
						Ok(_) => {
							println!("Captured {} of {} images.", signal_counter, stop_count);
						},
						Err(TryRecvError::Disconnected) => {},
						Err(TryRecvError::Empty) => {},
					}

					match rx_from_main_stop.try_recv() {
						Ok(_) => {
							println!("Captured {} images.", signal_counter);
							break;
						},
						Err(TryRecvError::Disconnected) => {},
						Err(TryRecvError::Empty) => {},
					}

					for i in 0..debug_stats.len() {
						let mut symbol: u8 = 48;
						if (debug_stats[i] == Level::High) {
							symbol = 49;
						}
						file.write_all(&[symbol]);
					}
					debug_stats = vec![];
					// Could this fix occasional delayed captures? NO
					//thread::sleep(Duration::from_millis(1000));
					now_complement = Instant::now();
				}

				for i in 0..15 {
					pin_input_states[i] = pin_input_states[i + 1];
				}

			}
		}
	});



	loop {
		let input = prompt("> ");

		if input == "capture" {
			let capture_count: u64 = prompt("> count? ").parse().unwrap();
			tx_from_main_start.send(Some(capture_count));
		} else if input == "stop" {
			tx_from_main_stop.send(Some(1));
		} else if input == "status" {
			tx_from_main_status.send(Some(1));
		} else if input == "exit" {
			break;
		} else {
			println!("unknown commanad");
		}
	}

	//capture_thread.join().unwrap()?;

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

fn stop_motor()
{
	Gpio::new().unwrap().get(GPIO_OUTPUT).unwrap().into_output().set_low();
}

// Interactive functions
fn prompt(prompt: &str) -> String
{
	let mut line = String::new();
	print!("{}", prompt);
	std::io::stdout().flush().unwrap();
	std::io::stdin().read_line(&mut line).expect("Error reading line");
	return line.trim().to_string();
}
