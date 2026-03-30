use uuid::Uuid;
use std::fmt;
use std::path::{Path, PathBuf};
 
use crate::routing::model::{Bundle};
use super::storage_interface::{StorageLayer, StorageError};


// implementation of StorageLayer
impl StorageLayer for JsonFileStorage{
    // Issue #11
    // Duplicate detection logic 
    fn save_bundle(&mut self, bundle: &Bundle) -> bool {
        let mut bundles = self.read_db();
        //check if the bundle ID already exists in the shared file bundle.json
        if bundles.iter().any(|b| b.id == bundle.id) {
            eprintln!("{}", StorageError::AlreadyExists(bundle.id.to_string()));
            return false;
        }

        // Add the new bundle to the array
        bundles.push(bundle.clone());

        //Save the updates array back to disk
        if self.write_db(&bundles) {
            return true;
        } else {
            eprintln!("{}", StorageError::SerializationError(bundle.id.to_string()));
            return false;
        }
    }

    fn get_bundle(&self, bundle_id: Uuid) -> Option<Bundle> {
        let bundles = self.read_db();
        //Serach the array for the requested bundle and return it if found
        bundles.into_iter().find(|b| b.id == bundle_id)
    }

    fn get_all_bundles(&self) -> Vec<Bundle> {
        self.read_db()
    }

    fn get_bundles_by_node(&self, node_id: Uuid) -> Vec<Uuid> {
        let bundles = self.read_db();
        bundles.into_iter()
            .filter(|b| b.source.id == node_id)
            .map(|b| b.id)
            .collect()
    }

    // Issue #13
    //Implement bundle deletion after delivery
    fn delete_bundle(&mut self, bundle_id: Uuid) -> bool {
        let mut bundles = self.read_db();
        let initial_len = bundles.len();

        // keep all bundles except the one matching the id we want to delete
        bundles.retain(|b| b.id != bundle_id);

        if bundles.len() < initial_len {
            //A bundle was removed so we overwrite the JSON file with the new array
            self.write_db(&bundles);
            return true;
        } else {
            //No bundle with the requested id was found in the storage
            eprintln!("{}", StorageError::NotFound(bundle_id.to_string()));
            return false;
        }
    }

    //Issue #12
    //iterates through the shared file, checks expiration and removes expired bundles
    fn cleanup_expired_bundles(&mut self) -> usize {
        let initial_len = self.bundles.len();

        //keep only bundles that are not expired
        self.bundles.retain(|b| !b.is_expired());

        let removed = initial_len - self.bundles.len();
        if removed > 0 {
        //save_to_file() returns a Result (Success or Error). 
        // Rust's safety rules require us to acknowledge this Result.
        // By assigning it to the underscore wildcard (let _ =), we tell the 
        // compiler: "I am intentionally ignoring the return value because 
        // save_to_file() already prints its own error messages."
            let _ = self.save_to_file();
        }

        removed
    }

}

