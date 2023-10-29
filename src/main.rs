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
	const POLLING_FREQUENCY_MS: u64 = 1;
	const POLLING_FREQUENCY_US: u64 = 10;
	//
	let (tx_from_feedback, rx_from_feedback) = mpsc::channel();
	let mut pin_input = Gpio::new().unwrap().get(GPIO_INPUT).unwrap().into_input_pullup();
	let mut pin_output = Gpio::new().unwrap().get(GPIO_OUTPUT).unwrap().into_output();
	pin_output.set_low();
	let mut count: u32 = 1;
	//let mut capture_name;

	// first capture takes a lot longer
	capture_image(format!("capture_000000000.arw").to_string());

	//let mut camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");
	//capture_name = format!("capture_{count:0>9}.arw").to_string();
	//let mut file;
	let mut pin_input_state_last = pin_input.read();
	let mut pin_input_state_current = pin_input.read();
	println!("Pin state {}", pin_input_state_last);
	pin_output.set_low();



	let mut signal_counter: u32 = 0;
	//pin_input.set_interrupt(Trigger::FallingEdge);

	    // wait for first signal
    let _capture_thread = thread::spawn(move || -> Result<()> {
        let mut capture_name;

        while let Some(_) = rx_from_feedback.recv().unwrap() {
            capture_name = format!("capture_{count:0>9}.arw").to_string();
            //let now = Instant::now();
	    //println!("Placeholder capture");
	    //thread::sleep(Duration::from_millis(2500));
            capture_image(capture_name);
            count += 1;
            //println!("{}", now.elapsed().as_millis());
	    //total_time += now.elapsed().as_millis();
	    //if now.elapsed().as_millis() > max_time {
	    //    max_time = now.elapsed().as_millis();
	    //}
        }

	// send statistics
	//tx_from_capture.send(total_time / (count as u128));
	//tx_from_capture.send(max_time);
        Ok(())
    });

	loop {
		pin_output.set_high();
		//pin_input.poll_interrupt(true, None);

		pin_input_state_current = pin_input.read();

		if (pin_input_state_last == Level::High && pin_input_state_current == Level::Low) {
			tx_from_feedback.send(Some(1));
			thread::sleep(Duration::from_millis(10));
			pin_output.set_low();
			// Capture image
			//file = camera.capture_image().wait()?;

			//// "Slower tasks"
			//// Download image
			//camera
			//	.fs()
			//	.download_to(&file.folder(), &file.name(), Path::new(&capture_name))
			//	.wait()?;
			//println!("Downloaded image {}", capture_name);

			//count += 1;
			//capture_name = format!("capture_{count:0>9}.arw").to_string();

			signal_counter += 1;
			if signal_counter > 16 {
				break;
			}

			// Renew camera context
			//drop(camera);
			//camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");
			thread::sleep(Duration::from_millis(3000));

		}

		//println!("Pin states (last, current): {} {}", pin_input_state_last, pin_input_state_current);
		pin_input_state_last = pin_input_state_current;
		pin_output.set_high();

		//thread::sleep(Duration::from_millis(POLLING_FREQUENCY_MS));
		thread::sleep(Duration::from_micros(POLLING_FREQUENCY_US));
	}

	pin_output.set_low();

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
