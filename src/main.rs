use autopilot::key::{Code, KeyCode};
use std::time::Duration;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vid = 0x1532; // Vendor Id
    let pid = 0x0084; // Product ID

    let device =
        rusb::open_device_with_vid_pid(vid, pid).expect("Device with this VID/PID not found");
    println!("Device Found: VID={:04x} PID={:04x}", vid, pid);

    let handle = device;

    let interface_number = 2; // Botões de DPI
    handle.detach_kernel_driver(interface_number).ok();
    handle.claim_interface(interface_number)?;

    println!("Interface Claimed, waiting for packets...");

    let running = Arc::new(AtomicBool::new(true));
    {
        let running = running.clone();
        ctrlc::set_handler(move || {
            running.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");
    }

    let endpoint_address = 0x83; // Endpoint de botões especiais
    let mut buf = [0u8; 8];

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
        match handle.read_interrupt(endpoint_address, &mut buf, Duration::from_millis(5)) {
            Ok(size) => {
                if size == 8 {
                    println!("Packet received ({} bytes): {:?}", size, &buf[..size]);
                    let key_pressed = buf[2] as u8;
                    let keyboard_button = parse_special_mouse_button_packet(key_pressed);
                    match keyboard_button {
                        Some(keyboard_button) => {
                            tx.send(keyboard_button).ok(); // Envia para a thread
                        }
                        None => {}
                    }
                } else {
                    println!("Unexpected packet size: {}", size);
                }
            }
            Err(rusb::Error::Timeout) => {
                // Timeout é normal no loop
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

enum KeyboardButtons {
    F23,
    F24,
}

fn parse_special_mouse_button_packet(packet: u8) -> Option<KeyboardButtons> {
    let keyboard_button = match packet {
        114 => Some(KeyboardButtons::F23),
        115 => Some(KeyboardButtons::F24),
        _ => None,
    };
    return keyboard_button;
}
