use gphoto2::{Context, Result};
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::sync::mpsc;
use rppal::gpio::Gpio;
use rppal::gpio::Trigger;


fn main() -> Result<()>
{
    // init section
    println!("Hello, world!");

    // Create a new context and detect the first camera from it
    //let camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");
    let (tx_from_feedback, rx_from_feedback) = mpsc::channel();
    const GPIO_INPUT: u8 = 3;
    const GPIO_OUTPUT: u8 = 17;
    const OFFSET: u32 = 10;
    const SIGNALS_PER_FRAME: u32 = 1;
    const SIGNAL_CAPTURE_OFFSET: u32 = 0;
    // 
    let mut pin_input = Gpio::new().unwrap().get(GPIO_INPUT).unwrap().into_input_pullup();
    let mut pin_output = Gpio::new().unwrap().get(GPIO_OUTPUT).unwrap().into_output();
    pin_output.set_low();
    let mut count: u32 = 0;
    let mut total_time: u128 = 0;
    let mut max_time: u128 = 0;

    // first capture takes a lot longer
    capture_image(format!("capture_000000000.arw").to_string());

    // to kill the thread, not strictly necessary
    //let (tx_to_feedback, rx_to_feedback) = mpsc::channel();
    //let (tx_from_capture, rx_from_capture) = mpsc::channel();


    // signal feedback thread
    // periodically checks for signal from projector, sends signal back to main thread on every Nth (3)
    // sinal
    let signal_feedback_thread = thread::spawn(move || -> Result<()> {
        let mut signal_counter: u32 = 0;
	pin_input.set_interrupt(Trigger::FallingEdge);
	//let mut now = Instant::now();
	pin_output.set_high();

        loop {
	    pin_input.poll_interrupt(true, None);
	    // Simple hysteresis
	    thread::sleep(Duration::from_millis(10));

	    if (pin_input.is_low()) {
		//thread::sleep(Duration::from_millis(OFFSET));

	    	//println!("Pin signal low");
            	//println!("{}", now.elapsed().as_millis());

            	//if signal_counter % SIGNALS_PER_FRAME == SIGNAL_CAPTURE_OFFSET {
	    	//println!("Sending signal with count: {}", signal_counter);
            	tx_from_feedback.send(Some(1));
	    	thread::sleep(Duration::from_millis(5));

	    	pin_output.set_low();
	    	thread::sleep(Duration::from_millis(5000));
	    	//now = Instant::now();
            	//}

            	signal_counter += 1;
            	if signal_counter > 16 {
            	    break;
            	}

	    	pin_output.set_high();

	    	// timeout to not register signal more than once
            	//thread::sleep(Duration::from_millis(100));
	    }

        }

	pin_output.set_low();
	// terminate capture thread
        tx_from_feedback.send(None);

        Ok(())
    });

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



    println!("Waiting for signal_feedback_thread.");
    signal_feedback_thread.join().unwrap()?;
    println!("The wait is over.");
    //println!("Average time to capture image: {}", rx_from_capture.recv().unwrap());
    //println!("Maximum time to capture image: {}", rx_from_capture.recv().unwrap());

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
