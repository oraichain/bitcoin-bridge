use crate::{
    bitcoin::{
        adapter::Adapter,
        header_queue::{WorkHeader, WrappedHeader},
    },
    constants::DEFAULT_FEE_RATE,
};

use super::{
    InnerAppV0, InnerAppV1, InnerAppV2, InnerAppV3, InnerAppV4, InnerAppV5, InnerAppV6, InnerAppV7,
};
use bitcoin::{
    util::{uint::Uint256, BitArray},
    BlockHeader,
};
use orga::{
    coins::Take,
    ibc::Ibc,
    migrate::{Migrate, MigrateFrom},
    state::State,
    store::Store,
    upgrade::Upgrade,
    Result,
};

impl MigrateFrom<InnerAppV0> for InnerAppV1 {
    fn migrate_from(_other: InnerAppV0) -> Result<Self> {
        unreachable!()
    }
}

impl MigrateFrom<InnerAppV1> for InnerAppV2 {
    fn migrate_from(_other: InnerAppV1) -> Result<Self> {
        unreachable!()
    }
}

impl MigrateFrom<InnerAppV2> for InnerAppV3 {
    fn migrate_from(other: InnerAppV2) -> Result<Self> {
        Ok(Self {
            accounts: other.accounts,
            staking: other.staking,
            community_pool: other.community_pool,
            staking_rewards: other.staking_rewards,
            community_pool_rewards: other.community_pool_rewards,
            bitcoin: other.bitcoin,
            reward_timer: other.reward_timer,
            #[cfg(feature = "testnet")]
            ibc: other.ibc,
            upgrade: other.upgrade,
            #[cfg(feature = "testnet")]
            cosmos: Default::default(),
        })
    }
}

impl MigrateFrom<InnerAppV3> for InnerAppV4 {
    #[allow(unused_mut)]
    fn migrate_from(mut other: InnerAppV3) -> Result<Self> {
        #[cfg(feature = "testnet")]
        {
            other.upgrade.activation_delay_seconds = 60 * 20;
        }

        #[cfg(not(feature = "testnet"))]
        {
            other.bitcoin.config.min_confirmations = 5;
        }

        Ok(Self {
            accounts: other.accounts,
            staking: other.staking,
            community_pool: other.community_pool,
            staking_rewards: other.staking_rewards,
            community_pool_rewards: other.community_pool_rewards,
            bitcoin: other.bitcoin,
            reward_timer: other.reward_timer,
            upgrade: other.upgrade,

            #[cfg(feature = "testnet")]
            ibc: other.ibc,
            #[cfg(not(feature = "testnet"))]
            ibc: Ibc::default(),

            #[cfg(feature = "testnet")]
            cosmos: other.cosmos,
            #[cfg(not(feature = "testnet"))]
            cosmos: Default::default(),
        })
    }
}

impl MigrateFrom<InnerAppV4> for InnerAppV5 {
    #[allow(unused_mut)]
    fn migrate_from(mut other: InnerAppV4) -> Result<Self> {
        #[cfg(not(feature = "testnet"))]
        {
            other.bitcoin.checkpoints.config.max_age = 60 * 60 * 24 * 30 * 4;
            other.bitcoin.headers.config.max_length = 24_192;

            // remove headers and revert to checkpoint so we can regain history which was pruned
            other
                .bitcoin
                .headers
                .deque
                .retain_unordered(|_| Ok(false))?;
            let checkpoint_json = include_str!("../bitcoin/checkpoint.json");
            let header: (u32, BlockHeader) = serde_json::from_str(checkpoint_json)?;
            let wrapped_header = WrappedHeader::new(Adapter::new(header.1), header.0);
            let work_header = WorkHeader::new(wrapped_header.clone(), wrapped_header.work());
            other.bitcoin.headers.current_work = Adapter::new(work_header.work());
            other.bitcoin.headers.deque.push_back(work_header)?;
        }

        Ok(Self {
            accounts: other.accounts,
            staking: other.staking,
            community_pool: other.community_pool,
            staking_rewards: other.staking_rewards,
            community_pool_rewards: other.community_pool_rewards,
            bitcoin: other.bitcoin,
            reward_timer: other.reward_timer,
            upgrade: other.upgrade,
            ibc: other.ibc,
            cosmos: other.cosmos,
        })
    }
}

impl MigrateFrom<InnerAppV5> for InnerAppV6 {
    #[allow(unused_mut)]
    fn migrate_from(mut other: InnerAppV5) -> Result<Self> {
        let mut checkpoint = other.bitcoin.checkpoints.building_mut()?;
        checkpoint.fee_rate = DEFAULT_FEE_RATE;
        Ok(Self {
            accounts: other.accounts,
            staking: other.staking,
            community_pool: other.community_pool,
            staking_rewards: other.staking_rewards,
            community_pool_rewards: other.community_pool_rewards,
            bitcoin: other.bitcoin,
            reward_timer: other.reward_timer,
            upgrade: other.upgrade,
            ibc: other.ibc,
            cosmos: other.cosmos,
        })
    }
}

impl MigrateFrom<InnerAppV6> for InnerAppV7 {
    #[allow(unused_mut)]
    fn migrate_from(mut other: InnerAppV6) -> Result<Self> {
        let mut checkpoint = other.bitcoin.checkpoints.building_mut()?;
        checkpoint.fee_rate = DEFAULT_FEE_RATE;
        Ok(Self {
            accounts: other.accounts,
            staking: other.staking,
            community_pool: other.community_pool,
            staking_rewards: other.staking_rewards,
            community_pool_rewards: other.community_pool_rewards,
            bitcoin: other.bitcoin,
            reward_timer: other.reward_timer,
            upgrade: other.upgrade,
            ibc: other.ibc,
            cosmos: other.cosmos,
        })
    }
}
