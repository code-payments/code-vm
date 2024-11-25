pub mod consts;
pub mod instruction;
pub mod state;
pub mod cpis;
pub mod helpers;
pub mod cvm;
pub mod types;
pub mod utils;
pub mod opcode;
pub mod pdas;
pub mod external;

#[cfg(not(target_os = "solana"))]
pub mod sdk;

pub mod prelude {
    pub use crate::consts::*;
    pub use crate::instruction::*;
    pub use crate::state::*; 
    pub use crate::cpis::*;
    pub use crate::helpers::*;
    pub use crate::cvm::*;
    pub use crate::types::*;
    pub use crate::utils::*;
    pub use crate::opcode::*;
    pub use crate::pdas::*;
    pub use crate::external::*;

    #[cfg(not(target_os = "solana"))]
    pub use crate::sdk::*;
}

use steel::*;

declare_id!("vmZ1WUq8SxjBWcaeTCvgJRZbS84R61uniFsQy5YMRTJ"); 
