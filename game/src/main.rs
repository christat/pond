use env_logger;
use koi;

fn main() {
    env_logger::init();
    koi::app::new(c"Pond")
        .run()
        .expect("Pond - failed to run Main Loop");
}
