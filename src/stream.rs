use std::{
	fmt::{Debug, Formatter},
	io, mem, str,
};

use bytes::Bytes;
use futures_core::Stream;
use tokio::io::{AsyncBufReadExt, AsyncReadExt};
use tokio_util::io::StreamReader;

use crate::Result;

pub(crate) type StreamResult = io::Result<Bytes>;
pub(crate) type BoxedStream = Box<dyn Stream<Item = StreamResult> + Unpin + Send + 'static>;
pub(crate) type Reader = StreamReader<BoxedStream, Bytes>;

#[cfg(todo)]
const BOUNDARY_START: &[u8] = b"--";

pub struct MjpegStream {
	reader: Reader,

	boundary: String,
	mime: String,
	pair: Vec<u8>,
	jpeg: Vec<u8>,
}

impl Debug for MjpegStream {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "MjpegStream {{ ... }}")
	}
}

impl MjpegStream {
	pub(crate) fn from_boxed_stream(stream: BoxedStream) -> Self {
		Self {
			reader: Reader::new(stream),
			boundary: String::with_capacity(17),
			mime: String::with_capacity(24),
			pair: vec![0; 2],
			jpeg: Vec::new(),
		}
	}

	pub async fn read_frame(&mut self) -> Result<()> {
		self.boundary.clear();
		self.reader.read_line(&mut self.boundary).await?;
		self.mime.clear();
		self.reader.read_line(&mut self.mime).await?;
		self.reader.consume(16);
		let mut len_str = Vec::with_capacity(6);
		self.reader.read_until(0xA, &mut len_str).await?;
		let len = parse_len(&len_str)?;
		self.jpeg.resize(len, 0);
		self.reader.read_exact(&mut self.pair).await?;
		self.reader.read_exact(&mut self.jpeg).await?;

		Ok(())
	}

	pub fn take_jpeg(&mut self) -> Vec<u8> {
		mem::take(&mut self.jpeg)
	}
}

fn parse_len(raw: &[u8]) -> Result<usize> {
	str::from_utf8(&raw[..raw.len() - 2])?.parse().map_err(Into::into)
}

#[cfg(todo)]
macro_rules! ready_some_ok {
	($poll:expr) => {
		match ready!($poll) {
			Some(Ok(result)) => result,
			Some(Err(result)) => return Poll::Ready(Some(Err(Error::from(result)))),
			None => return Poll::Ready(None),
		}
	};
}

#[cfg(todo)]
impl futures_core::Stream for MjpegStream {
	type Item = Result<Bytes>;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let this = self.get_mut();
	}
}
