use soroban_sdk::contracterror;

/// All error variants the did-registry contract can return.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    /// The requested DID or credential does not exist.
    NotFound = 1,
    /// The caller is not authorised to perform this action.
    Unauthorized = 2,
    /// A DID or credential with this identifier already exists.
    AlreadyExists = 3,
    /// The credential has passed its expiry timestamp.
    Expired = 4,
}
