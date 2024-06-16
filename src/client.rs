use std::io;

use futures_util::TryStreamExt;
use reqwest::{Client, IntoUrl, Url};

use crate::{
	stream::{BoxedStream, MjpegStream},
	AuthMethod, Error, IntoAuthMethod, Result,
};

#[derive(Debug, Clone, Default)]
pub struct MjpegClient(Client);

impl MjpegClient {
	async fn preconnect(&self, url: Url, auth: &AuthMethod) -> Result<()> {
		let req = auth.apply(self.0.get(url));
		let response = req.send().await?;

		if response.status().is_success() {
			Ok(())
		} else {
			Err(Error::NonSuccessStatus(response.status().as_u16()))
		}
	}

	async fn internal_get(&self, url: Url, auth: &AuthMethod) -> Result<MjpegStream> {
		let req = auth.apply(self.0.get(url));
		let stream = req.send().await?.bytes_stream().map_err(stream_map_err);
		let boxed = BoxedStream::from(Box::from(stream));
		Ok(MjpegStream::from_boxed_stream(boxed))
	}

	pub async fn get(&self, url: impl IntoUrl, auth: impl IntoAuthMethod) -> Result<MjpegStream> {
		self.internal_get(url.into_url()?, &auth.into_auth_method()).await
	}

	pub async fn pool(&self, url: impl IntoUrl, auth: impl IntoAuthMethod) -> Result<MjpegPool> {
		let client = self.clone();
		let url = url.into_url()?;
		let auth = auth.into_auth_method();

		client.preconnect(url.clone(), &auth).await?;

		Ok(MjpegPool { client, url, auth })
	}
}

#[inline]
fn stream_map_err(err: reqwest::Error) -> io::Error {
	io::Error::new(io::ErrorKind::Other, err)
}

#[derive(Debug, Clone)]
pub struct MjpegPool {
	pub client: MjpegClient,
	pub url: Url,
	pub auth: AuthMethod,
}

impl MjpegPool {
	pub async fn get(&self) -> Result<MjpegStream> {
		self.client.internal_get(self.url.clone(), &self.auth).await
	}
}
