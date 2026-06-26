#![no_std]
#![allow(unexpected_cfgs)]

#[cfg(test)]
extern crate std;

pub mod admin;
pub mod allowance;
pub mod balance;
pub mod batch;
pub mod dispute;
pub mod escrow;
pub mod freeze;
pub mod metadata;
pub mod pause;
pub mod recurring;
pub mod snapshot;
pub mod splitter;
pub mod storage_types;
pub mod validation;

mod contract;

#[cfg(test)]
mod test;

#[cfg(test)]
mod event_test;

#[cfg(test)]
mod balance_test;

#[cfg(test)]
mod allowance_test;

#[cfg(test)]
mod escrow_test;

#[cfg(test)]
mod admin_test;

#[cfg(test)]
mod splitter_test;

#[cfg(test)]
mod batch_test;

#[cfg(test)]
mod dispute_test;

#[cfg(test)]
mod pause_test;

#[cfg(test)]
mod recurring_test;

#[cfg(test)]
mod freeze_test;

#[cfg(test)]
mod contract_test;

pub use crate::contract::VeritixToken;
