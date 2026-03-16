use uuid::Uuid;
use crate::routing::model::{Bundle, MsgStatus};
use crate::storage::StorageLayer;

pub struct BundleManager {
    pub node_id: Uuid,
    pub storage: StorageLayer,
}

impl BundleManager {
     // Function to get bundles stored at the node, used by the engine to get the summary vector
    pub fn get_bundles_from_node(&self, node_id: Uuid) -> Vec<Uuid> {
        self.storage.get_bundles_by_node(node_id)
    }

    // Function to get a bundle by its id, used by the SCF to fetch the full bundle before forwarding
    pub fn get(&self, bundle_id: &str) -> Option<Bundle> {
        self.storage.get_bundle(bundle_id)        
    }

    // Function to delete a bundle by its id, used by the SCF to remove bundles that have been forwarded or expired
    pub fn delete_bundle(&mut self, bundle_id: &str) {
        self.storage.delete_bundle(bundle_id);
    }

    // Function to get all bundles stored at the node, used by the SCF to drop expired bundles
    pub fn all(&self) -> Vec<Bundle> {
        self.storage.get_all_bundles()
    }    
}
