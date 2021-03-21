use nats;
use serde_json::{json};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, Duration, Instant};
use std::{time, thread};
use rppal::gpio::{Gpio, Level};


#[derive(Serialize, Deserialize, Debug)]
struct GarageSensorState {
    distance: f64,
    time: u64
}

const GPIO_DISTANCE_READ_PIN: u8 = 24;
const GPIO_DISTANCE_TRIGGER_PIN: u8 = 20;

/**
 * 
 def calculate_distance_inches(high_level_time_secs):
 return calculate_distance_cm(high_level_time_secs) / 2.54

 def calculate_distance_cm(high_level_time_secs):
 return ((high_level_time_secs * 340) / 2) * 100
 */

fn calculate_distance(milliseconds: u128) -> f64 {
    ((((milliseconds as f64 / 1_000.0) * 340.0) / 2.0) * 100.0) / 2.54
}

fn post_update(conn: &nats::Connection) {
    let gpio = Gpio::new().unwrap();

    let mut trigger_pin = gpio.get(GPIO_DISTANCE_TRIGGER_PIN).unwrap().into_output();
    let mut read_pin = gpio.get(GPIO_DISTANCE_READ_PIN).unwrap().into_input();

    trigger_pin.set_high();
    thread::sleep(Duration::from_millis(100));
    trigger_pin.set_low();

    let start = Instant::now();
    let mut reading = false;

    println!("Reading input");

    loop {
        let level = read_pin.read();
        if level == Level::High && reading == false {
            reading = true
        }
        else if level == Level::Low && reading == true {
            break
        }
    }

    let total_time = start.elapsed().as_millis();
    
    println!("total time: {:?}", total_time);
    println!("distance: {:?}", calculate_distance(total_time));
    let distance = calculate_distance(total_time);

    let payload  = GarageSensorState {
        distance: distance,
        time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    };

    let payload_json = json!(payload);

    conn.publish("foo", payload_json.to_string());
}

fn main() {
    let nc = nats::connect("167.99.232.215:4222").unwrap();

    loop {
        let ten_millis = time::Duration::from_millis(1_000);
        let now = time::Instant::now();

        thread::sleep(ten_millis);
        println!("posting update");
        post_update(&nc)
    }
}
