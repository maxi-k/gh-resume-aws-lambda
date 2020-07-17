use anyhow::*;
// AWS lambda runtime
#[macro_use]
extern crate lambda_runtime as lambda;
// Serde JSON serializer
#[macro_use]
extern crate serde_derive;
// Logging
#[macro_use]
extern crate log;
extern crate simple_logger;

// --------------------------------------------------------------------------------
// Github GraphQL Api
// --------------------------------------------------------------------------------
use graphql_client::*;
use reqwest::blocking;
use structopt::StructOpt;

type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "rsrc/schema.github.graphql",
    query_path = "rsrc/github.repos.graphql",
    response_derives = "Debug"
)]
struct RepoView;

#[derive(Deserialize, Debug)]
struct Env {
    github_api_token: String,
}

fn fetch_repos(config: &Env) -> Result<repo_view::ResponseData, Box<dyn Error>>{
    // TODO: use tokio for async runtime w/ lambda?
    let client = reqwest::blocking::Client::builder().user_agent("Maxi").build()?;
    let query = RepoView::build_query(repo_view::Variables);
    let mut request = client
        .post("https://api.github.com/graphql")
        .bearer_auth(config.github_api_token.to_owned())
        .json(&query);
    let body = request.send()?.json()?;
    return if body.errors {
        Err("Error while fetching github API.".into())
    } else {
        Ok(body.data.expect("missing response data"))
    }
}


// --------------------------------------------------------------------------------
// Lambda Handler
// --------------------------------------------------------------------------------

use lambda::error::HandlerError;
use std::error::Error;

#[derive(Deserialize, Clone)]
struct APIRequest {
    #[serde(rename = "firstName")]
    first_name: String,
}

#[derive(Serialize, Clone)]
struct APIResponse {
    message: String,
}

fn request_handler(e: APIRequest, c: lambda::Context) -> Result<APIResponse, HandlerError> {
    if e.first_name == "" {
        error!("Empty first name in request {}", c.aws_request_id);
        return Err(c.new_error("Empty first name"));
    }

    Ok(APIResponse {
        message: format!("Hello, {}!", e.first_name),
    })
}

// --------------------------------------------------------------------------------
// Main Entrypoint
// --------------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info)?;
    // read .env
    dotenv::dotenv().ok();
    let config: Env = envy::from_env().context("while reading from environment")?;
    let repos = fetch_repos(&config)?;
    println!("{:?}", repos);
    // lambda!(request_handler);
    Ok(())
}
