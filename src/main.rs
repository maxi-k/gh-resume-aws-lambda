// Serde JSON serializer
#[macro_use]
extern crate serde_derive;
// logging
#[macro_use]
extern crate tracing;

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

fn fetch_repos(number: u16) -> Result<repo_view::ResponseData, Box<dyn std::error::Error>> {
    let api_token = env!("GITHUB_API_TOKEN");
    let client = Client::builder().user_agent("Maxi").build()?;
    let query = RepoView::build_query(repo_view::Variables { top_repo: number as i64, top_lang: 7 });
    let request = client
        .post("https://api.github.com/graphql")
        .bearer_auth(api_token)
        .json(&query);
    let body: Response<repo_view::ResponseData> = request.send()?.json()?;
    return if body.errors.is_some() {
        Err("Error while fetching github API.".into())
    } else {
        Ok(body.data.expect("missing response data"))
    };
}

// --------------------------------------------------------------------------------
// Convert to appropriate response
// --------------------------------------------------------------------------------
use std::collections::HashMap;
use std::convert::identity;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct Skill {
    name: String,
    code_size: u64,
    color: String,
}

static DEFAULT_COLOR: &str = "#000";

macro_rules! repos_to_skills {
    ($nodes:expr, $exclude:expr) => {
        $nodes
            .into_iter()
            .filter_map(identity)
            .filter(|r| !$exclude.contains(&r.name.to_owned()))
            .flat_map(|r| r.languages)
            .flat_map(|l| l.edges)
            .flatten()
            .filter_map(identity)
            .map(|lang| Skill{
                name: lang.node.name.to_owned(),
                code_size: lang.size as u64,
                color: lang.node.color.as_ref().unwrap_or(&DEFAULT_COLOR.to_string()).to_string(),
            })
    }
}

fn extract_skills(data: repo_view::ResponseData, exclude: HashSet<String>) -> Vec<Skill> {
    let personal = repos_to_skills!(data.viewer.repositories.nodes.unwrap_or(vec![]) , &exclude);
    let contributions = repos_to_skills!(data.viewer.repositories_contributed_to.nodes.unwrap_or(vec![]), &exclude);
    let combined = personal.chain(contributions);
    let skills: HashMap<String, Skill> = combined.fold(HashMap::new(), |mut map, skill| {
        map.entry(skill.name.to_owned())
           .and_modify(|s| s.code_size += skill.code_size)
           .or_insert(skill);
        map
    });
    skills.values().cloned().collect()
}

// --------------------------------------------------------------------------------
// Lambda Handler
// --------------------------------------------------------------------------------
use std::collections::HashSet;

#[derive(Deserialize, Clone)]
struct APIRequest {
    top: Option<u16>,
    exclude: Option<HashSet<String>>
}

#[derive(Serialize, Clone, Debug)]
struct APIResponse {
    skills: Vec<Skill>,
}

fn request_handler(e: APIRequest) -> Result<APIResponse, lambda_http::Error> {
    let top = e.top.unwrap_or(50);
    if top == 0 {
        event!(tracing::Level::ERROR, "Requesting zero github skills");
        return Err("No skills requested.".into());
    }
    let exclude = e.exclude.unwrap_or(HashSet::new());
    let repos = match fetch_repos(top) {
        Ok(value) => value,
        Err(err) => return Err(err.to_string().as_str().into()),
    };
    let skills = extract_skills(repos, exclude);
    Ok(APIResponse { skills })
}

// --------------------------------------------------------------------------------
// lambda-specific code
// --------------------------------------------------------------------------------
// AWS lambda runtime
// use lambda_http::{run, service_fn, Body, Request, RequestExt, Response, Error};
use lambda_runtime as aws;

async fn function_handler(event: aws::LambdaEvent<APIRequest>) -> Result<APIResponse, aws::Error> {
    request_handler(event.payload).map_err(|e| aws::Error::from(e.to_string()))
}

#[tokio::main]
async fn main() -> Result<(), aws::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    aws::run(aws::service_fn(function_handler)).await
}

// --------------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handler_returns_ok_with_argument() {
        let req = APIRequest { top: Some(2), exclude: None };
        let result = request_handler(req);
        assert!(result.is_ok());
    }

    #[test]
    fn handler_returns_ok_without_argument() {
        let req = APIRequest { top: None, exclude: None };
        let result = request_handler(req);
        assert!(result.is_ok());
    }

    #[test]
    fn handler_returns_error_with_zero_argument() {
        let req = APIRequest { top: Some(0), exclude: None };
        let result = request_handler(req);
        assert!(result.is_err());
    }
}
