use crystallink_protocol::{Packet, TileData, decompress_tile, MAX_UDP_PAYLOAD, TILE_SIZE};
use minifb::{Key, Window, WindowOptions};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use bincode;

const WIDTH: usize = 1920; // Default buffer size, will resize dynamically if possible or fix to 1080p
const HEIGHT: usize = 1080;

fn main() {
    let mut window = Window::new(
        "CrystalLink Receiver - Waiting for Stream...",
        WIDTH,
        HEIGHT,
        WindowOptions {
            borderless: false, // Set to true for "HDMI feel" later
            title: true,
            resize: true,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Framebuffer: ARGB format (Minifb uses u32: 00RR GGBB)
    // Default to Dark Blue (Waiting state) instead of Black to confirm App is running
    let buffer = Arc::new(Mutex::new(vec![0xFF000033; WIDTH * HEIGHT]));
    let buffer_clone = buffer.clone();

    // UDP Listener Thread
    thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:5555").expect("Could not bind socket");
        println!("Listening on 0.0.0.0:5555");
        
        // Discovery Beacon: Announce presence to Mac
        socket.set_broadcast(true).ok();
        let beacon_socket = socket.try_clone().unwrap();
        thread::spawn(move || {
            loop {
                let msg = bincode::serialize(&Packet::Discovery { hostname: "WinReceiver".into() }).unwrap();
                match beacon_socket.send_to(&msg, "255.255.255.255:5556") {
                    Ok(_) => println!("Beacon sent to 255.255.255.255:5556..."),
                    Err(e) => println!("Failed to send beacon: {}", e),
                }
                thread::sleep(Duration::from_secs(1));
            }
        });

        let mut buf = [0u8; MAX_UDP_PAYLOAD];
        let mut first_packet = true;
        loop {
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    if first_packet {
                        println!("Received FIRST packet from {}! Connection Established.", src);
                        first_packet = false;
                    }

                    let packet: Packet = match bincode::deserialize(&buf[..amt]) {
                        Ok(p) => p,
                        Err(_) => continue,
                    };

                    match packet {
                        Packet::TileBatch { tiles, .. } => {
                            let mut fb = buffer_clone.lock().unwrap();
                            for tile in tiles {
                                draw_tile(&mut fb, tile, WIDTH);
                            }
                        }
                        Packet::Cursor { x, y } => {
                            // Render cursor logic here (Overlay)
                            // For MVP, maybe verify simple drawing first
                        }
                        _ => {}
                    }
                }
                Err(e) => eprintln!("Error receiving: {}", e),
            }
        }
    });

    // Main Window Loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let fb = buffer.lock().unwrap();
        
        // Minifb expects u32 buffer
        window
            .update_with_buffer(&fb, WIDTH, HEIGHT)
            .unwrap();
    }
}

fn draw_tile(fb: &mut Vec<u32>, tile: TileData, stride: usize) {
    let raw = decompress_tile(&tile);
    // Raw is typically RGBA (u8) -> Need to convert to ARGB (u32)
    // Assuming Sender sends RGBA [R, G, B, A, R, G, B, A...]
    
    let tile_w = TILE_SIZE as usize;
    let tile_h = TILE_SIZE as usize;
    let start_x = tile.x as usize;
    let start_y = tile.y as usize;

    for y in 0..tile_h {
        for x in 0..tile_w {
            let buffer_idx = (start_y + y) * stride + (start_x + x);
            let tile_idx = (y * tile_w + x) * 4;
            
            if buffer_idx < fb.len() && tile_idx + 3 < raw.len() {
                let r = raw[tile_idx] as u32;
                let g = raw[tile_idx+1] as u32;
                let b = raw[tile_idx+2] as u32;
                // Minifb format: 0x00RRGGBB
                fb[buffer_idx] = (r << 16) | (g << 8) | b;
            }
        }
    }
}
