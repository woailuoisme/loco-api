#[ctor::ctor(unsafe)]
fn init_test_env() {
    loco_api::env_loader::load_test_env();
}

mod models;
mod requests;
mod tasks;
mod workers;
