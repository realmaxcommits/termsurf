use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2_foundation::NSString;

mod app;
pub mod bitmap;
pub mod clipboard;
pub mod connection;
pub mod menu;
pub mod window;

mod keycodes;

pub use self::window::*;
pub use bitmap::*;
pub use connection::*;

/// Convert a rust string to an NSString
fn nsstring(s: &str) -> Retained<NSString> {
    NSString::from_str(s)
}

unsafe fn nsstring_to_str<'a>(mut ns: *mut AnyObject) -> &'a str {
    let attributed_string_cls = AnyClass::get(c"NSAttributedString").unwrap();
    let is_astring: bool =
        objc2::msg_send![ns as *const AnyObject, isKindOfClass: attributed_string_cls];
    if is_astring {
        ns = objc2::msg_send![ns as *const AnyObject, string];
    }
    let data: *const u8 = objc2::msg_send![ns, UTF8String];
    let len: usize = objc2::msg_send![ns, lengthOfBytesUsingEncoding: 4usize];
    let bytes = std::slice::from_raw_parts(data, len);
    std::str::from_utf8_unchecked(bytes)
}
