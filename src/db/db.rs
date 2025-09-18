use mongodb::{
    Client,
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
};

use std::env;

pub async fn get_client() -> mongodb::error::Resuly<Client> {
    let db_uri = env::var("DATABASE_URL").expect("You must set the MONGODB_URI environment var!");
    let mut client_options = ClientOptions::parse(db_uri).await?;

    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();

    client_options.server_api = Some(server_api);

    Ok(Client::with_options(client_options)?)
}

pub async fn test_connection(client: &Client) -> mongodb::error::Result<()> {
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await?;
    println!("Connected successfully to MongoDB.");
    Ok(())
}
