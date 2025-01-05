macro_rules! error {
	($msg:expr, $span:expr) => {
		panic!("{}",
			::pest::error::Error::<Rule>::new_from_span(
				::pest::error::ErrorVariant::CustomError { message: $msg.to_owned() },
				$span
			)
		)
	};

	($msg:expr, $span:expr, $($var_args:expr),*) => {
		panic!("{}",
			::pest::error::Error::<Rule>::new_from_span(
				::pest::error::ErrorVariant::<Rule>::CustomError { message: format!($msg, $($var_args),*) },
				$span
			)
		)
	};
}

pub(crate) use error;
