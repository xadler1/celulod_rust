use gphoto2::{Context, Result};
use std::path::Path;
use std::thread;
use std::time::Duration;
use rppal::gpio::Gpio;
use rppal::gpio::Level;
use rppal::pwm::{Channel, Polarity, Pwm};

const STOP_COUNT: u64 = 12000;
const STATES_LOW: [Level; 8] = [Level::Low; 8];
const STATES_HIGH: [Level; 8] = [Level::High; 8];
const POLLING_FREQUENCY_US: u64 = 500;
const STOP_DELAY_MS: u64 = 500;
const PWM_FREQUENCY: f64 = 440.0;
const DUTY_CYCLE_RUN: f64 = 0.0;
const DUTY_CYCLE_STOP: f64 = 1.0;
const GPIO_INPUT: u8 = 3;

fn main() -> Result<()>
{
	let pwm = Pwm::with_frequency(Channel::Pwm0, 440.0, 1.0, Polarity::Normal, true).unwrap();
	let pin_input = Gpio::new().unwrap().get(GPIO_INPUT).unwrap().into_input_pullup();
	let mut capture_name;
	let mut pin_input_states: [Level; 16] = [Level::High; 16];
	let mut count: u64 = 1;

	let _ = pwm.set_frequency(PWM_FREQUENCY, DUTY_CYCLE_RUN);

	loop {
		thread::sleep(Duration::from_micros(POLLING_FREQUENCY_US));

		pin_input_states[15] = pin_input.read();

		if is_rising(pin_input_states) {
			let _ = pwm.set_frequency(PWM_FREQUENCY, DUTY_CYCLE_STOP);
			thread::sleep(Duration::from_millis(STOP_DELAY_MS));
			
			capture_name = format!("capture_{count:0>9}.arw").to_string();
			let _ = capture_image(capture_name);
			count += 1;

			if STOP_COUNT != 0 && count >= STOP_COUNT {
				break;
			}

			let _ = pwm.set_frequency(PWM_FREQUENCY, DUTY_CYCLE_RUN);
		}

		for i in 0..15 {
			pin_input_states[i] = pin_input_states[i + 1];
		}

	}


	println!("The wait is over.");
	let _ = pwm.set_frequency(PWM_FREQUENCY, DUTY_CYCLE_STOP);
	loop {
		thread::sleep(Duration::from_millis(10000));
	}

	Ok(())
}

// Don't know why, but reusing camera (context) leads to errors
fn capture_image(capture_name: String) -> Result<()>
{
	let camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");
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
