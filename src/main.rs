use game_lib::run;

fn main() {
    if let Err(err) = pollster::block_on(run()) {
        log::error!("Error: {:?}", err);
    };
}
