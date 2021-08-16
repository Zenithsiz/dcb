//! Instruction parsing

// Modules
mod error;
pub mod line;

// Exports
pub use error::ParseError;
pub use line::{Line, LineArg, LineArgExpr, LineInst, LineLabel, LineLabelFunc};

// Imports
use crate::Pos;
use int_conv::{Signed, Split};
use std::convert::{TryFrom, TryInto};

/// Instruction parsing
pub trait Parsable<'a>: Sized + 'a {
	/// Parses this instruction
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(mnemonic: &'a str, args: &'a [LineArg], ctx: &Ctx)
		-> Result<Self, ParseError>;
}

/// Parsing context
pub trait ParseCtx<'a> {
	/// Returns the current position
	fn cur_pos(&self) -> Pos;

	/// Returns the position of a label
	fn label_pos(&self, label: &str) -> Option<Pos>;

	/// Evaluates an expression
	fn eval_expr(&self, expr: &LineArgExpr) -> Result<i64, ParseError> {
		match *expr {
			LineArgExpr::Literal(num) => Ok(num),
			LineArgExpr::ByteChar(ch) => Ok(i64::from(ch)),
			LineArgExpr::Label {
				ref label,
				offset,
				ref func,
			} => {
				// Get the label value
				let value: i64 = self.label_pos(label).ok_or(ParseError::UnknownLabel)?.0.into();

				// Then add the offset
				let value = value
					.checked_add(offset.unwrap_or(0))
					.ok_or(ParseError::LiteralOutOfRange)?;

				// And evaluate any function on it
				let value = func.map_or(Ok(value), |func| self.eval_func(value, func))?;

				Ok(value)
			},
		}
	}

	/// Evaluates an expression as `T`, returning `LiteralOutOfBounds` else
	fn eval_expr_as<T: TryFrom<i64>>(&self, expr: &LineArgExpr) -> Result<T, ParseError> {
		self.eval_expr(expr)?
			.try_into()
			.map_err(|_| ParseError::LiteralOutOfRange)
	}

	/// Evaluates a function on a literal
	fn eval_func(&self, value: i64, func: LineLabelFunc) -> Result<i64, ParseError> {
		// Converts a value into a position
		let to_pos = |value: i64| value.try_into().map(Pos).map_err(|_| ParseError::LiteralOutOfRange);

		// Returns the signed lo-hi values of a position
		let signed_addr_lo_hi = |addr: Pos| -> (i16, u16) {
			let (lo, hi) = match addr.0.lo().as_signed() < 0 {
				true => (addr.0.lo(), addr.0.hi().wrapping_add(1)),
				false => addr.0.lo_hi(),
			};

			(lo.as_signed(), hi)
		};

		let value = match func {
			// For address, first get the value as a position to make sure it's within range
			LineLabelFunc::AddrLo => i64::from(signed_addr_lo_hi(to_pos(value)?).0),
			LineLabelFunc::AddrHi => i64::from(signed_addr_lo_hi(to_pos(value)?).1),
		};

		Ok(value)
	}

	/// Retrieves a position from an argument
	fn arg_pos(&self, arg: &LineArg) -> Result<Pos, ParseError> {
		match arg {
			LineArg::Expr(expr) => self
				.eval_expr(expr)?
				.try_into()
				.map(Pos)
				.map_err(|_| ParseError::LiteralOutOfRange),
			_ => Err(ParseError::InvalidArguments),
		}
	}
}
