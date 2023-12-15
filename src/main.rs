use gphoto2::{Context, Result};
use std::env;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use rppal::gpio::Gpio;
use rppal::gpio::Trigger;
use rppal::gpio::Level;


const STATES_LOW: [Level; 8] = [Level::Low; 8];
const STATES_HIGH: [Level; 8] = [Level::High; 8];
const STOP_COUNT: u64 = 64;
const POLLING_FREQUENCY_MS: u64 = 3;
const POLLING_FREQUENCY_US: u64 = 125;
const GPIO_INPUT: u8 = 3;
const GPIO_OUTPUT: u8 = 17;


fn main() -> Result<()>
{
	// init section
	println!("Hello, world!");


	// first capture takes a lot longer
	//capture_image(format!("capture_000000000.arw").to_string());


	//pin_input.set_interrupt(Trigger::FallingEdge);
	let (tx_from_feedback, rx_from_feedback) = mpsc::channel();
	let (tx_from_capture, rx_from_capture) = mpsc::channel();
	let (tx_from_main_stop, rx_from_main_stop) = mpsc::channel();
	let (tx_from_main_start, rx_from_main_start) = mpsc::channel();
	let (tx_from_main_status, rx_from_main_status) = mpsc::channel();
	let mut count: u64 = 1;

	let capture_thread = thread::spawn(move || -> Result<()> {
		let mut camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");
		let mut capture_name = format!("capture_{count:0>9}.arw").to_string();
		let mut file;

		while let Some(_) = rx_from_feedback.recv().unwrap() {
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

			// Signal that capture is complete
			tx_from_capture.send(Some(1));
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
        println!("Start control_thread");
        while let Some(stop_count) = rx_from_main_start.recv().unwrap() {
            break;
        }

        println!("control_thread start signal received");
	    loop {
	    	pin_output.set_high();
	    	thread::sleep(Duration::from_micros(POLLING_FREQUENCY_US));

	    	pin_input_states[15] = pin_input.read();

	    	if !capturing && is_falling(pin_input_states) {
	    		// Capture image
	    		tx_from_feedback.send(Some(1));
	    		capturing = true;
	    	}

	    	if capturing && is_rising(pin_input_states) {
	    		pin_output.set_low();

	    		// wait for capture feedback
	    		while let Some(_) = rx_from_capture.recv().unwrap() {
	    			break;
	    		}

	    		signal_counter += 1;
	    		if stop_count != 0 && signal_counter >= stop_count {
	    			break;
	    		}

	    		capturing = false;

                if let Some(_) = rx_from_main_status.try_recv().unwrap() {
                    println!("Captured {} of {} images.", signal_counter, stop_count);
                }

                if let Some(_) = rx_from_main_stop.try_recv().unwrap() {
                    println!("Captured {} images.", signal_counter);

	    		    while let Some(stop_count) = rx_from_main_start.recv().unwrap() {
	    		    	break;
	    		    }
                }
	    	}

	    	for i in 0..15 {
	    		pin_input_states[i] = pin_input_states[i + 1];
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

	stop_motor();


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
