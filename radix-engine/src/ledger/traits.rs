use sbor::*;
use scrypto::buffer::*;
use scrypto::engine::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::model::*;

const XRD_SYMBOL: &str = "XRD";
const XRD_NAME: &str = "Radix";
const XRD_DESCRIPTION: &str = "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.";
const XRD_URL: &str = "https://tokens.radixdlt.com";
const XRD_MAX_SUPPLY: i128 = 24_000_000_000_000i128;
const XRD_VAULT_ID: Vid = Vid(H256([0u8; 32]), 0);

const SYSTEM_COMPONENT_NAME: &str = "System";

#[derive(TypeId, Encode, Decode)]
struct SystemComponentState {
    xrd: Vid,
}

pub trait QueryableSubstateStore {
    fn get_lazy_map_entries(
        &self,
        component_address: &Address,
        mid: &Mid,
    ) -> HashMap<Vec<u8>, Vec<u8>>;
}

/// A ledger stores all transactions and substates.
pub trait SubstateStore {
    /// Top Level Objects
    fn get_resource_def(&self, address: Address) -> Option<ResourceDef>;
    fn put_resource_def(&mut self, address: Address, resource_def: ResourceDef);
    fn get_package(&self, address: Address) -> Option<Package>;
    fn put_package(&mut self, address: Address, package: Package);
    fn get_component(&self, address: Address) -> Option<Component>;
    fn put_component(&mut self, address: Address, component: Component);

    /// Child Objects
    fn get_lazy_map_entry(
        &self,
        component_address: &Address,
        mid: &Mid,
        key: &[u8],
    ) -> Option<Vec<u8>>;
    fn put_lazy_map_entry(
        &mut self,
        component_address: Address,
        mid: Mid,
        key: Vec<u8>,
        value: Vec<u8>,
    );
    fn get_vault(&self, component_address: &Address, vid: &Vid) -> Vault;
    fn put_vault(&mut self, component_address: Address, vid: Vid, vault: Vault);
    fn get_non_fungible(
        &self,
        resource_address: Address,
        id: &NonFungibleKey,
    ) -> Option<NonFungible>;
    fn put_non_fungible(
        &mut self,
        resource_address: Address,
        id: &NonFungibleKey,
        non_fungible: NonFungible,
    );

    fn bootstrap(&mut self) {
        if self.get_package(SYSTEM_PACKAGE).is_none() {
            // System package
            self.put_package(
                SYSTEM_PACKAGE,
                Package::new(include_bytes!("../../../assets/system.wasm").to_vec()),
            );

            // Account package
            self.put_package(
                ACCOUNT_PACKAGE,
                Package::new(include_bytes!("../../../assets/account.wasm").to_vec()),
            );

            // Radix token resource definition
            let mut metadata = HashMap::new();
            metadata.insert("symbol".to_owned(), XRD_SYMBOL.to_owned());
            metadata.insert("name".to_owned(), XRD_NAME.to_owned());
            metadata.insert("description".to_owned(), XRD_DESCRIPTION.to_owned());
            metadata.insert("url".to_owned(), XRD_URL.to_owned());
            self.put_resource_def(
                RADIX_TOKEN,
                ResourceDef::new(
                    ResourceType::Fungible { divisibility: 18 },
                    metadata,
                    0,
                    0,
                    HashMap::new(),
                    &Some(NewSupply::Fungible {
                        amount: XRD_MAX_SUPPLY.into(),
                    }),
                )
                .unwrap(),
            );

            self.put_resource_def(
                ECDSA_TOKEN,
                ResourceDef::new(
                    ResourceType::NonFungible,
                    HashMap::new(),
                    0,
                    0,
                    HashMap::new(),
                    &None,
                )
                .unwrap(),
            );

            // Instantiate system component
            self.put_vault(
                SYSTEM_COMPONENT,
                XRD_VAULT_ID,
                Vault::new(Bucket::new(
                    RADIX_TOKEN,
                    ResourceType::Fungible { divisibility: 18 },
                    Supply::Fungible {
                        amount: XRD_MAX_SUPPLY.into(),
                    },
                )),
            );
            self.put_component(
                SYSTEM_COMPONENT,
                Component::new(
                    SYSTEM_PACKAGE,
                    SYSTEM_COMPONENT_NAME.to_owned(),
                    scrypto_encode(&SystemComponentState { xrd: XRD_VAULT_ID }),
                ),
            );
        }
    }

    fn get_epoch(&self) -> u64;

    fn set_epoch(&mut self, epoch: u64);

    // Before transaction hash is defined, we use the following TEMPORARY interfaces
    // to introduce entropy for address derivation.

    fn get_nonce(&self) -> u64;

    fn increase_nonce(&mut self);
}
