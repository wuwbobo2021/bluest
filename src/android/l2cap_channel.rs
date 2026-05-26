#![cfg(feature = "l2cap")]

use std::pin;
use std::task::{Context, Poll};

pub use android_ble::{L2capChannelReader, L2capChannelWriter};
use futures_lite::{AsyncRead, AsyncWrite};

use crate::l2cap_channel::{derive_async_read, derive_async_write};

pub struct L2capChannel {
    pub(super) reader: L2capChannelReader,
    pub(super) writer: L2capChannelWriter,
}

impl L2capChannel {
    pub fn split(self) -> (L2capChannelReader, L2capChannelWriter) {
        (self.reader, self.writer)
    }
}

derive_async_read!(L2capChannel, reader);
derive_async_write!(L2capChannel, writer);
