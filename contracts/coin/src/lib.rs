use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::U128;
use std::convert::TryInto;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, Promise, PromiseOrValue, BorshStorageKey, Timestamp};

mod utils;

pub const ONE_TOKEN: u128 = 1_000_000_000_000_000_000_000_000;
pub const TOTAL_SUPPLY_TOKENS_AMOUNT: u128 = 777_777_777 * ONE_TOKEN;
pub const LP_TOKENS_AMOUNT: u128 = 327_736_777 * ONE_TOKEN;
// 0.075 NEAR - open box price
pub const OPEN_BOX_PRICE: Balance = 75 * ONE_TOKEN / 1000;
pub const BOX_REWARDS: [f32; 5] = [0.0, 0.1, 1.0, 10.0, 1000.0];
pub const PREMIUM_BOXES_PER_ACCOUNT: u32 = 100;
pub const MINT_START_TIMESTAMP: Timestamp = 1704531600000000000; // 2024-01-06 09:00:00 UTC


#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Token,
    TokenMetadata,
    UserNearReward,
    UserLolReward,
    UserTotalBoxOpened,
    UserPremiumBoxOpened,
    UserAdditionalPremium,
}

#[derive(Debug, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct LeaderboardItem {
    pub account_id: AccountId,
    pub amount: U128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    user_near_reward: LookupMap<AccountId, u128>,
    user_lol_reward: LookupMap<AccountId, u128>,
    user_total_box_opened: LookupMap<AccountId, u32>,
    user_premium_box_opened: LookupMap<AccountId, u32>,
    user_additional_premium: LookupMap<AccountId, u32>,
    lol_tokens_remain: u128,
    rewards_remain: Vec<u32>,
    total_box_init: u32,
    total_box_remain: u32,
    total_premium_remain: u32,
    near_leaderboard: Vec<LeaderboardItem>,
    lol_leaderboard: Vec<LeaderboardItem>,
    last_participants: Vec<AccountId>,
    total_participants: u32,
}

