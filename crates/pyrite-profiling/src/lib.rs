#[cfg(feature = "profile-with-tracy")]
#[macro_use]
mod tracy;

#[cfg(feature = "profile-with-tracy")]
pub use tracy::init;

#[cfg(feature = "profile-with-tracy")]
pub use ::tracy_client;

pub struct Handle(HandleInner);

enum HandleInner {
    #[cfg(feature = "profile-with-tracy")]
    Tracy(tracy_client::Client),
}
