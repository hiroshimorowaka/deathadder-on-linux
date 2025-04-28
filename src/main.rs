use autopilot::key::{Code, KeyCode};
use std::time::Duration;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};
mod keyboard_buttons;

use keyboard_buttons::KeyboardButtons;

use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long)]
    reattach: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Vendor ID and Product ID of Razer Deathadder V2
    let vid = 0x1532;
    let pid = 0x0084;

    let device =
        rusb::open_device_with_vid_pid(vid, pid).expect("Device with this VID/PID not found");
    println!("Device Found: VID={:04x} PID={:04x}", vid, pid);

    let handle = device;

    let interface_number = 2; // DPI Buttons

    let args = Args::parse();

    if args.reattach {
        println!("Reattaching kernel driver...");
        handle.attach_kernel_driver(interface_number)?;
        println!("Kernel driver reattached. Goodbye!");
        return Ok(());
    }

    handle.detach_kernel_driver(interface_number).ok();
    handle.claim_interface(interface_number)?;

    println!("Interface Claimed, waiting for packets...");

    // Detect Ctrl-C and stop program without just closing it, for driver reattachment
    let running = Arc::new(AtomicBool::new(true));
    {
        let running = running.clone();
        ctrlc::set_handler(move || {
            running.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");
    }

    let endpoint_address = 0x83; // DPI Buttons Endpoint
    let mut buf = [0u8; 8];

    // Create a thread to handle the packets
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        while let Ok(button) = rx.recv() {
            match button {
                KeyboardButtons::F23 => {
                    autopilot::key::tap(&Code(KeyCode::PrintScreen), &[], 1, 0);
                }
                KeyboardButtons::F24 => {
                    autopilot::key::tap(&Code(KeyCode::ScrollLock), &[], 1, 0);
                }
            }
        }
    });

    while running.load(Ordering::SeqCst) {
        match handle.read_interrupt(endpoint_address, &mut buf, Duration::from_millis(300)) {
            Ok(size) => {
                if size == 8 {
                    println!("Packet received ({} bytes): {:?}", size, &buf[..size]);
                    let key_pressed = buf[2] as u8;
                    let keyboard_button = KeyboardButtons::from_code(key_pressed);
                    match keyboard_button {
                        Some(keyboard_button) => {
                            tx.send(keyboard_button).ok(); // Send to thread
                        }
                        None => {
                            if key_pressed != 0 {
                                println!("Unknown key pressed: {}", key_pressed);
                            }
                        }
                    }
                } else {
                    println!("Unexpected packet size: {}", size);
                }
            }
            Err(rusb::Error::Timeout) => {
                // Ignore timeouts
            }
            Err(e) => {
                eprintln!("Read error: {:?}", e);
                break;
            }
        }
    }

    println!("Releasing interface...");
    handle.release_interface(interface_number)?;
    handle.attach_kernel_driver(interface_number)?;
    println!("Kernel driver reattached. Goodbye!");

    Ok(())
}
