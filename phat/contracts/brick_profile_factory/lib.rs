#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

pub use brick_profile_factory::*;

#[ink::contract(env = pink::PinkEnvironment)]
mod brick_profile_factory {
    use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
    use brick_profile::BrickProfileRef;
    use hex_literal::hex;
    use ink::ToAccountId;
    use pink_extension as pink;
    use pink_extension::chain_extension::signing;
    use scale::{Decode, Encode};
    use this_crate::{version_tuple, VersionTuple};

    #[ink(storage)]
    pub struct BrickProfileFactory {
        owner: AccountId,
        profile_code_hash: Hash,
        user_count: u64,
        users: BTreeMap<AccountId, BrickProfileRef>,
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BadOrigin,
        NoDuplicatedUserProfile,
        FailedToCreateProfile(String),
        UserProfileNotExists,
    }
    pub type Result<T> = core::result::Result<T, Error>;

    impl BrickProfileFactory {
        #[ink(constructor)]
        pub fn new(profile_code_hash: Hash) -> Self {
            let caller = Self::env().caller();
            Self {
                owner: caller,
                profile_code_hash,
                user_count: 0,
                users: Default::default(),
            }
        }

        #[ink(message)]
        pub fn version(&self) -> VersionTuple {
            version_tuple!()
        }

        /// Get the owner of the contract, only owner can change the profile contract code hash.
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Get the user profile number.
        #[ink(message)]
        pub fn user_count(&self) -> u64 {
            self.user_count
        }

        /// Get the code hash of the user profile contract.
        #[ink(message)]
        pub fn profile_code_hash(&self) -> Hash {
            self.profile_code_hash
        }

        /// Set the code hash of the user profile contract. Only owner.
        #[ink(message)]
        pub fn set_profile_code_hash(&mut self, profile_code_hash: Hash) -> Result<()> {
            self.ensure_owner()?;
            self.profile_code_hash = profile_code_hash;
            Ok(())
        }

        /// Instantiate a user profile contract for caller. Only once for each account.
        #[ink(message)]
        pub fn create_user_profile(&mut self) -> Result<()> {
            let caller = self.env().caller();

            if self.users.contains_key(&caller) {
                return Err(Error::NoDuplicatedUserProfile);
            }

            let random = signing::derive_sr25519_key(&self.user_count.to_be_bytes());
            let user_profile = BrickProfileRef::new(caller)
                .endowment(0)
                .salt_bytes(&random[..4])
                .code_hash(self.profile_code_hash)
                .try_instantiate()
                .map_err(|e| Error::FailedToCreateProfile(format!("{:?}", e)))?
                .map_err(|e| Error::FailedToCreateProfile(format!("{:?}", e)))?;

            self.users.insert(caller, user_profile);
            self.user_count += 1;

            Ok(())
        }

        /// Get the user profile contract address.
        #[ink(message)]
        pub fn get_user_profile_address(&self) -> Result<AccountId> {
            let caller = self.env().caller();
            let user_profile = self.users.get(&caller).ok_or(Error::UserProfileNotExists)?;
            Ok(user_profile.to_account_id())
        }

        /// Get the user profile contract list.
        #[ink(message)]
        pub fn get_user_profiles(&self) -> Result<Vec<(AccountId, AccountId)>> {
            let profiles = self
                .users
                .clone()
                .into_iter()
                .map(|kv| (kv.0, kv.1.to_account_id()))
                .collect();
            Ok(profiles)
        }

        /// Return BadOrigin error if the caller is not the owner.
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() == self.owner {
                Ok(())
            } else {
                Err(Error::BadOrigin)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        #[ink::test]
        fn it_works() {
            let result = 2 + 2;
            assert_eq!(result, 4);
        }
    }
}
