use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
	#[error(transparent)]
	Io(#[from] std::io::Error),

	#[error(transparent)]
	Reqwest(#[from] reqwest::Error),

	#[error(transparent)]
	Utf8(#[from] std::str::Utf8Error),

	#[error(transparent)]
	ParseInt(#[from] std::num::ParseIntError),

	#[error(transparent)]
	Request(#[from] ticque::RequestError),

	#[error("Timeout exceeded")]
	Timeout,
	
	#[error("Failed to preconnect to stream. Status code: {0}")]
	NonSuccessStatus(u16)
}
