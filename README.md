## Razer DeathAdder V2 Linux Fix

### A very simple driver just to remap the mouse's DPI buttons.

- [Razer DeathAdder V2 Linux Fix](#razer-deathadder-v2-linux-fix)
- [How to use the project](#how-to-use-the-project)
    - [Dependencies](#dependencies)
    - [Setting up the code](#setting-up-the-code)
    - [Running the code](#running-the-code)
    - [Running as a service](#running-as-a-service)
        - [Creating a service](#creating-a-service)
        - [Permissions](#permissions)
        - [Adding the service](#adding-the-service)
    - [Contributing](#contributing)



This driver was built to solve a very specific problem I had: **the Deathadder DPI buttons**.

I have, of course, a Razer DeathAdder V2 as my main mouse, and I use the DPI buttons to mute and unmute on discord. On Windows, I configured the DPI buttons in Razer Synapse to be the F23 and F24 function on the keyboard, but on my Linux (Zorin OS + Xorg) it didn't identify these buttons, and instead of configuring Xorg correctly, I made a “driver” to solve this problem.

This “driver” connects to interface 2 of the mouse, detatches the Kernel and intercepts the communications. Taking the bytes, I discovered that it sends, in the third byte, which button is being pressed, in the case of F23 and F24 that I configured, it is, respectively, 114 and 115. 

I intercept this communication and change the button being pressed to F14 and F15, which Xorg interprets correctly.

By turning this program into a service, I was able to solve my problem and now, by pressing the same mouse buttons, without changing my Windows settings, I can unmute and mutate in discord without any problems.

# How to use the project

## Dependencies

First, we need to install the Linux dependencies. 

```bash
sudo apt install libxtst-dev
sudo apt install libxdo-dev
```

## Setting up the code

The code needs to be adapted to your context. In my case, I use the F23 and F24 keys already configured within my mouse's memory instead of the DPI buttons, so you'll need to do the same thing if you don't want to change the code. 

`keyboard_buttons.rs`

```rust
pub enum KeyboardButtons {
    F23,
    F24,
}

impl KeyboardButtons {
    pub fn from_code(code: u8) -> Option<KeyboardButtons> {
        match code {
            114 => Some(KeyboardButtons::F23),
            115 => Some(KeyboardButtons::F24),
            _ => None,
        }
    }
}
```

You will also have to change which button will be remapped, in my case I put PrintScreen and ScrollLock, but they map to F14 and F15. 

`main.rs`
```rust
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
```



## Running the code

In my case, I prefer to run my code as root, since I need privileges to access the USB devices, so I execute the following command, inside the `/target/debug` folder of the project.

```bash
cargo build && sudo ./razer_deathadder_v2_buttons_control
```

## Running as a service

To run the program as a service within linux, you will have to create a new service in `systemd`.

### Creating a service

Using the following command, we can create a new file within the `systemd` services

```bash
sudo nano /etc/systemd/system/your_service_name.service
```

The following file contents are required

```
[Unit]
Description=Fix DPI buttons not working in linux
After=network.target

[Service]
ExecStart=/usr/bin/razer_deathadder_v2_buttons_control
WorkingDirectory=/usr/bin
Restart=always
User=root
Group=root
Environment=PATH=/usr/bin:/bin
Environment=LD_LIBRARY_PATH=/lib:/usr/lib
Environment=DISPLAY=:0
Environment=XAUTHORITY=/home/your_user_name/.Xauthority
StandardOutput=journal
StandardError=journal

# Reattaching driver after service stop
ExecStopPost=/bin/bash -c '/usr/bin/razer_deathadder_v2_buttons_control -r'

[Install]
WantedBy=multi-user.target
```

> **Note**: the line `ExecStopPost=/bin/bash -c '/usr/bin/razer_deathadder_v2_buttons_control -r'` is used so that when the service stops, it reattaches the driver to the kernel, to prevent the buttons from stopping working.

### Permissions

In the terminal, you need to give root permission so that it can access the X server

```bash
xhost +SI:localuser:root
```

### Adding the service

```bash
# Restarting daemon
sudo systemctl daemon-reload

# Enabling the service at startup
sudo systemctl enable your_service_name.service

# Starting the service manually
sudo systemctl start your_service_name.service
```

You can also check the logs of your service to debug it

```bash
journalctl -u your_service_name.service -f
```

Your service is now configured to start with the system and is already running.

## Contributing

If you are interested in contributing to the project, it would be much appreciated. Just open an issue and your PR and let's talk
