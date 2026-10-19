use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ReceiverError {
    InvalidConfiguration = 1300,
    UnexpectedBackstop = 1301,
    UnexpectedSwap = 1302,
    BalanceMismatch = 1303,
    ReentrantCall = 1304,
}
