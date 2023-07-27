use gphoto2::{Camera, Context, Result};
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::sync::mpsc;


fn main() -> Result<()>
{
    // init section
    println!("Hello, world!");

    // Create a new context and detect the first camera from it
    //let camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");
    let (tx_from_feedback, rx_from_feedback) = mpsc::channel();
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
            signal_counter += 1;
            should_capture = signal_counter % 3 == 1;
            if should_capture {
                println!("Sending signal with count: {}", signal_counter);
                tx_from_feedback.send(Some(1));
            }

            if (signal_counter > 32) {
                break;
            }

            thread::sleep(Duration::from_millis(1000));
        }

        tx_from_feedback.send(None);

        Ok(())
    });

    // wait for first signal
    let capture_thread = thread::spawn(move || -> Result<()> {
        let mut count: u32 = 0;
        let mut capture_name = String::new();
        //let mut file;

        while let Some(should_capture) = rx_from_feedback.recv().unwrap() {
            count += 1;
            capture_name = format!("capture_{count:0>9}.arw").to_string();
            let now = Instant::now();
            capture_image(count, capture_name);
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

//fn increment_capture_name(count: &mut u32) -> String
//{
//    *count += 1;
//    return format!("capture_{count:0>9}.arw").to_string();
//}

// Don't know why, but reusing camera (context) leads to errors
fn capture_image(count: u32, capture_name: String) -> Result<()>
{
    let camera = Context::new()?.autodetect_camera().wait().expect("Failed to autodetect camera");
    println!("Capturing image {} ...", capture_name);
    let file = camera.capture_image().wait()?;
    camera
        .fs()
        .download_to(&file.folder(), &file.name(), Path::new(&capture_name))
        .wait()?;
    println!("Downloaded image {}", capture_name);

    Ok(())
}
