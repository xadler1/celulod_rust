use gphoto2::{Context, Result};
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::sync::mpsc;
use rppal::gpio::Gpio;
use rppal::gpio::Error;


fn main() -> Result<()>
{
    // init section
    println!("Hello, world!");

    // Create a new context and detect the first camera from it
    //let camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");
    let (tx_from_feedback, rx_from_feedback) = mpsc::channel();
    const GPIO_SIGNAL: u8 = 17;
    // 
    let mut pin = Gpio::new().unwrap().get(GPIO_SIGNAL).unwrap().into_input();

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
        let mut should_capture: bool = false;
        loop {
	    println!("{}", pin.read());
	    if pin.is_low() {
		thread::sleep(Duration::from_millis(5));
		//println!("Pin signal low");
		continue;
	    }

	    println!("Pin signal high");
            signal_counter += 1;
            should_capture = signal_counter % 3 == 1;
            if should_capture {
                println!("Sending signal with count: {}", signal_counter);
                tx_from_feedback.send(Some(1));
            }

            if signal_counter > 32 {
                break;
            }

            thread::sleep(Duration::from_millis(200));
        }

        tx_from_feedback.send(None);

        Ok(())
    });

    // wait for first signal
    let capture_thread = thread::spawn(move || -> Result<()> {
        let mut count: u32 = 0;
        let mut capture_name = String::new();

        while let Some(should_capture) = rx_from_feedback.recv().unwrap() {
            count += 1;
            capture_name = format!("capture_{count:0>9}.arw").to_string();
            let now = Instant::now();
	    println!("Placeholder capture");
	    thread::sleep(Duration::from_millis(2500));
            //capture_image(capture_name);
            println!("{}", now.elapsed().as_millis());
        }

        Ok(())
    });


    // capture and download file
    //let mut file = camera.capture_image().wait()?;
    //println!("Captured image {}", file.name());

    //let mut count: u32 = 0;
    //let mut capture_name = String::new();

    //capture_name = increment_capture_name(&mut count);

    //camera
    //    .fs()
    //    .download_to(&file.folder(), &file.name(), Path::new(&capture_name))
    //    .wait()?;
    //println!("Downloaded image {}", capture_name);


    //thread::sleep(Duration::from_secs(1));
    //let file2 = camera.capture_image().wait()?;
    //capture_name = increment_capture_name(&mut count);

    //camera
    //    .fs()
    //    .download_to(&file2.folder(), &file2.name(), Path::new(&capture_name))
    //    .wait()?;
    //println!("Downloaded image {}", capture_name);


    println!("Waiting for signal_feedback_thread.");
    signal_feedback_thread.join().unwrap()?;
    println!("The wait is over.");

    Ok(())
}

// Don't know why, but reusing camera (context) leads to errors
fn capture_image(capture_name: String) -> Result<()>
{
    let camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");
    //let context = Context::new()?;
    //let camera_desc = context.list_cameras().wait()?.find(|desc| desc.model == "ILCE-7SM2").ok_or_else(|| format!("Could not find camera with name 'ILCE-7SM2'"))?;
    //let camera = context.get_camera(&camera_desc).wait()?;
    println!("Capturing image {} ...", capture_name);
    let file = camera.capture_image().wait()?;
    camera
        .fs()
        .download_to(&file.folder(), &file.name(), Path::new(&capture_name))
        .wait()?;
    println!("Downloaded image {}", capture_name);

    Ok(())
}
