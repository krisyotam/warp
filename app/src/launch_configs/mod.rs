pub mod launch_config;
#[cfg(feature = "local_fs")]
pub mod permanent_pins;
pub mod save_modal;

use warpui::AppContext;

pub fn init(app: &mut AppContext) {
    save_modal::init(app);
}
