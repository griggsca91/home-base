use nats;

fn main() {
    println!("Hello, world!");
    let nc = nats::connect("localhost:4222").unwrap();

    nc.publish("foo", "Rust is here");
}
