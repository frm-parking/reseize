use std::{
	convert::identity,
	fmt::Debug,
	io::{Error as IoError, ErrorKind as IoErrorKind},
	mem,
	ops::Deref,
	str::FromStr,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

use async_channel::{bounded, Receiver, Sender};
use bytes::Bytes;
use futures_core::Stream;
use futures_util::{stream::MapErr, TryStreamExt};
use reqwest::{IntoUrl, Url};
use tokio::{
	io::{AsyncBufReadExt, AsyncReadExt},
	time::timeout,
};
use tokio_util::io::StreamReader;

pub use crate::err::{Error, Result};

mod err;

#[derive(Debug)]
pub struct Capture {
	url: Url,
	req: Arc<AtomicBool>,
	snap_tx: Sender<Result<Snapshot>>,
	snap_rx: Receiver<Result<Snapshot>>,
	terminate: Arc<AtomicBool>,
}

impl Capture {
	pub async fn start(url: Url) -> Result<Self> {
		let req = Arc::new(AtomicBool::new(false));
		let (snap_tx, snap_rx) = bounded(1);
		let terminate = Arc::new(AtomicBool::new(false));

		let this = Self { req, snap_tx, snap_rx, url, terminate };
		this.spawn().await?;

		Ok(this)
	}

	async fn connect(
		url: impl IntoUrl,
	) -> Result<
		StreamReader<
			MapErr<impl Stream<Item = reqwest::Result<Bytes>> + Sized, impl FnMut(reqwest::Error) -> std::io::Error>,
			Bytes,
		>,
	> {
		let url = url.into_url()?;
		let stream = reqwest::get(url.clone()).await?.bytes_stream().map_err(|err| IoError::new(IoErrorKind::Other, err));
		#[cfg(feature = "tracing")]
		tracing::info!(%url, "Connection established");
		Ok(StreamReader::new(stream))
	}

	async fn spawn(&self) -> Result<()> {
		let mut reader = Self::connect(self.url.clone()).await?;
		let req = self.req.clone();
		let terminate = self.terminate.clone();
		let snap_tx = self.snap_tx.clone();

		tokio::spawn(async move {
			let mut jpeg = Vec::new();
			let mut pair = vec![0; 2];
			let mut header = vec![0; 63];

			loop {
				if terminate.load(Ordering::Relaxed) {
					break;
				}

				let result: Result<()> = timeout(Duration::from_secs(1), async {
					reader.read_exact(&mut header).await?;
					let mut len_str = Vec::with_capacity(6);
					reader.read_until(0xA, &mut len_str).await?;
					let len = parse_len(&len_str)?;
					jpeg.resize(len, 0);
					reader.read_exact(&mut pair).await?;
					reader.read_exact(&mut jpeg).await?;

					Ok(())
				})
				.await
				.map_err(|_| Error::Timeout)
				.and_then(identity);

				if req.load(Ordering::Relaxed) {
					let result = result
						.map_err(|err| {
							#[cfg(feature = "tracing")]
							tracing::error!(%err, "Failed to snapshot");
							err
						})
						.map(|_| Snapshot(mem::take(&mut jpeg)));
					let is_err = result.is_err();
					let _ = snap_tx.send(result).await;
					req.store(false, Ordering::Relaxed);

					if is_err {
						break;
					}
				} else if result.is_err() {
					break;
				}
			}
		});

		Ok(())
	}

	pub async fn snap(&self) -> Result<Snapshot> {
		if self.snap_tx.sender_count() == 1 {
			#[cfg(feature = "tracing")]
			tracing::warn!("Stream has been lost. Reconnecting...");
			while !self.terminate.load(Ordering::Relaxed) {
				if let Err(err) = self.spawn().await {
					#[cfg(feature = "tracing")]
					tracing::warn!(%err, "Waiting for connection");
				} else {
					break;
				}
			}
		}

		self.req.store(true, Ordering::Relaxed);

		match self.snap_rx.recv().await? {
			Ok(snapshot) => {
				#[cfg(feature = "tracing")]
				tracing::info!("Snapshot taken");
				Ok(snapshot)
			}
			Err(_) => Box::pin(self.snap()).await,
		}
	}
}

impl Drop for Capture {
	fn drop(&mut self) {
		self.terminate.store(true, Ordering::Relaxed);
	}
}

fn parse_len(raw: &[u8]) -> Result<usize> {
	let result = usize::from_str(std::str::from_utf8(&raw[..raw.len() - 2])?)?;
	Ok(result)
}

#[derive(Debug)]
pub struct Snapshot(Vec<u8>);

impl Deref for Snapshot {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<Snapshot> for Vec<u8> {
	fn from(value: Snapshot) -> Self {
		value.0
	}
}

#[cfg(feature = "image")]
impl TryFrom<Snapshot> for image::DynamicImage {
	type Error = image::ImageError;

	fn try_from(value: Snapshot) -> Result<Self, Self::Error> {
		image::load_from_memory_with_format(&value.0, image::ImageFormat::Jpeg)
	}
}
