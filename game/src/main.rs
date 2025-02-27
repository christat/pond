use env_logger;
use koi;

fn main() {
    env_logger::init();
    koi::app::new(String::from("Pond")).run().unwrap();
}
