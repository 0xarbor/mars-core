use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Binary, CanonicalAddr, Decimal, Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

// keys (for singleton)
pub static CONFIG_KEY: &[u8] = b"config";
pub static BASECAMP_KEY: &[u8] = b"basecamp";

// namespaces (for buckets)
pub static PROPOSALS_NAMESPACE: &[u8] = b"proposals";
pub static PROPOSAL_VOTES_NAMESPACE: &[u8] = b"proposal_votes";

/// Basecamp global configuration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Contract owner
    pub owner: CanonicalAddr,
    /// Mars token address
    pub mars_token_address: CanonicalAddr,
    /// xMars token address
    pub xmars_token_address: CanonicalAddr,
    /// Staking contract address
    pub staking_contract_address: CanonicalAddr,

    /// Blocks during which a proposal is active since being submitted
    pub proposal_voting_period: u64,
    /// Blocks that need to pass since a proposal succeeds in order for it to be available to be
    /// executed
    pub proposal_effective_delay: u64,
    /// Blocks after the effective_delay during which a successful proposal can be activated before it expires
    pub proposal_expiration_period: u64,
    /// Number of Mars needed to make a proposal. Will be returned if successful. Will be
    /// distributed between stakers if proposal is not executed.
    pub proposal_required_deposit: Uint128,
    /// % of total voting power required to participate in the proposal in order to consider it successfull
    pub proposal_required_quorum: Decimal,
    /// % of for votes required in order to consider the proposal successful
    pub proposal_required_threshold: Decimal,
}

pub fn config_state<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_state_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

/// Basecamp global state
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Basecamp {
    /// Number of proposals
    pub proposal_count: u64,
}

pub fn basecamp_state<S: Storage>(storage: &mut S) -> Singleton<S, Basecamp> {
    singleton(storage, BASECAMP_KEY)
}

pub fn basecamp_state_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Basecamp> {
    singleton_read(storage, BASECAMP_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Proposal {
    pub submitter_canonical_address: CanonicalAddr,
    pub status: ProposalStatus,
    pub for_votes: Uint128,
    pub against_votes: Uint128,
    pub start_height: u64,
    pub end_height: u64,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub execute_calls: Option<Vec<ProposalExecuteCall>>,
    pub deposit_amount: Uint128,
}

pub fn proposals_state<S: Storage>(storage: &mut S) -> Bucket<S, Proposal> {
    bucket(PROPOSALS_NAMESPACE, storage)
}

pub fn proposals_state_read<S: Storage>(storage: &S) -> ReadonlyBucket<S, Proposal> {
    bucket_read(PROPOSALS_NAMESPACE, storage)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

/// Execute call that will be done by the DAO if the proposal succeeds. As this is persisted,
/// the contract canonical address is stored (vs the human address when the proposal submit message is
/// sent)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProposalExecuteCall {
    pub execution_order: u64,
    pub target_contract_canonical_address: CanonicalAddr,
    pub msg: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProposalVote {
    pub option: ProposalVoteOption,
    pub power: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposalVoteOption {
    For,
    Against,
}

impl std::fmt::Display for ProposalVoteOption {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display_str = match self {
            ProposalVoteOption::For => "for",
            ProposalVoteOption::Against => "against",
        };
        write!(f, "{}", display_str)
    }
}

pub fn proposal_votes_state<S: Storage>(
    storage: &mut S,
    proposal_id: u64,
) -> Bucket<S, ProposalVote> {
    Bucket::multilevel(
        &[PROPOSAL_VOTES_NAMESPACE, &proposal_id.to_be_bytes()],
        storage,
    )
}

pub fn proposal_votes_state_read<S: Storage>(
    storage: &S,
    proposal_id: u64,
) -> ReadonlyBucket<S, ProposalVote> {
    ReadonlyBucket::multilevel(
        &[PROPOSAL_VOTES_NAMESPACE, &proposal_id.to_be_bytes()],
        storage,
    )
}
