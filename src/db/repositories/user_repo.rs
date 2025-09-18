use mongodb::{
    Collection, Database,
    bson::{doc, to_bson},
};

use crate::model::model::User;

use tracing::{debug, error, info};

pub struct UserRepo {
    collection: Collection<User>,
}

impl UserRepo {
    pub fn new(db: &Database) -> Self {
        debug!("Creating UserRepo");
        let collection = db.collection::<User>("users");
        UserRepo { collection }
    }

    pub async fn create_user(&self, user: User) -> mongodb::error::Result<User> {
        info!("Creating new user with email: {}", user.email);

        match self.collection.insert_one(user.clone()).await {
            Ok(_) => {
                info!("User created successfully with email: {}", user.email);
                Ok(user)
            }
            Err(e) => {
                error!(
                    "Failed to create user with email: {}. Error: {}",
                    user.email, e
                );
                Err(e)
            }
        }
    }

    pub async fn find_by_email(&self, email: &str) -> mongodb::error::Result<Option<User>> {
        debug!("Finding user by email: {}", email);
        let result = self.collection.find_one(doc! { "email": email }).await;

        match &result {
            Ok(Some(_)) => debug!("User with email {} found", email),
            Ok(None) => debug!("User with email {} not found", email),
            Err(e) => error!("Error finding user with email {}: {}", email, e),
        }

        result
    }

    pub async fn find_by_id(&self, id: u64) -> mongodb::error::Result<Option<User>> {
        debug!("Finding user by id: {}", id);
        let id_bson = to_bson(&id).unwrap_or_else(|e| {
            error!("Error converting id to BSON: {}", e);
            bson::Bson::Null
        });

        let result = self.collection.find_one(doc! { "id": id_bson }).await;

        match &result {
            Ok(Some(_)) => debug!("User with id {} found", id),
            Ok(None) => debug!("User with id {} not found", id),
            Err(e) => error!("Error finding user with id {}: {}", id, e),
        }

        result
    }

    pub async fn update_user(&self, user: User) -> mongodb::error::Result<User> {
        info!("Updating user with id: {}", user.id);
        let id_bson = to_bson(&user.id).unwrap_or_else(|e| {
            error!("Error converting id to BSON: {}", e);
            bson::Bson::Null
        });

        match self
            .collection
            .replace_one(doc! { "id": id_bson }, user.clone(), None)
            .await
        {
            Ok(update_result) => {
                if update_result.matched_count == 1 {
                    info!("User with id {} updated successfully", user.id);
                    Ok(user)
                } else {
                    error!("No user found with id {} to update", user.id);
                    Err(mongodb::error::Error::from(
                        mongodb::error::ErrorKind::ArgumentError {
                            message: format!("No user found with id {}", user.id),
                        },
                    ))
                }
            }
            Err(e) => {
                error!("Failed to update user with id {}: {}", user.id, e);
                Err(e)
            }
        }
    }

    pub async fn delete_user(&self, id: u64) -> mongodb::error::Result<bool> {
        info!("Deleting user with id: {}", id);
        let id_bson = to_bson(&id).unwrap_or_else(|e| {
            error!("Error converting id to BSON: {}", e);
            bson::Bson::Null
        });

        match self
            .collection
            .delete_one(doc! { "id": id_bson }, None)
            .await
        {
            Ok(delete_result) => {
                if delete_result.deleted_count == 1 {
                    info!("User with id {} deleted successfully", id);
                    Ok(true)
                } else {
                    error!("No user found with id {} to delete", id);
                    Ok(false)
                }
            }
            Err(e) => {
                error!("Failed to delete user with id {}: {}", id, e);
                Err(e)
            }
        }
    }
}
