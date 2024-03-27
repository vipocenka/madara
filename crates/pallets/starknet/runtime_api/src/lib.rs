//! Definition of the runtime API for the Starknet pallet.

// Adding allow unused type parameters to avoid clippy errors
// generated by the `decl_runtime_apis` macro.
// Specifically, the macro generates a trait (`StarknetRuntimeApi`) with unused type parameters.
#![allow(clippy::extra_unused_type_parameters)]


use blockifier::context::{BlockContext, FeeTokenAddresses};
use blockifier::execution::contract_class::ContractClass;
use blockifier::transaction::objects::TransactionExecutionInfo;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transaction_execution::Transaction;
use blockifier::transaction::transactions::L1HandlerTransaction;
use mp_felt::Felt252Wrapper;
use sp_api::BlockT;
pub extern crate alloc;
use alloc::vec::Vec;

use mp_contract::ContractAbi;
use mp_simulations::{PlaceHolderErrorTypeForFailedStarknetExecution, SimulationFlags};
use sp_runtime::DispatchError;
use blockifier::context::BlockContext;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::state::StorageKey;
use starknet_api::transaction::{Calldata, Event as StarknetEvent, MessageToL1, TransactionHash};

#[derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
pub enum StarknetTransactionExecutionError {
    ContractNotFound,
    ClassAlreadyDeclared,
    ClassHashNotFound,
    InvalidContractClass,
    ContractError,
}

sp_api::decl_runtime_apis! {
    pub trait StarknetRuntimeApi {
        /// Returns the nonce associated with the given address in the given block
        fn nonce(contract_address: ContractAddress) -> Nonce;
        /// Returns a storage slot value
        fn get_storage_at(address: ContractAddress, key: StorageKey) -> Result<StarkFelt, DispatchError>;
        /// Returns a storage keys and values of a given contract
        fn get_storage_from(address: ContractAddress) -> Result<Vec<(StorageKey, StarkFelt)>, DispatchError>;
        /// Returns a `Call` response.
        fn call(address: ContractAddress, function_selector: EntryPointSelector, calldata: Calldata) -> Result<Vec<Felt252Wrapper>, DispatchError>;
        /// Returns the contract class hash at the given address.
        fn contract_class_hash_by_address(address: ContractAddress) -> ClassHash;
        /// Returns the contract abi for the given class hash
        fn contract_abi_by_class_hash(class_hash: ClassHash) -> Option<ContractAbi>;
        /// Returns the contract class for the given class hash.
        fn contract_class_by_class_hash(class_hash: ClassHash) -> Option<ContractClass>;
        /// Returns the chain id.
        fn chain_id() -> Felt252Wrapper;
        /// Returns the Starknet OS Cairo program hash.
        fn program_hash() -> Felt252Wrapper;
        /// Returns the Starknet config hash.
        fn config_hash() -> StarkHash;
        /// Returns the fee token address.
        fn fee_token_addresses() -> FeeTokenAddresses;
        /// Returns fee estimate
        fn estimate_fee(transactions: Vec<AccountTransaction>) -> Result<Vec<(u128, u128)>, DispatchError>;
        /// Returns message fee estimate
        fn estimate_message_fee(message: L1HandlerTransaction) -> Result<(u128, u128, u128), DispatchError>;
        /// Simulates transactions and returns their trace
        fn simulate_transactions(transactions: Vec<AccountTransaction>, simulation_flags: SimulationFlags) -> Result<Vec<(CommitmentStateDiff, TransactionSimulationResult)>, DispatchError>;

        /// Filters extrinsic transactions to return only Starknet transactions
        ///
        /// To support runtime upgrades, the client must be unaware of the specific extrinsic
        /// details. To achieve this, the client uses an OpaqueExtrinsic type to represent and
        /// manipulate extrinsics. However, the client cannot decode and filter extrinsics due to
        /// this limitation. The solution is to offload decoding and filtering to the RuntimeApi in
        /// the runtime itself, accomplished through the extrinsic_filter method. This enables the
        /// client to operate seamlessly while abstracting the extrinsic complexity.
        fn extrinsic_filter(xts: Vec<<Block as BlockT>::Extrinsic>) -> Vec<Transaction>;
        /// Used to re-execute transactions from a past block and return their trace
        ///
        /// # Arguments
        ///
        /// * `transactions_before` - The first txs of the block. We don't want to trace those, but we need to execute them to rebuild the exact same state
        /// * `transactions_to_trace` - The transactions we want to trace (can be a complete block of transactions or a subset of it)
        ///
        /// # Return
        ///
        /// Idealy, the execution traces of all of `transactions_to_trace`.
        /// If any of the transactions (from both arguments) fails, an error is returned.
        fn re_execute_transactions(transactions: Vec<Transaction>) -> Result<Result<Vec<(TransactionExecutionInfo, CommitmentStateDiff)>, PlaceHolderErrorTypeForFailedStarknetExecution>, DispatchError>;

        fn get_index_and_tx_for_tx_hash(xts: Vec<<Block as BlockT>::Extrinsic>, chain_id: Felt252Wrapper, tx_hash: TransactionHash) -> Option<(u32, Transaction)>;
        /// Returns events, call with index from get_index_and_tx_for_tx_hash method
        fn get_events_for_tx_by_index(tx_index: u32) -> Option<Vec<StarknetEvent>>;

        /// Return the list of StarknetEvent evmitted during this block, along with the hash of the starknet transaction they bellong to
        ///
        /// `block_extrinsics` is the list of all the extrinsic executed during this block, it is used in order to match
        fn get_starknet_events_and_their_associated_tx_index() -> Vec<(u32, StarknetEvent)>;
        /// Return the outcome of the tx execution
        fn get_tx_execution_outcome(tx_hash: TransactionHash) -> Option<Vec<u8>>;
        /// Return the block context
        fn get_block_context() -> BlockContext;
        /// Return is fee disabled in state
        fn is_transaction_fee_disabled() -> bool;
        /// Return messages sent to L1 during tx execution
        fn get_tx_messages_to_l1(tx_hash: TransactionHash) -> Vec<MessageToL1>;
        /// Check if L1 Message Nonce has not been used
        fn l1_nonce_unused(nonce: Nonce) -> bool;
    }

    pub trait ConvertTransactionRuntimeApi {
        /// Converts the transaction to an UncheckedExtrinsic for submission to the pool.
        fn convert_transaction(transaction: AccountTransaction) -> <Block as BlockT>::Extrinsic;

        /// Converts the L1 Message transaction to an UncheckedExtrinsic for submission to the pool.
        fn convert_l1_transaction(transaction: L1HandlerTransaction) -> <Block as BlockT>::Extrinsic;

        /// Converts the DispatchError to an understandable error for the client
        fn convert_error(error: DispatchError) -> StarknetTransactionExecutionError;
    }
}