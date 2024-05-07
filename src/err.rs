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
	Recv(#[from] async_channel::RecvError),

	#[error("Timeout exceeded")]
	Timeout,
}
