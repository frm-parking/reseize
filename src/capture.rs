use std::{fmt::Debug, time::Duration};

use ticque::{Customer, Vendor};
use tracing::{debug, error, info};

use crate::{MjpegPool, Result};

#[derive(Debug, Clone)]
pub struct Capture {
	customer: Customer<Vec<u8>>,
}

impl Capture {
	pub fn new(pool: MjpegPool) -> Self {
		let vendor = Vendor::new();
		let customer = vendor.customer();

		let this = Self { customer };

		tokio::spawn(async move {
			loop {
				let mut stream = match pool.get().await {
					Ok(stream) => stream,
					Err(err) => {
						error!("{err}");
						break;
					}
				};
				info!(url = ?pool.url, "Connection to mjpeg stream established");

				loop {
					match stream.read_frame().await {
						Ok(()) => {
							if vendor.has_waiters() {
								vendor.send(stream.take_jpeg());
								debug!("Jpeg frame sent");
							}
						}
						Err(err) => {
							error!("{err}");
							break;
						}
					}
				}

				error!("Mjpeg stream lost. Reconnection in a second");
				tokio::time::sleep(Duration::from_secs(1)).await;
			}
		});

		this
	}

	pub async fn take(&self) -> Result<Vec<u8>> {
		debug!("Waiting for jpeg frame");
		self.customer.request().await.map_err(Into::into)
	}
}
