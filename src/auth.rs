use std::fmt::Display;

use reqwest::RequestBuilder;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Auth<U = String, P = String> {
	#[default]
	None,
	Basic(U, P),
}

impl<U, P> Auth<U, P>
where
	U: Display,
	P: Display,
{
	pub(crate) fn apply(&self, rb: RequestBuilder) -> RequestBuilder {
		match self {
			Auth::None => rb,
			Auth::Basic(username, password) => rb.basic_auth(username, Some(password)),
		}
	}

	pub(crate) fn into_strings(self) -> Auth<String, String> {
		match self {
			Auth::None => Auth::None,
			Auth::Basic(a, b) => Auth::Basic(a.to_string(), b.to_string()),
		}
	}
}