const IMAGE_ICON: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAMgAAADICAMAAACahl6sAAAAGXRFWHRTb2Z0d2FyZQBBZG9iZSBJbWFnZVJlYWR5ccllPAAAAyhpVFh0WE1MOmNvbS5hZG9iZS54bXAAAAAAADw/eHBhY2tldCBiZWdpbj0i77u/IiBpZD0iVzVNME1wQ2VoaUh6cmVTek5UY3prYzlkIj8+IDx4OnhtcG1ldGEgeG1sbnM6eD0iYWRvYmU6bnM6bWV0YS8iIHg6eG1wdGs9IkFkb2JlIFhNUCBDb3JlIDcuMi1jMDAwIDc5LjFiNjVhNzliNCwgMjAyMi8wNi8xMy0yMjowMTowMSAgICAgICAgIj4gPHJkZjpSREYgeG1sbnM6cmRmPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5LzAyLzIyLXJkZi1zeW50YXgtbnMjIj4gPHJkZjpEZXNjcmlwdGlvbiByZGY6YWJvdXQ9IiIgeG1sbnM6eG1wPSJodHRwOi8vbnMuYWRvYmUuY29tL3hhcC8xLjAvIiB4bWxuczp4bXBNTT0iaHR0cDovL25zLmFkb2JlLmNvbS94YXAvMS4wL21tLyIgeG1sbnM6c3RSZWY9Imh0dHA6Ly9ucy5hZG9iZS5jb20veGFwLzEuMC9zVHlwZS9SZXNvdXJjZVJlZiMiIHhtcDpDcmVhdG9yVG9vbD0iQWRvYmUgUGhvdG9zaG9wIDIzLjUgKE1hY2ludG9zaCkiIHhtcE1NOkluc3RhbmNlSUQ9InhtcC5paWQ6QzQ4RjI0ODNBMzVGMTFFRTk0MTFFRkQ2NkQ0MzMwQkIiIHhtcE1NOkRvY3VtZW50SUQ9InhtcC5kaWQ6QzQ4RjI0ODRBMzVGMTFFRTk0MTFFRkQ2NkQ0MzMwQkIiPiA8eG1wTU06RGVyaXZlZEZyb20gc3RSZWY6aW5zdGFuY2VJRD0ieG1wLmlpZDpDNDhGMjQ4MUEzNUYxMUVFOTQxMUVGRDY2RDQzMzBCQiIgc3RSZWY6ZG9jdW1lbnRJRD0ieG1wLmRpZDpDNDhGMjQ4MkEzNUYxMUVFOTQxMUVGRDY2RDQzMzBCQiIvPiA8L3JkZjpEZXNjcmlwdGlvbj4gPC9yZGY6UkRGPiA8L3g6eG1wbWV0YT4gPD94cGFja2V0IGVuZD0iciI/PhJVqaYAAAMAUExURQAAAOXl5by8vA0NDcbGxsTExC0tLZCQkLCwsK6urszMzCYmJdfX193d3bi4uFpaWT4+PTY2NcLCwuHh4djY2MDAwJKSkgkJCQUFBaqqqhERER4eHTo6Oaampt7e3l5eXYyMjCopKSIiIaKioqCgoHd2dhoaGbq6uqioqIiIiJaWlrKyspiYmI6OjouKihUVFX18fHl4eFJSUmhoaJycnGJiYkRERGRkZJSUlM/Pz0xMTElJSIaGhk5OTlRUVEBAQH5+fqWlpIKCgsjIyEJCQmFgYG9ublBQUGxsbDAwMKysrP///2dmZnJycnR0dDMyMmtranBwcICAgEpKSoSEhFdWVgICAlxbW2BgX0dGRkZGRUVFRVxcWzU0Mzg3N3Bvb0BAPjw8PBAQEDExMS8vLxcXFx8fHtra2re3t7+/v5ubm3t7ewEBAfz8/Pn5+fv7+/Ly8vT09MvLy/Pz8+3t7eLi4vf39+fn5/39/bS0tOvr6/r6+vHx8QgICNLS0u7u7u/v79XV1fX19dTU1La2tv7+/rW1tZ+fn+zs7AcHB3p6evb29p6enunp6Q8PD/j4+NHR0fDw8NPT09DQ0Ojo6Orq6iAgHwwMDJqamuPj48rKyk9PTywsK+bm5nBwb4eHh4iIh76+vgEBANvb25eXl5aWlWlpaWNjY5+fnjMzM9TT00REQ3p5eTs7Ozg4OCcnJ4mJiWVlZRwcGxMTExQUFHJycUNDQ4aFhQIBAVZWVZuamp+enlRUU4+Pj8rKyW5ubdHR0Hx7e4uLi6urq+Dg3wMDA6Cfn0hHR5GRkQsLC319fSsqKp6encHBwV9fXyMjIn9/f5ycm8nJyUxLS52dncDAv6mpqQICASQkJFhYWHZ2dYODg6enp/Dw75WVlXt6ehkYGHt7enx8e3JxcaOjo5mZmWpqaWFhYWJiYcPDw1FRUVJSUU1NTU1MTNXU1IGBgaCgn0FBQQQEBEJCQUhIR21tbZOTk9PS0gIBAjIyMWloZ1VVVUtKSrOzswEAAP///95SPzwAAAEAdFJOU////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////wBT9wclAAANsUlEQVR42tSdeUATVx7HHwmGSDgVnWmkiqh4trRYpZaqa9fWbaO2btt4IHYNiEcD4b4kHIJoUONtrNrSU0urokDtpeuumF0ia2Ft11VxXbbFurp0u2SFXdd203BoZjKTzLzJSzLz/ZP5veMT5s17v9+7gMU9Sh+8r2Lzmd8eDN7/XVx8/O3vpl+ee7hqzUTf4YVuKhAgz7FkpzTihf2hsmgDpsPNRqUaWKVWGs04jhnqmp//bm7VA/4F/AbJ+eDqlR9qVBiuBE70ZxzTN099ZoN/Ji9BNAGfxybIMSNgKaVWLxlxbdpWfoGkiV4Nqdf+F0BKrVMlfJSfxReQ9KBXJGIz4CijSXZ8YIn3QfIyKhMNSuCSlKaa00NSvQqy8f0lcjNAIKN4eWO210ACKmUYQCbdsBcueAXE9+8rzACpjPqTQSmeBjl/V68EyKUWL5ZqPAnie1evBm7R24apFR4D+efTKjdh9Ep8y9cjIFsq24zArVKr5ga6HSTld004cLuMyWfS3QsybboJeETaEJEbQTa9uUINPCXVNxvdBZIxWws8KDxxlVtANBFtauBZqcakoQc5exIDHhfeOQ01SMVII/CC1LXvIwXJe1MFvCTxG+noQIpvYMBrwn8YjgokrNMMvCilZAAakCEjlcC7ihqHAqQhCnhdqkbXQa6rAA8kPucqSIQe8ELYNxqXQFoNgCfSHlzrAkgLbzisJHNTOYM8xSMOK8nktRxBxvOKw0ryDDeQCXrAM2GHuYDMkgPeyXAMHmRsG+Ch9J/DghTJAC8VHQQHkhui5icIkBXBgGiWmnnKAdSzsyBAWjDAW+Gx7EEa9IDHMkWwBSmVAV6rXsQOJKdDyW8QkPA6K5DvtTznAOZYNiAiOeC9DBuYQYqPAAHoUhgjyGFcCCDGhUwgQnixel+u0c5B1p0AAtGBbKcgD2FCAcGfcQYSFgkEI/lqJyDfmoUDon5C4xBkiAoISKZ2hyAxaiGBgGUlDkCkYkFxAK0fPUjqEiAw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";

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default_meta(owner_id: AccountId) -> Self {
        Self::new(
            owner_id,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "LOL Memecoin".to_string(),
                symbol: "LOL".to_string(),
                icon: Some(IMAGE_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        )
    }

    #[init]
    pub fn new(
        owner_id: AccountId,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();

        let rewards_remain_init: Vec<u32> = vec![44449, 5000, 500, 50, 1];
        let lol_tokens_for_boxes = TOTAL_SUPPLY_TOKENS_AMOUNT - LP_TOKENS_AMOUNT;

        let mut this = Self {
            owner_id,
            token: FungibleToken::new(StorageKeys::Token),
            metadata: LazyOption::new(StorageKeys::TokenMetadata, Some(&metadata)),
            user_near_reward: LookupMap::new(StorageKeys::UserNearReward),
            user_lol_reward: LookupMap::new(StorageKeys::UserLolReward),
            user_total_box_opened: LookupMap::new(StorageKeys::UserTotalBoxOpened),
            user_premium_box_opened: LookupMap::new(StorageKeys::UserPremiumBoxOpened),
            user_additional_premium: LookupMap::new(StorageKeys::UserAdditionalPremium),
            lol_tokens_remain: lol_tokens_for_boxes,
            rewards_remain: rewards_remain_init.clone(),
            total_box_init: rewards_remain_init.iter().sum(),
            total_box_remain: rewards_remain_init.iter().sum(),
            total_premium_remain: rewards_remain_init[1] + rewards_remain_init[2] + rewards_remain_init[3] + rewards_remain_init[4],
            near_leaderboard: vec![],
            lol_leaderboard: vec![],
            last_participants: vec![],
            total_participants: 0,
        };

        // Mint LOL tokens for box rewards
        let current_contract = env::current_account_id();
        this.token.internal_register_account(&current_contract);
        this.token.internal_deposit(&current_contract, lol_tokens_for_boxes);
        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &current_contract,
            amount: &lol_tokens_for_boxes.into(),
            memo: Some("Initial tokens supply is minted"),
        }.emit();

        // Mint LOL tokens for LP
        let lp_contract: AccountId = format!("{}.{}", "liquidity", env::current_account_id()).try_into().unwrap();
        this.token.internal_register_account(&lp_contract);
        this.token.internal_deposit(&lp_contract, LP_TOKENS_AMOUNT);
        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &lp_contract,
            amount: &LP_TOKENS_AMOUNT.into(),
            memo: Some("LP tokens supply is minted"),
        }.emit();

        // Register burn contract
        let burn_contract: AccountId = format!("{}.{}", "burn", env::current_account_id()).try_into().unwrap();
        this.token.internal_register_account(&burn_contract);

        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    #[payable]
    pub fn open_box(&mut self) -> (usize, U128, U128) {
        if env::attached_deposit() < OPEN_BOX_PRICE {
            env::panic_str("Error: Wrong open deposit");
        }
        if self.total_box_remain == 0 {
            env::panic_str("Error: No boxes remains");
        }
        if env::block_timestamp() < MINT_START_TIMESTAMP {
            env::panic_str("Error: Too early to open boxes");
        }

        let owner_id = env::predecessor_account_id();
        let user_additional_premium: u32 = self.user_additional_premium.get(&owner_id).unwrap_or(0);
        let user_premium_box_opened: u32 = self.user_premium_box_opened.get(&owner_id).unwrap_or(0);
        let user_total_box_opened: u32 = self.user_total_box_opened.get(&owner_id).unwrap_or(0);

        // Add Token Storage
        if self.ft_balance_of(env::predecessor_account_id()) == U128(0) && user_total_box_opened == 0 {
            self.token.internal_register_account(&owner_id);
        }

        let mut can_get_premium: bool = false;
        if self.total_premium_remain > 0 && user_premium_box_opened < PREMIUM_BOXES_PER_ACCOUNT + user_additional_premium {
            can_get_premium = true;
        }

        self.user_total_box_opened.remove(&owner_id);
        self.user_total_box_opened.insert(&owner_id, &(user_total_box_opened + 1));
        self.total_box_remain -= 1;

        let reward_type_index = self._get_random_user_reward(can_get_premium);
        let is_premium_box = reward_type_index != 0;

        self.rewards_remain[reward_type_index] -= 1;

        // if owner_id not in self.last_participants - add to self.last_participants and increase self.total_participants
        if !self.last_participants.contains(&owner_id) {
            self.last_participants.push(owner_id.clone());
            self.total_participants += 1;

            // if self.total_participants > 500 - remove first element from self.last_participants
            if self.total_participants > 500 {
                self.last_participants.remove(0);
            }
        }

        let lol_reward = self._claim_lol_reward(&owner_id, is_premium_box);
        let mut near_reward = 0;
        if is_premium_box {
            near_reward = self._get_near_reward_amount(reward_type_index);
            self._claim_near_reward(owner_id.clone(), near_reward);
        }

        env::log_str(&format!("Reward: {}, {}, {}, {}", owner_id, reward_type_index, lol_reward, near_reward));
        (reward_type_index, lol_reward.into(), near_reward.into())
    }

    pub fn get_user_rewards(&self, owner_id: AccountId) -> (u128, u128, u32) {
        let user_near_reward = self.user_near_reward.get(&owner_id).unwrap_or(0);
        let user_lol_reward = self.user_lol_reward.get(&owner_id).unwrap_or(0);
        let total_box_opened = self.user_total_box_opened.get(&owner_id).unwrap_or(0);

        (user_lol_reward, user_near_reward, total_box_opened)
    }

    pub fn get_total_stats(&self) -> (u32, Vec<u32>, u32, u32, Vec<u128>, Vec<f32>, u64) {
        let remains = self.rewards_remain.clone();
        let total_box_remain = self.total_box_remain;
        let lol_tokens_remain: Vec<u128> = vec![TOTAL_SUPPLY_TOKENS_AMOUNT, LP_TOKENS_AMOUNT, self.lol_tokens_remain];
        let total_participants = self.total_participants;
        let total_box_init = self.total_box_init;

        (
            total_participants,
            remains,
            total_box_remain,
            total_box_init,
            lol_tokens_remain,
            BOX_REWARDS.into(),
            MINT_START_TIMESTAMP.into(),
        )
    }

    pub fn get_all_participants(&self) -> Vec<LeaderboardItem> {
        let mut participants: Vec<LeaderboardItem> = vec![];
        self.last_participants.iter().for_each(|account_id| {
            let user_lol_reward = self.user_lol_reward.get(&account_id).unwrap_or(0);
            participants.push(LeaderboardItem {
                account_id: account_id.clone(),
                amount: user_lol_reward.into(),
            });
        });

        participants
    }

    pub fn get_leaderboards(&self) -> (&Vec<LeaderboardItem>, &Vec<LeaderboardItem>) {
        (&self.near_leaderboard, &self.lol_leaderboard)
    }

    pub fn user_premium_boxes_left(&self, account_id: AccountId) -> u32 {
        let user_additional_premium: u32 = self.user_additional_premium.get(&account_id).unwrap_or(0);
        let user_premium_box_opened: u32 = self.user_premium_box_opened.get(&account_id).unwrap_or(0);
        PREMIUM_BOXES_PER_ACCOUNT + user_additional_premium - user_premium_box_opened
    }

    // -------------- Admin functions --------------

    pub fn add_additional_premium(&mut self, account_id: AccountId, amount: u32) -> u32 {
        if env::predecessor_account_id() != self.owner_id {
            env::panic_str("Error: only owner can call this method");
        }

        let user_additional_premium: u32 = self.user_additional_premium.get(&account_id).unwrap_or(0);
        let new_amount: u32 = user_additional_premium + amount;
        self.user_additional_premium.remove(&account_id);
        self.user_additional_premium.insert(&account_id, &new_amount);

        new_amount
    }

    // DANGEROUS: Free NEAR tokens by storage cleanup
    pub fn cleanup_storage_erase_data(&mut self) {
        if env::predecessor_account_id() != self.owner_id {
            env::panic_str("Error: only owner can call this method");
        }

        self.last_participants = vec![];
        self.rewards_remain = vec![];
        self.near_leaderboard = vec![];
        self.lol_leaderboard = vec![];
    }

    // -------------- Private functions --------------

    fn _get_random_user_reward(&self, can_get_premium: bool) -> usize {
        let mut result: u32 = 0;
        let base_box_count: usize = (self.total_box_remain - self.rewards_remain[0] / 2) as usize;
        let rand_val = self.random_in_range(1, base_box_count);

        if can_get_premium {
            if rand_val < self.rewards_remain[4] {
                result = 4;
            } else if rand_val < self.rewards_remain[4] + self.rewards_remain[3] {
                result = 3;
            } else if rand_val < self.rewards_remain[4] + self.rewards_remain[3] + self.rewards_remain[2] {
                result = 2;
            } else if rand_val < self.rewards_remain[4] + self.rewards_remain[3] + self.rewards_remain[2] + self.rewards_remain[1] {
                result = 1;
            }
        }

        usize::try_from(result).unwrap()
    }

    fn _get_near_reward_amount(&self, reward_type_index: usize) -> u128 {
        match reward_type_index {
            1 => { ONE_TOKEN / 10 }
            2 => { ONE_TOKEN }
            3 => { 10 * ONE_TOKEN }
            4 => { 1000 * ONE_TOKEN }
            _ => { 0 }
        }
    }

    fn _claim_near_reward(&mut self, owner_id: AccountId, near_amount: u128) {
        let user_premium_box_opened: u32 = self.user_premium_box_opened.get(&owner_id).unwrap_or(0);
        self.user_premium_box_opened.remove(&owner_id);
        self.user_premium_box_opened.insert(&owner_id, &(user_premium_box_opened + 1));
        self.total_premium_remain -= 1;

        let user_near_reward = self.user_near_reward.get(&owner_id).unwrap_or(0);
        self.user_near_reward.remove(&owner_id);
        self.user_near_reward.insert(&owner_id, &(user_near_reward + near_amount));

        // Update leaderboard
        self._update_leaderboard("near_leaderboard", &owner_id, user_near_reward + near_amount);

        // Transfer tokens
        Promise::new(owner_id).transfer(near_amount);
    }

    fn _claim_lol_reward(&mut self, owner_id: &AccountId, is_premium_box: bool) -> u128 {
        let min: usize = if is_premium_box { 100 } else { 1000 };
        let max: usize = if is_premium_box { 1000 } else { 10000 };
        let lol_amount: u128 = (self.random_in_range(0, max - min) + min as u32) as u128 * ONE_TOKEN;

        let user_lol_reward = self.user_lol_reward.get(&owner_id).unwrap_or(0);
        self.user_lol_reward.remove(&owner_id);
        self.user_lol_reward.insert(&owner_id, &(user_lol_reward + lol_amount));

        if self.lol_tokens_remain >= lol_amount {
            // Update leaderboard
            self._update_leaderboard("lol_leaderboard", &owner_id, user_lol_reward + lol_amount);

            // Transfer tokens
            self.lol_tokens_remain -= lol_amount;
            self.token.internal_transfer(&env::current_account_id(), &owner_id, lol_amount.into(), None);
        }

        return lol_amount;
    }

    fn _update_leaderboard(&mut self, leaderboard_type: &str, owner_id: &AccountId, amount: u128) {
        let mut is_updated: bool = false;
        let mut index: usize = 0;

        let leaderboard = match leaderboard_type {
            "near_leaderboard" => &mut self.near_leaderboard,
            "lol_leaderboard" => &mut self.lol_leaderboard,
            _ => panic!("Error: wrong leaderboard type"),
        };

        leaderboard.iter().for_each(|item| {
            if &item.account_id == owner_id {
                is_updated = true;
                return;
            }
            index += 1;
        });

        if is_updated {
            leaderboard[index].amount = amount.into();
        } else {
            leaderboard.push(LeaderboardItem {
                account_id: owner_id.clone(),
                amount: amount.into(),
            });
        }

        leaderboard.sort_by(|a, b| b.amount.cmp(&a.amount));
        if leaderboard.len() > 10 {
            leaderboard.pop();
        }
    }
}

// near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[near_bindgen]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) {
        // Left 1% fees
        let transfer_amount = amount.0 * 99 / 100;
        let fee_amount = amount.0 - transfer_amount;

        let burn_contract: AccountId = format!("{}.{}", "burn", env::current_account_id()).try_into().unwrap();
        self.token.internal_transfer(&env::current_account_id(), &burn_contract, fee_amount, None);

        self.token.ft_transfer(receiver_id, transfer_amount.into(), memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // Left 1% fees
        let transfer_amount = amount.0 * 99 / 100;
        let fee_amount = amount.0 - transfer_amount;

        let burn_contract: AccountId = format!("{}.{}", "burn", env::current_account_id()).try_into().unwrap();
        self.token.internal_transfer(&env::current_account_id(), &burn_contract, fee_amount, None);

        self.token.ft_transfer_call(receiver_id, transfer_amount.into(), memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}
