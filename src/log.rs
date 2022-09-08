use fern::Dispatch;

pub fn init_logger() {
	Dispatch::new().level(log::LevelFilter::Warn).apply()
}
