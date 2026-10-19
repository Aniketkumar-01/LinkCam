pub mod clipboard;
pub mod receiver;
pub mod storage;

pub use clipboard::WindowsClipboard;
pub use receiver::{CaptureReceiver, DeliveredCaptureEvent};
pub use storage::CaptureStorage;
