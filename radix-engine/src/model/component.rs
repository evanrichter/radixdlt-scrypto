use crate::model::{convert, MethodAuthorization, ValidatedData};
use sbor::*;
use scrypto::engine::types::*;
use scrypto::prelude::ComponentAuthorization;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;

/// A component is an instance of blueprint.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Component {
    package_id: PackageId,
    blueprint_name: String,
    method_auth: ComponentAuthorization,
    state: Vec<u8>,
}

impl Component {
    pub fn new(
        package_id: PackageId,
        blueprint_name: String,
        method_auth: ComponentAuthorization,
        state: Vec<u8>,
    ) -> Self {
        Self {
            package_id,
            blueprint_name,
            method_auth,
            state,
        }
    }

    pub fn method_authorization(
        &self,
        schema: &Type,
        method_name: &str,
    ) -> (ValidatedData, MethodAuthorization) {
        let data = ValidatedData::from_slice(&self.state).unwrap();
        let authorization = match self.method_auth.get(method_name) {
            Some(auth) => convert(schema, &data.dom, auth),
            None => MethodAuthorization::Private,
        };

        (data, authorization)
    }

    pub fn authorization(&self) -> &ComponentAuthorization {
        &self.method_auth
    }

    pub fn package_id(&self) -> PackageId {
        self.package_id.clone()
    }

    pub fn blueprint_name(&self) -> &str {
        &self.blueprint_name
    }

    pub fn state(&self) -> &[u8] {
        &self.state
    }

    pub fn set_state(&mut self, new_state: Vec<u8>) {
        self.state = new_state;
    }
}
