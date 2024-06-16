use reqwest::RequestBuilder;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum AuthMethod {
	#[default]
	None,
	Basic(String, String),
}

impl AuthMethod {
	pub(crate) fn apply(&self, rb: RequestBuilder) -> RequestBuilder {
		match self {
			Self::None => rb,
			Self::Basic(username, password) => rb.basic_auth(username, Some(password)),
		}
	}
}

trait Sealed {}
impl Sealed for AuthMethod {}
impl<T0, T1> Sealed for BasicAuth<T0, T1> {}
impl<T> Sealed for Option<T> {}

#[allow(private_bounds)]
pub trait IntoAuthMethod: Sealed {
	fn into_auth_method(self) -> AuthMethod;
}

impl IntoAuthMethod for AuthMethod {
	fn into_auth_method(self) -> AuthMethod {
		self
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicAuth<T0, T1>(T0, T1);

impl<T0, T1> IntoAuthMethod for BasicAuth<T0, T1>
where
	T0: Into<String>,
	T1: Into<String>,
{
	fn into_auth_method(self) -> AuthMethod {
		AuthMethod::Basic(self.0.into(), self.1.into())
	}
}

impl<T: IntoAuthMethod> IntoAuthMethod for Option<T> {
	fn into_auth_method(self) -> AuthMethod {
		match self {
			Some(it) => it.into_auth_method(),
			None => AuthMethod::None,
		}
	}
}
