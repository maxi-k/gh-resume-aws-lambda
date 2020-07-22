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

use std::error::Error;

// --------------------------------------------------------------------------------
// Load .env configuration at compile time
// --------------------------------------------------------------------------------
use load_dotenv::load_dotenv;
load_dotenv!();

// --------------------------------------------------------------------------------
// Github GraphQL Api
// --------------------------------------------------------------------------------
use graphql_client::*;
use reqwest::blocking::Client;

type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "rsrc/schema.github.graphql",
    query_path = "rsrc/github.repos.graphql",
    response_derives = "Debug"
)]
struct RepoView;

fn fetch_repos(number: i64) -> Result<repo_view::ResponseData, Box<dyn Error>>{
    let api_token = env!("GITHUB_API_TOKEN");
    let client = Client::builder().user_agent("Maxi").build()?;
    let query = RepoView::build_query(repo_view::Variables { top: number });
    let request = client
        .post("https://api.github.com/graphql")
        .bearer_auth(api_token)
        .json(&query);
    let body: Response<repo_view::ResponseData> = request.send()?.json()?;
    return if body.errors.is_some() {
        Err("Error while fetching github API.".into())
    } else {
        Ok(body.data.expect("missing response data"))
    }
}

// --------------------------------------------------------------------------------
// Convert to appropriate response
// --------------------------------------------------------------------------------
use std::collections::HashMap;

#[derive(Serialize, Clone, Debug)]
struct Skill {
    name: String,
    code_size: i64,
    color: String
}

fn extract_skills(data: repo_view::ResponseData) -> Vec<Skill> {
    let repos = match data.viewer.repositories.nodes {
        Some(nodes) => nodes,
        None => return vec![],
    };
    let default_color = "#000".to_string();
    let skills : HashMap<String, Skill> = repos.iter().fold(HashMap::new(), |mut map, repo| {
        if let Some(languages) = repo.as_ref()
            .and_then(|r| r.languages.as_ref())
            .and_then(|l| l.edges.as_ref()) {
            for opt_lang in languages {
                if let Some(lang) = opt_lang {
                    map.entry(lang.node.name.to_owned()).or_insert_with(|| Skill {
                        name: lang.node.name.to_owned(),
                        code_size: 0,
                        color: lang.node.color.as_ref().unwrap_or(&default_color).to_string()
                    }).code_size += lang.size
                }
            }
        }
        map
    });
    skills.values().cloned().collect()
}

// --------------------------------------------------------------------------------
// Lambda Handler
// --------------------------------------------------------------------------------
use lambda::error::HandlerError;

#[derive(Deserialize, Clone)]
struct APIRequest {
    #[serde(rename = "top")]
    top: i64
}

#[derive(Serialize, Clone, Debug)]
struct APIResponse  {
    skills: Vec<Skill>
}

fn request_handler(e: APIRequest, c: lambda::Context) -> Result<APIResponse, HandlerError> {
    if e.top == 0 {
        error!("Requesting zero github skills in request {}", c.aws_request_id);
        return Err("No skills requested.".into());
    }
    let repos = match fetch_repos(5){
        Ok(value) => value,
        Err(err) => return Err(err.to_string().as_str().into())
    };
    let skills = extract_skills(repos);

    Ok(APIResponse { skills })
}

// --------------------------------------------------------------------------------
// Main Entrypoint
// --------------------------------------------------------------------------------
fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info)?;
    // Test handler
    // let result = request_handler(APIRequest{ top: 100 }, Default::default());
    // println!("{:?}", result);
    lambda!(request_handler);
    Ok(())
}

// --------------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn returns_ok() {
        let result = request_handler(APIRequest{ top: 2 }, Default::default());
        assert!(result.is_ok());
    }
}