#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

mod contract;
mod emitter;
mod errors;
mod storage;

pub use contract::{BlendV21Receiver, BlendV21ReceiverClient};
pub use emitter::Swap;
pub use errors::ReceiverError;
pub use storage::Config;

#[cfg(test)]
mod test;
