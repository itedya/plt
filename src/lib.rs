mod file_generator;
mod text_code_fsa;

pub mod prelude {
    pub use crate::file_generator::*;
    pub use crate::text_code_fsa::*;
    pub use anyhow::Result;
}
