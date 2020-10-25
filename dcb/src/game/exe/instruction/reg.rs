//! Cpu registers

// Macro to generate `Register`
macro_rules! generate_register {
	(
		pub enum Register {
			$(
				$( #[doc = $doc:literal] )?
				#[display(fmt = $fmt:literal)]
				$variant:ident = $value:expr
			),+ $(,)?
		}
	) => {
		#[derive(PartialEq, Eq, Clone, Copy, Debug)]
		#[derive(derive_more::Display)]
		pub enum Register {
			$(
				$( #[doc = $doc] )?
				#[display(fmt = $fmt)]
				$variant = $value,
			)*
		}

		impl Register {
			/// Array containing all registers
			pub const ALL_REGISTERS: [Self; 32] = [
				$(
					Self::$variant,
				)*
			];

			/// Creates a new register index from a `u8`.
			#[must_use]
			pub const fn new(idx: u8) -> Option<Self> {
				match idx {
					$(
						$value => Some( Self::$variant ),
					)*

					_ => None,
				}
			}
		}

		impl From<Register> for usize {
			fn from(idx: Register) -> Self {
				match idx {
					$(
						Register::$variant => $value,
					)*
				}
			}
		}

		impl std::str::FromStr for Register {
			type Err = ();

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				match s.trim() {
					$(
						$fmt => Ok(Self::$variant),
					)*

					_ => Err(())
				}
			}
		}
	}
}

generate_register! {
	pub enum Register {
		/// Zero register
		#[display(fmt = "$zr")]
		Zr = 0,

		/// Assembler temporary
		#[display(fmt = "$at")]
		At = 1,

		// Return values
		#[display(fmt = "$v0")] V0 = 2,
		#[display(fmt = "$v1")] V1 = 3,

		// Arguments
		#[display(fmt = "$a0")] A0 = 4,
		#[display(fmt = "$a1")] A1 = 5,
		#[display(fmt = "$a2")] A2 = 6,
		#[display(fmt = "$a3")] A3 = 7,

		// Temporaries
		#[display(fmt = "$t0")] T0 = 8,
		#[display(fmt = "$t1")] T1 = 9,
		#[display(fmt = "$t2")] T2 = 10,
		#[display(fmt = "$t3")] T3 = 11,
		#[display(fmt = "$t4")] T4 = 12,
		#[display(fmt = "$t5")] T5 = 13,
		#[display(fmt = "$t6")] T6 = 14,
		#[display(fmt = "$t7")] T7 = 15,

		// Static variables
		#[display(fmt = "$s0")] S0 = 16,
		#[display(fmt = "$s1")] S1 = 17,
		#[display(fmt = "$s2")] S2 = 18,
		#[display(fmt = "$s3")] S3 = 19,
		#[display(fmt = "$s4")] S4 = 20,
		#[display(fmt = "$s5")] S5 = 21,
		#[display(fmt = "$s6")] S6 = 22,
		#[display(fmt = "$s7")] S7 = 23,

		// Temporaries
		#[display(fmt = "$t8")] T8 = 24,
		#[display(fmt = "$t9")] T9 = 25,

		// Kernel
		#[display(fmt = "$k0")] K0 = 26,
		#[display(fmt = "$k1")] K1 = 27,

		/// Global pointer
		#[display(fmt = "$gp")]
		Gp = 28,

		/// Stack pointer
		#[display(fmt = "$sp")]
		Sp = 29,

		/// Frame pointer
		#[display(fmt = "$fp")]
		Fp = 30,

		/// Return address
		#[display(fmt = "$ra")]
		Ra = 31,
	}
}
